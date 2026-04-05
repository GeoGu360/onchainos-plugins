use anyhow::Result;
use clap::Args;
use alloy_sol_types::{sol, SolCall};
use crate::{config, onchainos};

sol! {
    function processExpiredLocks(bool _relock) external;
}

#[derive(Args, Debug)]
pub struct UnlockCvxArgs {
    /// Re-lock the CVX after unlocking instead of withdrawing
    #[arg(long, default_value = "false")]
    pub relock: bool,

    /// Wallet address override
    #[arg(long)]
    pub from: Option<String>,
}

pub async fn run(args: UnlockCvxArgs, chain_id: u64, dry_run: bool) -> Result<()> {
    if dry_run {
        let call = processExpiredLocksCall { _relock: args.relock };
        let calldata = format!("0x{}", hex::encode(call.abi_encode()));
        let output = serde_json::json!({
            "ok": true,
            "dry_run": true,
            "data": { "txHash": "0x0000000000000000000000000000000000000000000000000000000000000000" },
            "calldata": calldata,
            "contract": config::VLCVX,
            "note": "processExpiredLocks will revert if there are no expired locks"
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    let wallet = match &args.from {
        Some(addr) => addr.clone(),
        None => onchainos::resolve_wallet(chain_id)?,
    };

    // Build calldata (ask user to confirm before executing)
    let call = processExpiredLocksCall { _relock: args.relock };
    let calldata = format!("0x{}", hex::encode(call.abi_encode()));

    let result = onchainos::wallet_contract_call(
        chain_id,
        config::VLCVX,
        &calldata,
        Some(&wallet),
        None,
        false,
    ).await?;

    let tx_hash = onchainos::extract_tx_hash(&result);

    let output = serde_json::json!({
        "ok": true,
        "data": {
            "action": "unlock-cvx",
            "wallet": wallet,
            "relock": args.relock,
            "txHash": tx_hash,
            "explorer": format!("https://etherscan.io/tx/{}", tx_hash)
        }
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
