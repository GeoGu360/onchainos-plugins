use anyhow::Result;
use clap::Args;
use alloy_sol_types::sol;
use crate::{config, onchainos, rpc};

sol! {
    /// Claim rewards from CvxCrvStaking
    function getRewardCvxCrv(address _account, bool _claimExtras) external;
}

sol! {
    /// Claim rewards from vlCVX
    function getRewardVlCvx() external;
}

#[derive(Args, Debug)]
pub struct ClaimRewardsArgs {
    /// Claim from cvxCRV staking (default: true)
    #[arg(long, default_value = "true")]
    pub cvxcrv: bool,

    /// Claim from vlCVX (default: true)
    #[arg(long, default_value = "true")]
    pub vlcvx: bool,

    /// Wallet address override
    #[arg(long)]
    pub from: Option<String>,
}

pub async fn run(args: ClaimRewardsArgs, chain_id: u64, dry_run: bool) -> Result<()> {
    if dry_run {
        let mut steps = Vec::new();
        if args.cvxcrv {
            // getReward(address,bool) selector = 0x7050ccd9
            steps.push(serde_json::json!({
                "action": "claim-cvxcrv-rewards",
                "contract": config::CVXCRV_STAKING,
                "calldata": "0x7050ccd9[wallet_padded]0000000000000000000000000000000000000000000000000000000000000001"
            }));
        }
        if args.vlcvx {
            // getReward() selector = 0x3d18b912
            steps.push(serde_json::json!({
                "action": "claim-vlcvx-rewards",
                "contract": config::VLCVX,
                "calldata": "0x3d18b912"
            }));
        }
        let output = serde_json::json!({
            "ok": true,
            "dry_run": true,
            "data": { "txHash": "0x0000000000000000000000000000000000000000000000000000000000000000" },
            "steps": steps
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    let wallet = match &args.from {
        Some(addr) => addr.clone(),
        None => onchainos::resolve_wallet(chain_id)?,
    };

    let mut tx_hashes = Vec::new();

    // Claim cvxCRV staking rewards (ask user to confirm)
    if args.cvxcrv {
        let pending = rpc::cvxcrv_earned(&wallet).await.unwrap_or(0);
        if pending > 0 {
            // getReward(address,bool) where _account=wallet, _claimExtras=true
            let wallet_padded = format!("{:0>64}", wallet.strip_prefix("0x").unwrap_or(&wallet).to_lowercase());
            let calldata = format!("0x7050ccd9{}0000000000000000000000000000000000000000000000000000000000000001", wallet_padded);

            let result = onchainos::wallet_contract_call(
                chain_id,
                config::CVXCRV_STAKING,
                &calldata,
                Some(&wallet),
                None,
                false,
            ).await?;
            let tx_hash = onchainos::extract_tx_hash(&result);
            eprintln!("cvxCRV staking rewards claimed: {}", tx_hash);
            tx_hashes.push(serde_json::json!({
                "source": "cvxCRV_staking",
                "pending_crv": rpc::format_amount(pending, 18),
                "txHash": tx_hash
            }));
        } else {
            tx_hashes.push(serde_json::json!({
                "source": "cvxCRV_staking",
                "pending_crv": "0",
                "note": "No pending rewards to claim"
            }));
        }
    }

    // Claim vlCVX rewards
    if args.vlcvx {
        let vlcvx_bal = rpc::erc20_balance_of(config::VLCVX, &wallet).await.unwrap_or(0);
        if vlcvx_bal > 0 {
            // getReward() — no args
            let calldata = "0x3d18b912";
            let result = onchainos::wallet_contract_call(
                chain_id,
                config::VLCVX,
                calldata,
                Some(&wallet),
                None,
                false,
            ).await?;
            let tx_hash = onchainos::extract_tx_hash(&result);
            eprintln!("vlCVX rewards claimed: {}", tx_hash);
            tx_hashes.push(serde_json::json!({
                "source": "vlCVX",
                "txHash": tx_hash
            }));
        } else {
            tx_hashes.push(serde_json::json!({
                "source": "vlCVX",
                "note": "No vlCVX balance — no rewards to claim"
            }));
        }
    }

    let output = serde_json::json!({
        "ok": true,
        "data": {
            "action": "claim-rewards",
            "wallet": wallet,
            "claims": tx_hashes
        }
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
