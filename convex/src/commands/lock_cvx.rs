use anyhow::Result;
use clap::Args;
use alloy_sol_types::{sol, SolCall};
use crate::{config, onchainos, rpc};

sol! {
    function lock(uint256 _amount, uint256 _spendRatio) external;
}

#[derive(Args, Debug)]
pub struct LockCvxArgs {
    /// Amount of CVX to lock as vlCVX (in token units, e.g. 10.0)
    #[arg(long)]
    pub amount: f64,

    /// Wallet address override
    #[arg(long)]
    pub from: Option<String>,
}

pub async fn run(args: LockCvxArgs, chain_id: u64, dry_run: bool) -> Result<()> {
    let amount_raw = (args.amount * 1e18) as u128;
    if amount_raw == 0 {
        anyhow::bail!("Amount must be greater than 0");
    }

    if dry_run {
        let calldata_approve = format!(
            "0x095ea7b3{}{:064x}",
            format!("{:0>64}", config::VLCVX.strip_prefix("0x").unwrap_or(config::VLCVX).to_lowercase()),
            u128::MAX
        );
        let call = lockCall {
            _amount: alloy_primitives::U256::from(amount_raw),
            _spendRatio: alloy_primitives::U256::ZERO,
        };
        let calldata_lock = format!("0x{}", hex::encode(call.abi_encode()));
        let output = serde_json::json!({
            "ok": true,
            "dry_run": true,
            "data": { "txHash": "0x0000000000000000000000000000000000000000000000000000000000000000" },
            "steps": [
                {"action": "approve", "token": config::CVX_TOKEN, "spender": config::VLCVX, "calldata": calldata_approve},
                {"action": "lock", "contract": config::VLCVX, "amount": args.amount, "calldata": calldata_lock,
                 "note": "CVX will be locked for 16 weeks (vlCVX)"}
            ]
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    let wallet = match &args.from {
        Some(addr) => addr.clone(),
        None => onchainos::resolve_wallet(chain_id)?,
    };
    let from_ref: Option<&str> = Some(&wallet);

    // Check CVX balance
    let balance = rpc::erc20_balance_of(config::CVX_TOKEN, &wallet).await?;
    if balance < amount_raw {
        anyhow::bail!(
            "Insufficient CVX balance. Have: {}, Need: {}",
            rpc::format_amount(balance, 18),
            args.amount
        );
    }

    // Check allowance
    let allowance = rpc::erc20_allowance(config::CVX_TOKEN, &wallet, config::VLCVX).await.unwrap_or(0);

    let mut approve_tx = None;
    if allowance < amount_raw {
        eprintln!("Approving CVX for vlCVX locking (ask user to confirm)...");
        let approve_result = onchainos::erc20_approve(
            chain_id,
            config::CVX_TOKEN,
            config::VLCVX,
            u128::MAX,
            from_ref,
            false,
        ).await?;
        let approve_hash = onchainos::extract_tx_hash(&approve_result);
        approve_tx = Some(approve_hash.clone());
        eprintln!("Approve tx: {}", approve_hash);
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
    }

    // Lock CVX as vlCVX (ask user to confirm — 16-week lock period!)
    let call = lockCall {
        _amount: alloy_primitives::U256::from(amount_raw),
        _spendRatio: alloy_primitives::U256::ZERO,
    };
    let calldata = format!("0x{}", hex::encode(call.abi_encode()));

    let result = onchainos::wallet_contract_call(
        chain_id,
        config::VLCVX,
        &calldata,
        from_ref,
        None,
        false,
    ).await?;

    let tx_hash = onchainos::extract_tx_hash(&result);

    let output = serde_json::json!({
        "ok": true,
        "data": {
            "action": "lock-cvx",
            "amount": args.amount,
            "wallet": wallet,
            "lock_period": "16 weeks",
            "approve_txHash": approve_tx,
            "txHash": tx_hash,
            "explorer": format!("https://etherscan.io/tx/{}", tx_hash)
        }
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
