use clap::Args;
use serde_json::json;

#[derive(Args)]
pub struct OpenPositionArgs {
    /// Market symbol or index token address (e.g. "ETH" or "ETH/USD")
    #[arg(long)]
    pub market: String,

    /// Collateral token address (e.g. USDC on Arbitrum: 0xaf88d065e77c8cC2239327C5EDb3A432268e5831)
    #[arg(long)]
    pub collateral_token: String,

    /// Collateral amount in smallest units (e.g. 1000000000 for 1000 USDC with 6 decimals)
    #[arg(long)]
    pub collateral_amount: u128,

    /// Position size in USD (e.g. 5000.0 for $5000 leveraged position)
    #[arg(long)]
    pub size_usd: f64,

    /// Long (true) or short (false)
    #[arg(long)]
    pub long: bool,

    /// Slippage in basis points (default: 100 = 1%)
    #[arg(long, default_value_t = 100)]
    pub slippage_bps: u32,

    /// Wallet address (defaults to logged-in wallet)
    #[arg(long)]
    pub from: Option<String>,
}

pub async fn run(chain: &str, dry_run: bool, args: OpenPositionArgs) -> anyhow::Result<()> {
    let cfg = crate::config::get_chain_config(chain)?;

    let wallet = args.from.clone().unwrap_or_else(|| {
        crate::onchainos::resolve_wallet(cfg.chain_id).unwrap_or_default()
    });
    if wallet.is_empty() {
        anyhow::bail!("Cannot determine wallet address. Pass --from or ensure onchainos is logged in.");
    }

    // Fetch markets to find the target market
    let markets = crate::api::fetch_markets(cfg).await?;
    let market = crate::api::find_market_by_symbol(&markets, &args.market)
        .ok_or_else(|| anyhow::anyhow!("Market '{}' not found on {}", args.market, chain))?;

    let market_token = market.market_token.as_deref()
        .ok_or_else(|| anyhow::anyhow!("Market has no marketToken address"))?;
    let index_token = market.index_token.as_deref()
        .ok_or_else(|| anyhow::anyhow!("Market has no indexToken (swap-only market?)"))?;

    // Fetch prices
    let tickers = crate::api::fetch_prices(cfg).await?;
    let price_tick = crate::api::find_price(&tickers, index_token)
        .ok_or_else(|| anyhow::anyhow!("Price not found for index token {}", index_token))?;

    let min_price_raw: u128 = price_tick.min_price.as_deref().unwrap_or("0").parse().unwrap_or(0);
    let max_price_raw: u128 = price_tick.max_price.as_deref().unwrap_or("0").parse().unwrap_or(0);
    // GMX prices are stored as price_usd * 10^(30 - token_decimals).
    // For display, fetch token decimals to convert properly. Default to 18.
    let token_infos = crate::api::fetch_tokens(cfg).await.unwrap_or_default();
    let index_decimals = token_infos.iter()
        .find(|t| t.address.as_deref().map(|a| a.to_lowercase()) == Some(index_token.to_lowercase()))
        .and_then(|t| t.decimals)
        .unwrap_or(18u8);
    let min_price_usd = crate::api::raw_price_to_usd(min_price_raw, index_decimals);
    let max_price_usd = crate::api::raw_price_to_usd(max_price_raw, index_decimals);
    let mid_price_usd = (min_price_usd + max_price_usd) / 2.0;

    // Size in GMX 30-decimal units (price_usd * 10^30).
    // Use integer arithmetic to avoid float precision issues.
    // args.size_usd is a float; scale cents (2 decimal places) then multiply by 10^28.
    let size_dollars = args.size_usd as u128;
    let size_cents = ((args.size_usd - size_dollars as f64) * 100.0).round() as u128;
    let size_delta_usd = (size_dollars * 100 + size_cents) * 10u128.pow(28);

    // Check liquidity
    let avail_liq = if args.long {
        market.available_liquidity_long.as_deref().unwrap_or("0").parse::<u128>().unwrap_or(0)
    } else {
        market.available_liquidity_short.as_deref().unwrap_or("0").parse::<u128>().unwrap_or(0)
    };
    let avail_liq_usd = avail_liq as f64 / 1e30;
    if size_delta_usd > avail_liq {
        anyhow::bail!(
            "Insufficient liquidity. Required: ${:.2} USD, Available: ${:.2} USD",
            args.size_usd,
            avail_liq_usd
        );
    }

    // Compute acceptable price with slippage.
    // GMX API returns prices as price_usd * 10^(30 - index_decimals).
    // GMX contract requires prices in full 30-decimal precision (price_usd * 10^30).
    // Scale up: multiply by 10^index_decimals to convert to contract precision.
    let price_scale_factor: u128 = 10u128.pow(index_decimals as u32);
    let min_price_30dec = min_price_raw.saturating_mul(price_scale_factor);
    let max_price_30dec = max_price_raw.saturating_mul(price_scale_factor);
    let base_price_30dec = if args.long { min_price_30dec } else { max_price_30dec };
    let acceptable_price = crate::abi::compute_acceptable_price(base_price_30dec, args.long, args.slippage_bps);

    let execution_fee = cfg.execution_fee_wei;

    // Check ERC-20 allowance and approve if needed
    if !dry_run {
        let allowance = crate::onchainos::check_allowance(
            cfg.rpc_url,
            &args.collateral_token,
            &wallet,
            cfg.router,
        ).await.unwrap_or(0);

        if allowance < args.collateral_amount {
            eprintln!("Approving collateral token to Router...");
            let approve_result = crate::onchainos::erc20_approve(
                cfg.chain_id,
                &args.collateral_token,
                cfg.router,
                u128::MAX,
                Some(&wallet),
                false,
            ).await?;
            let approve_hash = crate::onchainos::extract_tx_hash(&approve_result)
                .unwrap_or_else(|e| format!("(approval error: {})", e));
            eprintln!("Approval tx: {}", approve_hash);
        }
    }

    // Determine if collateral is native ETH (WETH address on each chain).
    // For GMX V2: WETH collateral must be deposited via sendWnt (ETH auto-wraps in vault),
    // NOT via sendTokens (which transfers ERC-20 WETH from wallet).
    // Total ETH sent as tx value = executionFee + collateralAmount (if WETH) or executionFee only.
    let weth_arb = "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1";
    let weth_avax = "0x49D5c2BdFfac6CE2BFdB6640F4F80f226bc10bAB";
    let collateral_is_native_eth = args.collateral_token.to_lowercase() == weth_arb.to_lowercase()
        || args.collateral_token.to_lowercase() == weth_avax.to_lowercase();

    let (multicall_calls, total_eth_value): (Vec<String>, u64) = if collateral_is_native_eth {
        // Single sendWnt for both execution fee and collateral; no ERC-20 sendTokens needed
        let total_wnt = execution_fee + args.collateral_amount as u64;
        let send_wnt = crate::abi::encode_send_wnt(cfg.order_vault, total_wnt);
        let create_order = crate::abi::encode_create_order(
            &wallet,   // _account (unused, msg.sender implicit)
            &wallet,   // receiver
            market_token,
            &args.collateral_token,
            2, // MarketIncrease
            size_delta_usd,
            args.collateral_amount,
            0, // triggerPrice = 0 for market orders
            acceptable_price,
            execution_fee,
            args.long,
            cfg.chain_id, // _src_chain_id (unused)
        );
        (vec![send_wnt, create_order], total_wnt)
    } else {
        // ERC-20 collateral: sendWnt for fee + sendTokens for collateral
        let send_wnt = crate::abi::encode_send_wnt(cfg.order_vault, execution_fee);
        let send_tokens = crate::abi::encode_send_tokens(
            &args.collateral_token,
            cfg.order_vault,
            args.collateral_amount,
        );
        let create_order = crate::abi::encode_create_order(
            &wallet,   // _account (unused, msg.sender implicit)
            &wallet,   // receiver
            market_token,
            &args.collateral_token,
            2, // MarketIncrease
            size_delta_usd,
            args.collateral_amount,
            0, // triggerPrice = 0 for market orders
            acceptable_price,
            execution_fee,
            args.long,
            cfg.chain_id, // _src_chain_id (unused)
        );
        (vec![send_wnt, send_tokens, create_order], execution_fee)
    };

    let multicall_hex = crate::abi::encode_multicall(&multicall_calls);
    let calldata = format!("0x{}", multicall_hex);

    let leverage = if mid_price_usd > 0.0 {
        args.size_usd / (args.collateral_amount as f64 / 1e6)
    } else {
        0.0
    };

    // Preview
    eprintln!("=== Open Position Preview ===");
    eprintln!("Market: {}", market.name.as_deref().unwrap_or("?"));
    eprintln!("Direction: {}", if args.long { "LONG" } else { "SHORT" });
    eprintln!("Size: ${:.2} USD", args.size_usd);
    eprintln!("Collateral: {} units", args.collateral_amount);
    eprintln!("Current price: ${:.4}", mid_price_usd);
    eprintln!("Acceptable price: {}", acceptable_price);
    eprintln!("Execution fee: {} wei", execution_fee);
    eprintln!("Estimated leverage: {:.1}x", leverage);
    eprintln!("⚠ GMX V2 uses a keeper model — position opens 1-30s after tx lands.");
    eprintln!("Ask user to confirm before proceeding.");

    let result = crate::onchainos::wallet_contract_call(
        cfg.chain_id,
        cfg.exchange_router,
        &calldata,
        Some(&wallet),
        Some(total_eth_value),
        dry_run,
    ).await?;

    let tx_hash = crate::onchainos::extract_tx_hash(&result)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "ok": true,
            "dry_run": dry_run,
            "chain": chain,
            "txHash": tx_hash,
            "market": market.name,
            "marketToken": market_token,
            "direction": if args.long { "long" } else { "short" },
            "sizeDeltaUsd": args.size_usd,
            "collateralAmount": args.collateral_amount.to_string(),
            "entryPrice_approx_usd": format!("{:.4}", mid_price_usd),
            "acceptablePrice": acceptable_price.to_string(),
            "executionFeeWei": execution_fee,
            "note": "GMX V2 keeper model: position will open within 1-30s after tx confirmation",
            "calldata": if dry_run { Some(calldata.as_str()) } else { None }
        }))?
    );
    Ok(())
}
