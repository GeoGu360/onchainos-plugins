// src/commands/redeem.rs -- Redeem qiTokens to get back underlying assets
// Uses redeemUnderlying(uint256) to redeem by underlying amount
// selector: 0x852a12e3
use anyhow::Result;
use serde_json::{json, Value};

use crate::config::{find_market, CHAIN_ID, to_raw};
use crate::onchainos::{resolve_wallet, wallet_contract_call, extract_tx_hash};

pub async fn run(
    chain_id: u64,
    asset: String,
    amount: f64,
    from: Option<String>,
    dry_run: bool,
) -> Result<Value> {
    if chain_id != CHAIN_ID {
        anyhow::bail!("Benqi Lending is only supported on Avalanche C-Chain (chain 43114). Got chain {}.", chain_id);
    }

    let market = find_market(&asset)
        .ok_or_else(|| anyhow::anyhow!("Unknown asset '{}'. Supported: AVAX, USDC, USDT, ETH, BTC, LINK, DAI, QI", asset))?;

    let wallet = match from {
        Some(ref w) => w.clone(),
        None => {
            if dry_run {
                "0x0000000000000000000000000000000000000000".to_string()
            } else {
                resolve_wallet(chain_id)?
            }
        }
    };

    let raw_amount = to_raw(amount, market.underlying_decimals);
    if raw_amount == 0 {
        anyhow::bail!("Amount too small to represent in base units.");
    }

    // redeemUnderlying(uint256 redeemAmount) selector: 0x852a12e3
    let calldata = format!("0x852a12e3{:064x}", raw_amount);

    if dry_run {
        return Ok(json!({
            "ok": true,
            "dry_run": true,
            "action": format!("redeem {}", asset),
            "qi_token": market.qi_token,
            "amount": amount,
            "raw_amount": raw_amount.to_string(),
            "steps": [
                {
                    "step": 1,
                    "action": format!("qiToken.redeemUnderlying({} {})", amount, asset),
                    "to": market.qi_token,
                    "calldata": calldata
                }
            ]
        }));
    }

    let result = wallet_contract_call(chain_id, market.qi_token, &calldata, Some(&wallet), None, false).await?;
    let tx_hash = extract_tx_hash(&result);

    Ok(json!({
        "ok": true,
        "action": format!("redeem {}", asset),
        "txHash": tx_hash,
        "amount": amount,
        "raw_amount": raw_amount.to_string(),
        "asset": asset,
        "qi_token": market.qi_token
    }))
}
