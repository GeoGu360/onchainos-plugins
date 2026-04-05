use clap::Args;
use serde_json::json;
use crate::config::{LENDING_FACTORY, get_rpc};
use crate::onchainos::resolve_wallet_from_addresses;
use crate::rpc::*;

#[derive(Args)]
pub struct PositionsArgs {
    /// EVM chain ID (default: 1)
    #[arg(long, default_value = "1")]
    pub chain: u64,

    /// Wallet address (resolved from onchainos if not provided)
    #[arg(long)]
    pub address: Option<String>,

    /// Market name or index to check (checks all if not provided)
    #[arg(long)]
    pub market: Option<String>,
}

pub async fn run(args: PositionsArgs) -> anyhow::Result<()> {
    let rpc = get_rpc(args.chain);

    // Resolve wallet address
    let wallet = if let Some(addr) = args.address {
        addr
    } else {
        resolve_wallet_from_addresses("1")
            .or_else(|_| crate::onchainos::resolve_wallet(args.chain))?
    };

    // Get market count
    let count_raw = eth_call_raw(rpc, LENDING_FACTORY, "0xfd775c78")?;
    let count = decode_uint256(&count_raw) as u64;

    // Determine which markets to check
    let indices: Vec<u64> = if let Some(ref market_ref) = args.market {
        if let Ok(idx) = market_ref.parse::<u64>() {
            vec![idx]
        } else {
            let mut found = Vec::new();
            for i in 0..std::cmp::min(count, 46) {
                let name = eth_call_string(rpc, LENDING_FACTORY, &calldata_factory_names(i))
                    .unwrap_or_default();
                if name.to_lowercase().contains(&market_ref.to_lowercase()) {
                    found.push(i);
                }
            }
            found
        }
    } else {
        (0..std::cmp::min(count, 20)).collect()
    };

    let mut active_positions = Vec::new();
    let mut checked = 0u64;

    for i in indices {
        let controller = eth_call_address(rpc, LENDING_FACTORY, &calldata_factory_controllers(i))
            .unwrap_or_default();
        if controller.is_empty() {
            continue;
        }

        // loan_exists(wallet)?
        let exists_raw = eth_call_raw(rpc, &controller, &calldata_loan_exists(&wallet))
            .unwrap_or_default();
        let exists = decode_uint256(&exists_raw) > 0;
        checked += 1;

        if !exists {
            continue;
        }

        let name = eth_call_string(rpc, LENDING_FACTORY, &calldata_factory_names(i))
            .unwrap_or_else(|_| format!("market-{}", i));
        let collateral_token = eth_call_address(
            rpc,
            LENDING_FACTORY,
            &calldata_factory_collateral_tokens(i),
        )
        .unwrap_or_default();
        let collateral_symbol = if !collateral_token.is_empty() {
            eth_call_string(rpc, &collateral_token, calldata_symbol()).unwrap_or_default()
        } else {
            String::new()
        };

        // debt
        let debt_raw = eth_call_raw(rpc, &controller, &calldata_debt(&wallet))
            .unwrap_or_default();
        let debt = decode_uint256(&debt_raw);

        // user_state
        let state_raw = eth_call_raw(rpc, &controller, &calldata_user_state(&wallet))
            .unwrap_or_default();
        let (collateral, stablecoin, _debt2, n1, n2) =
            decode_user_state(&state_raw).unwrap_or((0, 0, 0, 0, 0));

        // user_prices
        let prices_raw = eth_call_raw(rpc, &controller, &calldata_user_prices(&wallet))
            .unwrap_or_default();
        let (price_high, price_low) = decode_user_prices(&prices_raw).unwrap_or((0, 0));

        // health
        let health_raw = eth_call_raw(rpc, &controller, &calldata_health(&wallet, false))
            .unwrap_or_default();
        let health = decode_int256_as_i128(&health_raw);
        let health_pct = health as f64 / 1e16; // health is scaled 1e18; display as %

        // collateral decimals
        let coll_dec = if !collateral_token.is_empty() {
            let dec_raw = eth_call_uint256(rpc, &collateral_token, calldata_decimals()).unwrap_or(18);
            dec_raw as u32
        } else {
            18
        };
        let coll_divisor = 10u128.pow(coll_dec) as f64;

        active_positions.push(json!({
            "market_index": i,
            "market_name": name,
            "controller": controller,
            "collateral_token": collateral_token,
            "collateral_symbol": collateral_symbol,
            "collateral_amount": format!("{:.6}", collateral as f64 / coll_divisor),
            "collateral_in_crvusd": format!("{:.6}", stablecoin as f64 / 1e18),
            "debt_crvusd": format!("{:.6}", debt as f64 / 1e18),
            "band_low": n1,
            "band_high": n2,
            "liquidation_price_high": format!("{:.2}", price_high as f64 / 1e18),
            "liquidation_price_low": format!("{:.2}", price_low as f64 / 1e18),
            "health_pct": format!("{:.4}", health_pct),
            "is_healthy": health > 0,
        }));
    }

    let output = json!({
        "ok": true,
        "chain": args.chain,
        "wallet": wallet,
        "markets_checked": checked,
        "active_positions": active_positions,
        "position_count": active_positions.len(),
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
