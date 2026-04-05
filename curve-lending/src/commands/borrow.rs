use clap::Args;
use serde_json::json;
use crate::config::{LENDING_FACTORY, get_rpc};
use crate::onchainos::{resolve_wallet_from_addresses, wallet_contract_call, extract_tx_hash};
use crate::rpc::*;

#[derive(Args)]
pub struct BorrowArgs {
    /// Market name (e.g. "WETH-long") or index
    #[arg(long)]
    pub market: String,

    /// Amount of crvUSD to borrow
    #[arg(long)]
    pub amount: f64,

    /// Additional collateral to deposit (token units), optional for borrow_more
    #[arg(long, default_value = "0")]
    pub collateral: f64,

    /// EVM chain ID (default: 1)
    #[arg(long, default_value = "1")]
    pub chain: u64,

    /// Number of LLAMMA bands for new loan (4-50, default 10)
    #[arg(long, default_value = "10")]
    pub bands: u64,

    /// Simulate without broadcasting (GUARDRAILS: borrow is dry-run only in test env)
    #[arg(long)]
    pub dry_run: bool,
}

pub async fn run(args: BorrowArgs) -> anyhow::Result<()> {
    let rpc = get_rpc(args.chain);

    // Resolve wallet
    let wallet = if args.dry_run {
        "0x0000000000000000000000000000000000000000".to_string()
    } else {
        resolve_wallet_from_addresses("1")
            .or_else(|_| crate::onchainos::resolve_wallet(args.chain))?
    };

    let count_raw = eth_call_raw(rpc, LENDING_FACTORY, "0xfd775c78")?;
    let count = decode_uint256(&count_raw) as u64;

    let (market_idx, market_name, controller, collateral_token) =
        find_market(rpc, &args.market, count)?;

    // Get collateral decimals
    let coll_decimals = eth_call_uint256(rpc, &collateral_token, calldata_decimals())
        .unwrap_or(18) as u32;
    let coll_symbol = eth_call_string(rpc, &collateral_token, calldata_symbol())
        .unwrap_or_default();

    let coll_divisor = 10u128.pow(coll_decimals);
    let collateral_raw = (args.collateral * coll_divisor as f64) as u128;
    let debt_raw = (args.amount * 1e18) as u128;

    // Check loan status
    let exists_raw = eth_call_raw(rpc, &controller, &calldata_loan_exists(&wallet))
        .unwrap_or_default();
    let loan_exists = decode_uint256(&exists_raw) > 0;

    // Max borrowable check
    let max_borrow = eth_call_uint256(
        rpc,
        &controller,
        &calldata_max_borrowable(collateral_raw, args.bands),
    )
    .unwrap_or(0);

    eprintln!("Market: {} (index {})", market_name, market_idx);
    eprintln!("Borrow amount: {} crvUSD", args.amount);
    eprintln!("Additional collateral: {} {}", args.collateral, coll_symbol);
    eprintln!("Loan exists: {}", loan_exists);
    eprintln!("Max borrowable with collateral: {:.4} crvUSD", max_borrow as f64 / 1e18);

    // Build calldata
    let (action, calldata) = if loan_exists {
        (
            "borrow_more",
            calldata_borrow_more(collateral_raw, debt_raw),
        )
    } else {
        if collateral_raw == 0 {
            anyhow::bail!(
                "No active loan and no collateral specified. \
                 Provide --collateral <amount> to create a new loan."
            );
        }
        (
            "create_loan",
            calldata_create_loan(collateral_raw, debt_raw, args.bands),
        )
    };

    if args.dry_run {
        let approve_calldata = if collateral_raw > 0 {
            calldata_approve(&controller, collateral_raw)
        } else {
            String::new()
        };

        let output = json!({
            "ok": true,
            "dry_run": true,
            "market": market_name,
            "action": action,
            "borrow_amount_crvusd": args.amount,
            "collateral": args.collateral,
            "collateral_symbol": coll_symbol,
            "approve_calldata": approve_calldata,
            "calldata": calldata,
            "max_borrowable_crvusd": format!("{:.4}", max_borrow as f64 / 1e18),
            "note": "Dry-run mode. This is a borrow operation — ask user to confirm and ensure sufficient collateral before executing.",
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    // ── Real execution ─────────────────────────────────────────────────────────
    let mut results = json!({
        "ok": false,
        "market": market_name,
        "action": action,
        "wallet": wallet,
    });

    // Approve collateral if needed
    if collateral_raw > 0 {
        eprintln!("Approving collateral...");
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
        results["approve_txHash"] = json!(approve_hash);
    }

    // Execute borrow
    eprintln!("Executing {}...", action);
    let borrow_result = wallet_contract_call(
        args.chain,
        &controller,
        &calldata,
        Some(&wallet),
        None,
        false,
    )
    .await?;
    let borrow_hash = extract_tx_hash(&borrow_result);

    results["ok"] = json!(borrow_result["ok"].as_bool().unwrap_or(false));
    results["txHash"] = json!(borrow_hash);
    results["borrow_amount_crvusd"] = json!(args.amount);
    results["result"] = borrow_result;

    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}

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
