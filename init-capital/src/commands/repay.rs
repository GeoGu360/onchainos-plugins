// src/commands/repay.rs — Repay debt in INIT Capital on Blast (DRY-RUN recommended)
//
// Flow: ERC-20 approve(MoneyMarketHook, amount) → MoneyMarketHook.execute with RepayParams
// selector: 0x247d4981
use anyhow::Result;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

use crate::abi::{RepayParams, encode_execute};
use crate::config::{find_pool, MONEY_MARKET_HOOK, POS_MANAGER, RPC_URL, to_raw};
use crate::onchainos::{resolve_wallet, wallet_contract_call, erc20_approve, extract_tx_hash};
use crate::rpc::get_pos_debt_shares;

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

    // Resolve wallet after dry-run guard
    let wallet = if dry_run {
        from.clone().unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string())
    } else {
        match from.clone() {
            Some(w) => w,
            None => resolve_wallet(chain_id)?,
        }
    };

    // Get current debt shares for this position
    let debt_shares = if dry_run {
        raw_amount
    } else {
        match get_pos_debt_shares(POS_MANAGER, pos_id as u128, pool.pool, RPC_URL).await {
            Ok(s) => s.min(raw_amount + 1000), // cap at requested + dust buffer
            Err(_) => raw_amount,
        }
    };

    let repay_params = vec![RepayParams {
        pool: pool.pool.to_string(),
        shares: debt_shares,
    }];

    let approve_calldata = format!(
        "0x095ea7b3{:0>64}{:064x}",
        MONEY_MARKET_HOOK.trim_start_matches("0x"),
        raw_amount
    );

    let calldata = encode_execute(
        pos_id,
        &wallet,
        1,
        &[],
        &[],
        &[],
        &repay_params,
        0,
        false,
    );

    if dry_run {
        return Ok(json!({
            "ok": true,
            "dry_run": true,
            "action": format!("repay {} {} on INIT Capital position {}", amount, asset, pos_id),
            "pos_id": pos_id,
            "asset": asset,
            "amount": amount,
            "raw_amount": raw_amount.to_string(),
            "debt_shares": debt_shares.to_string(),
            "steps": [
                {
                    "step": 1,
                    "action": format!("{}.approve(MoneyMarketHook, {})", asset, raw_amount),
                    "to": pool.underlying,
                    "calldata": approve_calldata
                },
                {
                    "step": 2,
                    "action": "MoneyMarketHook.execute(OperationParams) with repay",
                    "to": MONEY_MARKET_HOOK,
                    "calldata": calldata
                }
            ],
            "note": "Ask user to confirm before broadcasting on-chain repay transaction. DRY-RUN recommended for repay."
        }));
    }

    // Step 1: ERC-20 approve MoneyMarketHook
    let approve_result = erc20_approve(
        chain_id, pool.underlying, MONEY_MARKET_HOOK, raw_amount, Some(&wallet), false
    ).await?;
    let approve_hash = extract_tx_hash(&approve_result);
    eprintln!("[repay] approve txHash: {}", approve_hash);

    sleep(Duration::from_secs(5)).await;

    // Step 2: MoneyMarketHook.execute() with repay
    let execute_result = wallet_contract_call(
        chain_id, MONEY_MARKET_HOOK, &calldata, Some(&wallet), None, false,
    ).await?;
    let execute_hash = extract_tx_hash(&execute_result);

    Ok(json!({
        "ok": true,
        "action": format!("repay {} {} on INIT Capital position {}", amount, asset, pos_id),
        "pos_id": pos_id,
        "asset": asset,
        "amount": amount,
        "debt_shares": debt_shares.to_string(),
        "approveTxHash": approve_hash,
        "executeTxHash": execute_hash,
        "explorer": format!("https://blastscan.io/tx/{}", execute_hash),
        "note": "Ask user to confirm before broadcasting on-chain repay transaction."
    }))
}
