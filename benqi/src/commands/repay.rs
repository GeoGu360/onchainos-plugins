// src/commands/repay.rs -- Repay borrowed assets on Benqi (DRY-RUN ONLY)
// For AVAX: repayBorrow() payable, selector 0x4e4d9fea
// For ERC20: approve(qiToken, amount) + repayBorrow(uint256), selector 0x0e752702
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

    let wallet_display = from.unwrap_or_else(|| "<logged-in wallet>".to_string());

    if market.is_native {
        // repayBorrow() payable for AVAX - selector: 0x4e4d9fea
        let calldata = "0x4e4d9fea".to_string();
        return Ok(json!({
            "ok": true,
            "dry_run": true,
            "note": "Repay is DRY-RUN ONLY for safety. No on-chain transaction is broadcast.",
            "action": "repay AVAX",
            "qi_token": market.qi_token,
            "wallet": wallet_display,
            "amount": amount,
            "amount_wei": raw_amount.to_string(),
            "steps": [
                {
                    "step": 1,
                    "action": format!("qiAVAX.repayBorrow() payable with {} AVAX", amount),
                    "to": market.qi_token,
                    "calldata": calldata,
                    "value_wei": raw_amount.to_string()
                }
            ]
        }));
    }

    // ERC20 repay: approve + repayBorrow(uint256)
    // repayBorrow(uint256) selector: 0x0e752702
    let underlying = market.underlying.expect("ERC20 must have underlying");
    let approve_calldata = format!(
        "0x095ea7b3{:0>64}{:064x}",
        &market.qi_token[2..],
        raw_amount
    );
    let repay_calldata = format!("0x0e752702{:064x}", raw_amount);

    Ok(json!({
        "ok": true,
        "dry_run": true,
        "note": "Repay is DRY-RUN ONLY for safety. No on-chain transaction is broadcast.",
        "action": format!("repay {}", asset),
        "qi_token": market.qi_token,
        "wallet": wallet_display,
        "amount": amount,
        "raw_amount": raw_amount.to_string(),
        "steps": [
            {
                "step": 1,
                "action": format!("{}.approve(qiToken, amount)", asset),
                "to": underlying,
                "calldata": approve_calldata
            },
            {
                "step": 2,
                "action": format!("qiToken.repayBorrow({} {})", amount, asset),
                "to": market.qi_token,
                "calldata": repay_calldata
            }
        ],
        "tip": "Use wallet balance as repay amount to avoid reverting due to accrued interest"
    }))
}
