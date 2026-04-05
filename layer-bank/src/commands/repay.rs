// src/commands/repay.rs — Repay a borrow position on LayerBank (DRY-RUN ONLY per GUARDRAILS)
//
// LayerBank Core.repayBorrow(address lToken, uint256 amount) external payable
// Selector: 0xabdb5ea8
//
// For ETH repay: send msg.value; for ERC-20: approve Core first.
use anyhow::Result;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

use crate::config::{find_market, CORE, to_raw};
use crate::onchainos::{resolve_wallet, wallet_contract_call, erc20_approve, extract_tx_hash};

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

    // repayBorrow(address lToken, uint256 amount) selector = 0xabdb5ea8
    let ltoken_padded = format!("{:0>64}", market.ltoken.trim_start_matches("0x"));
    let amount_padded = format!("{:064x}", raw_amount);
    let repay_calldata = format!("0xabdb5ea8{}{}", ltoken_padded, amount_padded);

    if market.is_eth {
        if dry_run {
            return Ok(json!({
                "ok": true,
                "dry_run": true,
                "action": "repay ETH borrow on LayerBank",
                "ltoken": market.ltoken,
                "amount_eth": amount,
                "amount_wei": raw_amount.to_string(),
                "calldata": repay_calldata,
                "steps": [
                    {
                        "step": 1,
                        "action": "Core.repayBorrow(lETH, amount) payable",
                        "to": CORE,
                        "value_wei": raw_amount.to_string()
                    }
                ],
                "note": "Ask user to confirm before broadcasting on-chain repay transaction"
            }));
        }

        let result = wallet_contract_call(
            chain_id, CORE, &repay_calldata, Some(&wallet), Some(raw_amount), false,
        ).await?;
        let tx_hash = extract_tx_hash(&result);

        return Ok(json!({
            "ok": true,
            "action": "repay ETH borrow on LayerBank",
            "txHash": tx_hash,
            "amount_eth": amount,
            "ltoken": market.ltoken,
            "explorer": format!("https://scrollscan.com/tx/{}", tx_hash),
            "note": "Ask user to confirm before broadcasting on-chain repay transaction"
        }));
    }

    // ERC-20 path: approve Core first, then repayBorrow
    let underlying = market.underlying.expect("ERC-20 market must have underlying");

    if dry_run {
        let approve_calldata = format!(
            "0x095ea7b3{:0>64}{:064x}",
            CORE.trim_start_matches("0x"),
            raw_amount
        );
        return Ok(json!({
            "ok": true,
            "dry_run": true,
            "action": format!("repay {} borrow on LayerBank", asset),
            "ltoken": market.ltoken,
            "amount": amount,
            "raw_amount": raw_amount.to_string(),
            "steps": [
                {
                    "step": 1,
                    "action": format!("{}.approve(Core, {})", asset, raw_amount),
                    "to": underlying,
                    "calldata": approve_calldata
                },
                {
                    "step": 2,
                    "action": format!("Core.repayBorrow(l{}, {})", asset, raw_amount),
                    "to": CORE,
                    "calldata": repay_calldata
                }
            ],
            "note": "Ask user to confirm before broadcasting on-chain repay transaction"
        }));
    }

    // Step 1: ERC-20 approve
    let approve_result = erc20_approve(chain_id, underlying, CORE, raw_amount, Some(&wallet), false).await?;
    let approve_hash = extract_tx_hash(&approve_result);
    eprintln!("[repay] approve txHash: {}", approve_hash);

    sleep(Duration::from_secs(3)).await;

    // Step 2: repayBorrow
    let repay_result = wallet_contract_call(
        chain_id, CORE, &repay_calldata, Some(&wallet), None, false,
    ).await?;
    let repay_hash = extract_tx_hash(&repay_result);

    Ok(json!({
        "ok": true,
        "action": format!("repay {} borrow on LayerBank", asset),
        "approveTxHash": approve_hash,
        "repayTxHash": repay_hash,
        "amount": amount,
        "asset": asset,
        "ltoken": market.ltoken,
        "explorer_repay": format!("https://scrollscan.com/tx/{}", repay_hash),
        "note": "Ask user to confirm before broadcasting on-chain repay transaction"
    }))
}
