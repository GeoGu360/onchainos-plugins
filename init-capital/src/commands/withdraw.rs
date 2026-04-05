// src/commands/withdraw.rs — Withdraw collateral from INIT Capital on Blast
//
// MoneyMarketHook.execute with WithdrawParams
// selector: 0x247d4981
use anyhow::Result;
use serde_json::{json, Value};

use crate::abi::{WithdrawParams, encode_execute};
use crate::config::{find_pool, MONEY_MARKET_HOOK, RPC_URL, to_raw};
use crate::onchainos::{resolve_wallet, wallet_contract_call, extract_tx_hash};
use crate::rpc::to_shares;

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

    // Resolve wallet
    let wallet = if dry_run {
        from.clone().unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string())
    } else {
        match from.clone() {
            Some(w) => w,
            None => resolve_wallet(chain_id)?,
        }
    };

    let recipient = to.as_deref().unwrap_or(&wallet).to_string();

    // Convert token amount to inToken shares
    let shares = if dry_run {
        raw_amount // dry-run: use raw amount as approximate shares
    } else {
        to_shares(pool.pool, raw_amount, RPC_URL).await.unwrap_or(raw_amount)
    };

    let withdraw_params = vec![WithdrawParams {
        pool: pool.pool.to_string(),
        shares,
        to: recipient.clone(),
    }];

    let calldata = encode_execute(
        pos_id,
        &wallet,
        1, // mode = 1 (general)
        &[],
        &withdraw_params,
        &[],
        &[],
        0, // minHealth = 0
        false,
    );

    if dry_run {
        return Ok(json!({
            "ok": true,
            "dry_run": true,
            "action": format!("withdraw {} {} from INIT Capital position {}", amount, asset, pos_id),
            "pos_id": pos_id,
            "asset": asset,
            "amount": amount,
            "shares": shares.to_string(),
            "to": recipient,
            "steps": [
                {
                    "step": 1,
                    "action": "MoneyMarketHook.execute(OperationParams) with withdraw",
                    "to": MONEY_MARKET_HOOK,
                    "calldata": calldata
                }
            ],
            "note": "Ask user to confirm before broadcasting on-chain withdraw transaction. Ensure health factor stays above 1.0 after withdrawal."
        }));
    }

    let result = wallet_contract_call(
        chain_id, MONEY_MARKET_HOOK, &calldata, Some(&wallet), None, false,
    ).await?;
    let tx_hash = extract_tx_hash(&result);

    Ok(json!({
        "ok": true,
        "action": format!("withdraw {} {} from INIT Capital position {}", amount, asset, pos_id),
        "pos_id": pos_id,
        "asset": asset,
        "amount": amount,
        "shares": shares.to_string(),
        "to": recipient,
        "txHash": tx_hash,
        "explorer": format!("https://blastscan.io/tx/{}", tx_hash),
        "note": "Ask user to confirm before broadcasting on-chain withdraw transaction. Ensure health factor stays above 1.0 after withdrawal."
    }))
}
