// src/commands/supply.rs — Supply assets to INIT Capital on Blast
//
// Flow: ERC-20 approve(MoneyMarketHook, amount) → MoneyMarketHook.execute(OperationParams)
// MoneyMarketHook.execute selector: 0x247d4981
// ERC-20 approve selector: 0x095ea7b3
use anyhow::Result;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

use crate::abi::{DepositParams, encode_execute};
use crate::config::{find_pool, MONEY_MARKET_HOOK, to_raw};
use crate::onchainos::{resolve_wallet, wallet_contract_call, erc20_approve, extract_tx_hash};

pub async fn run(
    chain_id: u64,
    asset: String,
    amount: f64,
    pos_id: u64,
    from: Option<String>,
    dry_run: bool,
) -> Result<Value> {
    if chain_id != 81457 {
        anyhow::bail!(
            "INIT Capital Blast deployment uses chain 81457. Got chain {}.",
            chain_id
        );
    }

    let pool = find_pool(&asset)
        .ok_or_else(|| anyhow::anyhow!("Unknown asset '{}'. Supported: WETH, USDB", asset))?;

    let raw_amount = to_raw(amount, pool.underlying_decimals);
    if raw_amount == 0 {
        anyhow::bail!("Amount too small to represent in base units.");
    }

    // Resolve wallet after dry-run check (avoid wallet resolution failure in dry-run mode)
    let wallet = if dry_run {
        from.clone().unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string())
    } else {
        match from.clone() {
            Some(w) => w,
            None => resolve_wallet(chain_id)?,
        }
    };

    // Build execute calldata
    let deposit_params = vec![DepositParams {
        pool: pool.pool.to_string(),
        amt: raw_amount,
    }];
    let execute_calldata = encode_execute(
        pos_id,
        &wallet,
        1, // mode = 1 (general)
        &deposit_params,
        &[],
        &[],
        &[],
        0, // minHealth = 0 (no check)
        false, // returnNative = false
    );

    // Approve calldata
    let approve_calldata = format!(
        "0x095ea7b3{:0>64}{:064x}",
        MONEY_MARKET_HOOK.trim_start_matches("0x"),
        raw_amount
    );

    if dry_run {
        return Ok(json!({
            "ok": true,
            "dry_run": true,
            "action": format!("supply {} {} to INIT Capital on Blast", amount, asset),
            "pos_id": pos_id,
            "asset": asset,
            "amount": amount,
            "raw_amount": raw_amount.to_string(),
            "steps": [
                {
                    "step": 1,
                    "action": format!("{}.approve(MoneyMarketHook, {})", asset, raw_amount),
                    "to": pool.underlying,
                    "calldata": approve_calldata
                },
                {
                    "step": 2,
                    "action": "MoneyMarketHook.execute(OperationParams) with deposit",
                    "to": MONEY_MARKET_HOOK,
                    "calldata": execute_calldata
                }
            ],
            "note": "Ask user to confirm before broadcasting on-chain supply transaction"
        }));
    }

    // Step 1: ERC-20 approve MoneyMarketHook
    let approve_result = erc20_approve(
        chain_id, pool.underlying, MONEY_MARKET_HOOK, raw_amount, Some(&wallet), false
    ).await?;
    let approve_hash = extract_tx_hash(&approve_result);
    eprintln!("[supply] approve txHash: {}", approve_hash);

    // Wait for nonce safety before submitting execute
    sleep(Duration::from_secs(5)).await;

    // Step 2: MoneyMarketHook.execute()
    let execute_result = wallet_contract_call(
        chain_id, MONEY_MARKET_HOOK, &execute_calldata, Some(&wallet), None, false,
    ).await?;
    let execute_hash = extract_tx_hash(&execute_result);

    Ok(json!({
        "ok": true,
        "action": format!("supply {} {} to INIT Capital on Blast", amount, asset),
        "pos_id": pos_id,
        "asset": asset,
        "amount": amount,
        "raw_amount": raw_amount.to_string(),
        "approveTxHash": approve_hash,
        "executeTxHash": execute_hash,
        "explorer": format!("https://blastscan.io/tx/{}", execute_hash),
        "note": "Ask user to confirm before broadcasting on-chain supply transaction"
    }))
}
