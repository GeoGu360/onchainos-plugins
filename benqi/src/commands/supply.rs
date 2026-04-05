// src/commands/supply.rs -- Supply assets to Benqi (mint qiTokens)
// For AVAX: mint() payable (selector 0x1249c58b), send AVAX as value
// For ERC20: approve(qiToken, amount) + mint(uint256) (selector 0xa0712d68)
use anyhow::Result;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

use crate::config::{find_market, RPC_URL, CHAIN_ID, to_raw};
use crate::onchainos::{resolve_wallet, wallet_contract_call, erc20_approve, extract_tx_hash};
use crate::rpc::balance_of;

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

    if market.is_native {
        // qiAVAX: mint() payable - send AVAX as value
        // selector: 0x1249c58b (mint())
        let calldata = "0x1249c58b".to_string();

        if dry_run {
            return Ok(json!({
                "ok": true,
                "dry_run": true,
                "action": "supply AVAX",
                "qi_token": market.qi_token,
                "amount_avax": amount,
                "amount_wei": raw_amount.to_string(),
                "calldata": calldata,
                "steps": [
                    {
                        "step": 1,
                        "action": "qiAVAX.mint() payable",
                        "to": market.qi_token,
                        "value_wei": raw_amount.to_string(),
                        "calldata": calldata
                    }
                ]
            }));
        }

        let result = wallet_contract_call(chain_id, market.qi_token, &calldata, Some(&wallet), Some(raw_amount), false).await?;
        let tx_hash = extract_tx_hash(&result);

        let new_qi_bal = balance_of(market.qi_token, &wallet, RPC_URL).await.unwrap_or(0);
        let new_qi_human = (new_qi_bal as f64) / 1e8;

        return Ok(json!({
            "ok": true,
            "action": "supply AVAX",
            "txHash": tx_hash,
            "amount_avax": amount,
            "amount_wei": raw_amount.to_string(),
            "new_qiAVAX_balance": format!("{:.8}", new_qi_human),
            "qi_token_address": market.qi_token
        }));
    }

    // ERC20 path: approve(qiToken, amount) + mint(uint256)
    let underlying = market.underlying.expect("ERC20 market must have underlying address");

    // mint(uint256) selector: 0xa0712d68
    let mint_calldata = format!("0xa0712d68{:064x}", raw_amount);

    if dry_run {
        let approve_calldata = format!(
            "0x095ea7b3{:0>64}{:064x}",
            &market.qi_token[2..],
            raw_amount
        );
        return Ok(json!({
            "ok": true,
            "dry_run": true,
            "action": format!("supply {}", asset),
            "qi_token": market.qi_token,
            "underlying": underlying,
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
                    "action": "qiToken.mint(amount)",
                    "to": market.qi_token,
                    "calldata": mint_calldata
                }
            ]
        }));
    }

    // Step 1: ERC20 approve
    let approve_result = erc20_approve(chain_id, underlying, market.qi_token, raw_amount, Some(&wallet), false).await?;
    let approve_hash = extract_tx_hash(&approve_result);
    eprintln!("[supply] approve txHash: {}", approve_hash);

    // Wait for nonce safety (3 seconds between approve and mint)
    sleep(Duration::from_secs(3)).await;

    // Step 2: qiToken.mint(uint256)
    let mint_result = wallet_contract_call(chain_id, market.qi_token, &mint_calldata, Some(&wallet), None, false).await?;
    let mint_hash = extract_tx_hash(&mint_result);

    let new_qi_bal = balance_of(market.qi_token, &wallet, RPC_URL).await.unwrap_or(0);
    let new_qi_human = (new_qi_bal as f64) / 1e8;

    Ok(json!({
        "ok": true,
        "action": format!("supply {}", asset),
        "approveTxHash": approve_hash,
        "mintTxHash": mint_hash,
        "amount": amount,
        "raw_amount": raw_amount.to_string(),
        "asset": asset,
        "qi_token": market.qi_token,
        "new_qi_token_balance": format!("{:.8}", new_qi_human)
    }))
}
