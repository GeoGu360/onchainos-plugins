// src/commands/supply.rs — Supply assets to LayerBank to earn interest (mints lTokens)
//
// LayerBank Core.supply(address lToken, uint256 uAmount) external payable
// Selector: 0xf2b9fdb8
//
// For ETH: pass uAmount=0, attach msg.value; for ERC-20: approve Core first.
use anyhow::Result;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

use crate::config::{find_market, CORE, RPC_URL, to_raw};
use crate::onchainos::{resolve_wallet, wallet_contract_call, erc20_approve, extract_tx_hash};
use crate::rpc::ltoken_balance_of;

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

    // supply(address lToken, uint256 uAmount) selector = 0xf2b9fdb8
    let ltoken_padded = format!("{:0>64}", market.ltoken.trim_start_matches("0x"));

    if market.is_eth {
        // ETH supply: uAmount is ignored (msg.value used), but we encode it anyway
        let amount_padded = format!("{:064x}", 0u128); // uAmount=0 for ETH path
        let calldata = format!("0xf2b9fdb8{}{}", ltoken_padded, amount_padded);

        if dry_run {
            return Ok(json!({
                "ok": true,
                "dry_run": true,
                "action": "supply ETH to LayerBank",
                "ltoken": market.ltoken,
                "amount_eth": amount,
                "amount_wei": raw_amount.to_string(),
                "calldata": calldata,
                "steps": [
                    {
                        "step": 1,
                        "action": "Core.supply(lETH, 0) payable",
                        "to": CORE,
                        "value_wei": raw_amount.to_string()
                    }
                ]
            }));
        }

        let result = wallet_contract_call(
            chain_id, CORE, &calldata, Some(&wallet), Some(raw_amount), false,
        ).await?;
        let tx_hash = extract_tx_hash(&result);

        let new_ltoken_bal = ltoken_balance_of(market.ltoken, &wallet, RPC_URL).await.unwrap_or(0);
        let new_ltoken_human = (new_ltoken_bal as f64) / 1e18;

        return Ok(json!({
            "ok": true,
            "action": "supply ETH to LayerBank",
            "txHash": tx_hash,
            "amount_eth": amount,
            "amount_wei": raw_amount.to_string(),
            "new_ltoken_balance": format!("{:.8}", new_ltoken_human),
            "ltoken_address": market.ltoken,
            "explorer": format!("https://scrollscan.com/tx/{}", tx_hash)
        }));
    }

    // ERC-20 path: approve Core first, then supply
    let underlying = market.underlying.expect("ERC-20 market must have underlying address");
    let amount_padded = format!("{:064x}", raw_amount);
    let supply_calldata = format!("0xf2b9fdb8{}{}", ltoken_padded, amount_padded);

    if dry_run {
        let approve_calldata = format!(
            "0x095ea7b3{:0>64}{:064x}",
            CORE.trim_start_matches("0x"),
            raw_amount
        );
        return Ok(json!({
            "ok": true,
            "dry_run": true,
            "action": format!("supply {} to LayerBank", asset),
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
                    "action": format!("Core.supply(l{}, {})", asset, raw_amount),
                    "to": CORE,
                    "calldata": supply_calldata
                }
            ]
        }));
    }

    // Step 1: ERC-20 approve Core to spend underlying
    let approve_result = erc20_approve(chain_id, underlying, CORE, raw_amount, Some(&wallet), false).await?;
    let approve_hash = extract_tx_hash(&approve_result);
    eprintln!("[supply] approve txHash: {}", approve_hash);

    // Wait for nonce safety before submitting the supply tx
    sleep(Duration::from_secs(3)).await;

    // Step 2: Core.supply(lToken, amount)
    let supply_result = wallet_contract_call(
        chain_id, CORE, &supply_calldata, Some(&wallet), None, false,
    ).await?;
    let supply_hash = extract_tx_hash(&supply_result);

    let new_ltoken_bal = ltoken_balance_of(market.ltoken, &wallet, RPC_URL).await.unwrap_or(0);
    let new_ltoken_human = (new_ltoken_bal as f64) / 1e18;

    Ok(json!({
        "ok": true,
        "action": format!("supply {} to LayerBank", asset),
        "approveTxHash": approve_hash,
        "supplyTxHash": supply_hash,
        "amount": amount,
        "asset": asset,
        "ltoken": market.ltoken,
        "new_ltoken_balance": format!("{:.8}", new_ltoken_human),
        "explorer_supply": format!("https://scrollscan.com/tx/{}", supply_hash),
        "note": "Ask user to confirm before broadcasting on-chain supply transaction"
    }))
}
