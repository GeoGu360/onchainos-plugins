use clap::Args;
use serde_json::json;
use crate::config::{LENDING_FACTORY, CRVUSD, get_rpc};
use crate::onchainos::{resolve_wallet_from_addresses, wallet_contract_call, extract_tx_hash};
use crate::rpc::*;

#[derive(Args)]
pub struct RepayArgs {
    /// Market name (e.g. "WETH-long") or index
    #[arg(long)]
    pub market: String,

    /// Amount of crvUSD to repay. Use 0 for full repay (uses wallet balance).
    #[arg(long)]
    pub amount: f64,

    /// EVM chain ID (default: 1)
    #[arg(long, default_value = "1")]
    pub chain: u64,

    /// Simulate without broadcasting (GUARDRAILS: repay dry-run only in test env)
    #[arg(long)]
    pub dry_run: bool,
}

pub async fn run(args: RepayArgs) -> anyhow::Result<()> {
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

    let (market_idx, market_name, controller, _) = find_market(rpc, &args.market, count)?;

    // Check current debt
    let debt_raw = eth_call_raw(rpc, &controller, &calldata_debt(&wallet))
        .unwrap_or_default();
    let current_debt = decode_uint256(&debt_raw);

    // Repay amount: use wallet balance for full repay, otherwise specified amount
    // ⚠️ GUARDRAILS pitfall: never use uint256.max for repay — it reverts if wallet balance < accrued interest
    let repay_raw = if args.amount == 0.0 {
        // Full repay: use current_debt (not uint256.max, which can revert)
        current_debt
    } else {
        (args.amount * 1e18) as u128
    };

    eprintln!("Market: {} (index {})", market_name, market_idx);
    eprintln!("Current debt: {:.6} crvUSD", current_debt as f64 / 1e18);
    eprintln!("Repay amount: {:.6} crvUSD", repay_raw as f64 / 1e18);

    // Build calldatas
    let approve_calldata = calldata_approve(&controller, repay_raw);
    let repay_calldata = calldata_repay(repay_raw);

    if args.dry_run {
        let output = json!({
            "ok": true,
            "dry_run": true,
            "market": market_name,
            "action": "repay",
            "wallet": wallet,
            "current_debt_crvusd": format!("{:.6}", current_debt as f64 / 1e18),
            "repay_amount_crvusd": format!("{:.6}", repay_raw as f64 / 1e18),
            "approve_calldata": approve_calldata,
            "calldata": repay_calldata,
            "crvusd_contract": CRVUSD,
            "note": "Dry-run mode. Ask user to confirm before executing real transaction.",
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    // ── Real execution ─────────────────────────────────────────────────────────
    // Step 1: approve crvUSD for controller
    eprintln!("Step 1: Approving crvUSD for controller...");
    let approve_result = wallet_contract_call(
        args.chain,
        CRVUSD,
        &approve_calldata,
        Some(&wallet),
        None,
        false,
    )
    .await?;
    let approve_hash = extract_tx_hash(&approve_result);
    eprintln!("Approve txHash: {}", approve_hash);

    if !approve_result["ok"].as_bool().unwrap_or(false) {
        anyhow::bail!("crvUSD approve failed: {}", approve_result);
    }

    // Step 2: repay
    eprintln!("Step 2: Repaying crvUSD...");
    let repay_result = wallet_contract_call(
        args.chain,
        &controller,
        &repay_calldata,
        Some(&wallet),
        None,
        false,
    )
    .await?;
    let repay_hash = extract_tx_hash(&repay_result);

    let output = json!({
        "ok": repay_result["ok"].as_bool().unwrap_or(false),
        "market": market_name,
        "action": "repay",
        "wallet": wallet,
        "repay_amount_crvusd": format!("{:.6}", repay_raw as f64 / 1e18),
        "approve_txHash": approve_hash,
        "txHash": repay_hash,
        "result": repay_result,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
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
