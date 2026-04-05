// src/commands/borrow.rs -- Borrow assets from Benqi (DRY-RUN ONLY)
// borrow(uint256) selector: 0xc5ebeaec
// Requires collateral to be supplied and set as collateral via Comptroller first.
// Benqi uses Compound V2 architecture -- borrow called on qiToken directly.
use anyhow::Result;
use serde_json::{json, Value};

use crate::config::{find_market, CHAIN_ID, to_raw};

pub async fn run(
    chain_id: u64,
    asset: String,
    amount: f64,
    from: Option<String>,
) -> Result<Value> {
    if chain_id != CHAIN_ID {
        anyhow::bail!("Benqi Lending is only supported on Avalanche C-Chain (chain 43114). Got chain {}.", chain_id);
    }

    let market = find_market(&asset)
        .ok_or_else(|| anyhow::anyhow!("Unknown asset '{}'. Supported: AVAX, USDC, USDT, ETH, BTC, LINK, DAI, QI", asset))?;

    let raw_amount = to_raw(amount, market.underlying_decimals);
    if raw_amount == 0 {
        anyhow::bail!("Amount too small to represent in base units.");
    }

    // borrow(uint256 borrowAmount) selector: 0xc5ebeaec
    let calldata = format!("0xc5ebeaec{:064x}", raw_amount);

    let wallet_display = from.unwrap_or_else(|| "<logged-in wallet>".to_string());

    Ok(json!({
        "ok": true,
        "dry_run": true,
        "note": "Borrow is DRY-RUN ONLY for safety. No on-chain transaction is broadcast.",
        "action": format!("borrow {}", asset),
        "qi_token": market.qi_token,
        "wallet": wallet_display,
        "amount": amount,
        "raw_amount": raw_amount.to_string(),
        "steps": [
            {
                "step": 1,
                "action": format!("qiToken.borrow({} {})", amount, asset),
                "to": market.qi_token,
                "calldata": calldata,
                "note": "Requires sufficient collateral supplied to Benqi Comptroller"
            }
        ],
        "prerequisites": [
            "Supply collateral via 'benqi supply --asset <collateral>'",
            "Ensure health factor > 1.0 after borrow"
        ]
    }))
}
