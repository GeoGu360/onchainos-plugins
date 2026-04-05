// src/commands/borrow.rs — Borrow from INIT Capital on Blast (DRY-RUN recommended)
//
// MoneyMarketHook.execute with BorrowParams
// selector: 0x247d4981
use anyhow::Result;
use serde_json::{json, Value};

use crate::abi::{BorrowParams, encode_execute};
use crate::config::{find_pool, MONEY_MARKET_HOOK, to_raw};
use crate::onchainos::{resolve_wallet, wallet_contract_call, extract_tx_hash};

pub async fn run(
    chain_id: u64,
    asset: String,
    amount: f64,
    pos_id: u64,
    to: Option<String>,
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

    let recipient = to.as_deref().unwrap_or(&wallet).to_string();

    let borrow_params = vec![BorrowParams {
        pool: pool.pool.to_string(),
        amt: raw_amount,
        to: recipient.clone(),
    }];

    // Set a conservative min health to prevent over-borrowing
    let min_health_e18: u128 = 1_100_000_000_000_000_000u128; // 1.1

    let calldata = encode_execute(
        pos_id,
        &wallet,
        1,
        &[],
        &[],
        &borrow_params,
        &[],
        min_health_e18,
        false,
    );

    if dry_run {
        return Ok(json!({
            "ok": true,
            "dry_run": true,
            "action": format!("borrow {} {} from INIT Capital position {}", amount, asset, pos_id),
            "pos_id": pos_id,
            "asset": asset,
            "amount": amount,
            "raw_amount": raw_amount.to_string(),
            "recipient": recipient,
            "min_health_after": "1.1",
            "steps": [
                {
                    "step": 1,
                    "action": "MoneyMarketHook.execute(OperationParams) with borrow",
                    "to": MONEY_MARKET_HOOK,
                    "calldata": calldata
                }
            ],
            "warning": "Borrowing creates liquidation risk. Ensure health factor stays above 1.0.",
            "note": "Ask user to confirm before broadcasting on-chain borrow transaction. Dry-run recommended."
        }));
    }

    let result = wallet_contract_call(
        chain_id, MONEY_MARKET_HOOK, &calldata, Some(&wallet), None, false,
    ).await?;
    let tx_hash = extract_tx_hash(&result);

    Ok(json!({
        "ok": true,
        "action": format!("borrow {} {} from INIT Capital position {}", amount, asset, pos_id),
        "pos_id": pos_id,
        "asset": asset,
        "amount": amount,
        "recipient": recipient,
        "txHash": tx_hash,
        "explorer": format!("https://blastscan.io/tx/{}", tx_hash),
        "warning": "Borrowing creates liquidation risk. Monitor health factor.",
        "note": "Ask user to confirm before broadcasting on-chain borrow transaction."
    }))
}
