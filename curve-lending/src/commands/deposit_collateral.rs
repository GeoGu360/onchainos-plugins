use clap::Args;
use serde_json::json;
use crate::config::{LENDING_FACTORY, CRVUSD, get_rpc};
use crate::onchainos::{resolve_wallet_from_addresses, wallet_contract_call, extract_tx_hash};
use crate::rpc::*;

#[derive(Args)]
pub struct DepositCollateralArgs {
    /// Market name (e.g. "WETH-long") or index
    #[arg(long)]
    pub market: String,

    /// Amount of collateral to deposit (in token units, e.g. 0.001)
    #[arg(long)]
    pub amount: f64,

    /// EVM chain ID (default: 1)
    #[arg(long, default_value = "1")]
    pub chain: u64,

    /// Simulate without broadcasting
    #[arg(long)]
    pub dry_run: bool,

    /// Number of LLAMMA bands for new loan (4-50, default 10)
    #[arg(long, default_value = "10")]
    pub bands: u64,
}

pub async fn run(args: DepositCollateralArgs) -> anyhow::Result<()> {
    let rpc = get_rpc(args.chain);

    // Resolve wallet
    let wallet = if args.dry_run {
        "0x0000000000000000000000000000000000000000".to_string()
    } else {
        resolve_wallet_from_addresses("1")
            .or_else(|_| crate::onchainos::resolve_wallet(args.chain))?
    };

    // Find market by name or index
    let count_raw = eth_call_raw(rpc, LENDING_FACTORY, "0xfd775c78")?;
    let count = decode_uint256(&count_raw) as u64;

    let (market_idx, market_name, controller, collateral_token) = find_market(rpc, &args.market, count)?;

    // Get collateral token info
    let coll_decimals = eth_call_uint256(rpc, &collateral_token, calldata_decimals())
        .unwrap_or(18) as u32;
    let coll_symbol = eth_call_string(rpc, &collateral_token, calldata_symbol())
        .unwrap_or_default();

    // Convert amount to token units
    let divisor = 10u128.pow(coll_decimals);
    let collateral_raw = (args.amount * divisor as f64) as u128;

    // Check loan_exists
    let exists_raw = eth_call_raw(rpc, &controller, &calldata_loan_exists(&wallet))
        .unwrap_or_default();
    let loan_exists = decode_uint256(&exists_raw) > 0;

    // Check max_borrowable for reference
    let max_borrow = eth_call_uint256(
        rpc,
        &controller,
        &calldata_max_borrowable(collateral_raw, args.bands),
    )
    .unwrap_or(0);

    eprintln!("Market: {} (index {})", market_name, market_idx);
    eprintln!("Collateral: {} {} (raw: {})", args.amount, coll_symbol, collateral_raw);
    eprintln!("Loan exists: {}", loan_exists);
    eprintln!("Max borrowable with this collateral: {:.4} crvUSD", max_borrow as f64 / 1e18);

    if args.dry_run {
        // Build calldata for both cases without executing
        let (calldata, action) = if loan_exists {
            (
                calldata_add_collateral(collateral_raw, &wallet),
                "add_collateral",
            )
        } else {
            // For dry_run new loan: use min debt = 1 crvUSD to avoid zero-debt revert
            let min_debt = 1_000_000_000_000_000_000u128; // 1 crvUSD
            (
                calldata_create_loan(collateral_raw, min_debt, args.bands),
                "create_loan",
            )
        };

        let approve_calldata = calldata_approve(&controller, collateral_raw);

        let output = json!({
            "ok": true,
            "dry_run": true,
            "market": market_name,
            "action": action,
            "wallet": wallet,
            "collateral": args.amount,
            "collateral_symbol": coll_symbol,
            "collateral_raw": collateral_raw.to_string(),
            "approve_calldata": approve_calldata,
            "calldata": calldata,
            "max_borrowable_crvusd": format!("{:.4}", max_borrow as f64 / 1e18),
            "note": "Dry-run mode. Ask user to confirm before executing real transaction.",
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    // ── Real execution ─────────────────────────────────────────────────────────
    // Step 1: ERC-20 approve collateral token for controller
    eprintln!("Step 1: Approving {} for controller...", coll_symbol);
    let approve_data = calldata_approve(&controller, collateral_raw);
    let approve_result = wallet_contract_call(
        args.chain,
        &collateral_token,
        &approve_data,
        Some(&wallet),
        None,
        false,
    )
    .await?;
    let approve_hash = extract_tx_hash(&approve_result);
    eprintln!("Approve txHash: {}", approve_hash);

    if !approve_result["ok"].as_bool().unwrap_or(false) {
        anyhow::bail!("Approve failed: {}", approve_result);
    }

    // Step 2: deposit collateral
    eprintln!("Step 2: Depositing collateral...");
    let (deposit_data, action) = if loan_exists {
        (calldata_add_collateral(collateral_raw, &wallet), "add_collateral")
    } else {
        // Create new loan: user must also specify debt; use minimum (1 crvUSD) to just open position
        let min_debt = 1_000_000_000_000_000_000u128;
        eprintln!("No active loan — creating new loan with 1 crvUSD minimum debt");
        eprintln!("NOTE: You will need crvUSD to repay this minimum debt. Consider using --dry-run first.");
        (calldata_create_loan(collateral_raw, min_debt, args.bands), "create_loan")
    };

    let deposit_result = wallet_contract_call(
        args.chain,
        &controller,
        &deposit_data,
        Some(&wallet),
        None,
        false,
    )
    .await?;
    let deposit_hash = extract_tx_hash(&deposit_result);

    let output = json!({
        "ok": deposit_result["ok"].as_bool().unwrap_or(false),
        "market": market_name,
        "action": action,
        "wallet": wallet,
        "collateral": args.amount,
        "collateral_symbol": coll_symbol,
        "approve_txHash": approve_hash,
        "txHash": deposit_hash,
        "result": deposit_result,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Locate a market by name substring or numeric index.
/// Returns (index, name, controller_addr, collateral_token_addr)
fn find_market(
    rpc: &str,
    market_ref: &str,
    count: u64,
) -> anyhow::Result<(u64, String, String, String)> {
    if let Ok(idx) = market_ref.parse::<u64>() {
        let name = eth_call_string(rpc, LENDING_FACTORY, &calldata_factory_names(idx))
            .unwrap_or_else(|_| format!("market-{}", idx));
        let controller =
            eth_call_address(rpc, LENDING_FACTORY, &calldata_factory_controllers(idx))?;
        let collateral_token =
            eth_call_address(rpc, LENDING_FACTORY, &calldata_factory_collateral_tokens(idx))?;
        return Ok((idx, name, controller, collateral_token));
    }
    // Search by name substring
    for i in 0..std::cmp::min(count, 46) {
        let name = eth_call_string(rpc, LENDING_FACTORY, &calldata_factory_names(i))
            .unwrap_or_default();
        if name.to_lowercase().contains(&market_ref.to_lowercase()) {
            let controller =
                eth_call_address(rpc, LENDING_FACTORY, &calldata_factory_controllers(i))?;
            let collateral_token = eth_call_address(
                rpc,
                LENDING_FACTORY,
                &calldata_factory_collateral_tokens(i),
            )?;
            return Ok((i, name, controller, collateral_token));
        }
    }
    anyhow::bail!("Market '{}' not found", market_ref)
}
