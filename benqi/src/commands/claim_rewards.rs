// src/commands/claim_rewards.rs -- Claim QI and AVAX rewards from Benqi Comptroller
// claimReward(uint8 rewardType, address holder) selector: 0x0952c563
// rewardType: 0 = QI token, 1 = AVAX
use anyhow::Result;
use serde_json::{json, Value};

use crate::config::{COMPTROLLER, CHAIN_ID, QI_TOKEN};
use crate::onchainos::{resolve_wallet, wallet_contract_call, extract_tx_hash};

pub async fn run(
    chain_id: u64,
    reward_type: u8,
    from: Option<String>,
    dry_run: bool,
) -> Result<Value> {
    if chain_id != CHAIN_ID {
        anyhow::bail!("Benqi Lending is only supported on Avalanche C-Chain (chain 43114). Got chain {}.", chain_id);
    }

    if reward_type > 1 {
        anyhow::bail!("Invalid reward type {}. Use 0 for QI token or 1 for AVAX.", reward_type);
    }

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

    let reward_name = if reward_type == 0 { "QI" } else { "AVAX" };

    // claimReward(uint8 rewardType, address holder) selector: 0x0952c563
    let calldata = format!(
        "0x0952c563{:064x}{}",
        reward_type as u128,
        format!("{:0>64}", &wallet[2..])
    );

    if dry_run {
        return Ok(json!({
            "ok": true,
            "dry_run": true,
            "action": format!("claim {} rewards", reward_name),
            "comptroller": COMPTROLLER,
            "wallet": wallet,
            "reward_type": reward_type,
            "reward_token": if reward_type == 0 { QI_TOKEN } else { "native AVAX" },
            "calldata": calldata,
            "steps": [
                {
                    "step": 1,
                    "action": format!("Comptroller.claimReward({}, wallet)", reward_type),
                    "to": COMPTROLLER,
                    "calldata": calldata.clone()
                }
            ]
        }));
    }

    let result = wallet_contract_call(chain_id, COMPTROLLER, &calldata, Some(&wallet), None, false).await?;
    let tx_hash = extract_tx_hash(&result);

    Ok(json!({
        "ok": true,
        "action": format!("claim {} rewards", reward_name),
        "txHash": tx_hash,
        "comptroller": COMPTROLLER,
        "wallet": wallet,
        "reward_type": reward_type,
        "reward_name": reward_name,
        "reward_token": if reward_type == 0 { QI_TOKEN } else { "native AVAX" }
    }))
}
