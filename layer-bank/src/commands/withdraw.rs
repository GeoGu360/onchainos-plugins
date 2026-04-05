// src/commands/withdraw.rs — Redeem lTokens to get back underlying asset
//
// LayerBank Core.redeemUnderlying(address lToken, uint256 uAmount) external
// Selector: 0x96294178
//
// No approval needed — Core burns lTokens directly from the caller.
use anyhow::Result;
use serde_json::{json, Value};

use crate::config::{find_market, CORE, to_raw};
use crate::onchainos::{resolve_wallet, wallet_contract_call, extract_tx_hash};

pub async fn run(
    chain_id: u64,
    asset: String,
    amount: f64,
    from: Option<String>,
    dry_run: bool,
) -> Result<Value> {
    if chain_id != 534352 {
        anyhow::bail!(
            "LayerBank is deployed on Scroll (chain 534352). Got chain {}.",
            chain_id
        );
    }

    let market = find_market(&asset)
        .ok_or_else(|| anyhow::anyhow!("Unknown asset '{}'. Supported: ETH, USDC, USDT, wstETH, WBTC", asset))?;

    // Resolve wallet after dry-run guard
    let wallet = if dry_run {
        from.clone().unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string())
    } else {
        match from.clone() {
            Some(w) => w,
            None => resolve_wallet(chain_id)?,
        }
    };

    let raw_amount = to_raw(amount, market.underlying_decimals);
    if raw_amount == 0 {
        anyhow::bail!("Amount too small to represent in base units.");
    }

    // redeemUnderlying(address lToken, uint256 uAmount) selector = 0x96294178
    let ltoken_padded = format!("{:0>64}", market.ltoken.trim_start_matches("0x"));
    let amount_padded = format!("{:064x}", raw_amount);
    let calldata = format!("0x96294178{}{}", ltoken_padded, amount_padded);

    if dry_run {
        return Ok(json!({
            "ok": true,
            "dry_run": true,
            "action": format!("withdraw {} from LayerBank", asset),
            "ltoken": market.ltoken,
            "amount": amount,
            "raw_amount": raw_amount.to_string(),
            "calldata": calldata,
            "steps": [
                {
                    "step": 1,
                    "action": format!("Core.redeemUnderlying(l{}, {})", asset, raw_amount),
                    "to": CORE,
                    "calldata": calldata.clone()
                }
            ],
            "note": "Ask user to confirm before broadcasting. Requires no outstanding borrows or sufficient collateral after withdrawal."
        }));
    }

    let result = wallet_contract_call(
        chain_id, CORE, &calldata, Some(&wallet), None, false,
    ).await?;
    let tx_hash = extract_tx_hash(&result);

    Ok(json!({
        "ok": true,
        "action": format!("withdraw {} from LayerBank", asset),
        "txHash": tx_hash,
        "amount": amount,
        "asset": asset,
        "ltoken": market.ltoken,
        "explorer": format!("https://scrollscan.com/tx/{}", tx_hash),
        "note": "Ask user to confirm before broadcasting on-chain withdraw transaction"
    }))
}
