use anyhow::Result;
use clap::Args;
use alloy_sol_types::{sol, SolCall};
use crate::{config, onchainos, rpc};

sol! {
    function stake(uint256 _amount) external;
}

#[derive(Args, Debug)]
pub struct StakeCvxCrvArgs {
    /// Amount of cvxCRV to stake (in token units, e.g. 1.5)
    #[arg(long)]
    pub amount: f64,

    /// Wallet address override
    #[arg(long)]
    pub from: Option<String>,
}

pub async fn run(args: StakeCvxCrvArgs, chain_id: u64, dry_run: bool) -> Result<()> {
    // Convert human-readable amount to raw u128 (18 decimals)
    let amount_raw = (args.amount * 1e18) as u128;
    if amount_raw == 0 {
        anyhow::bail!("Amount must be greater than 0");
    }

    // dry_run guard: resolve wallet after this point only
    if dry_run {
        let calldata_approve = format!("0x095ea7b3{}{:064x}",
            format!("{:0>64}", config::CVXCRV_STAKING.strip_prefix("0x").unwrap_or(config::CVXCRV_STAKING).to_lowercase()),
            u128::MAX
        );
        let call = stakeCall { _amount: alloy_primitives::U256::from(amount_raw) };
        let calldata_stake = format!("0x{}", hex::encode(call.abi_encode()));
        let output = serde_json::json!({
            "ok": true,
            "dry_run": true,
            "data": {
                "txHash": "0x0000000000000000000000000000000000000000000000000000000000000000"
            },
            "steps": [
                {"action": "approve", "token": config::CVXCRV_TOKEN, "spender": config::CVXCRV_STAKING, "calldata": calldata_approve},
                {"action": "stake", "contract": config::CVXCRV_STAKING, "amount": args.amount, "calldata": calldata_stake}
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

    // Check cvxCRV balance
    let balance = rpc::erc20_balance_of(config::CVXCRV_TOKEN, &wallet).await?;
    if balance < amount_raw {
        anyhow::bail!(
            "Insufficient cvxCRV balance. Have: {}, Need: {}",
            rpc::format_amount(balance, 18),
            args.amount
        );
    }

    // Check allowance
    let allowance = rpc::erc20_allowance(config::CVXCRV_TOKEN, &wallet, config::CVXCRV_STAKING).await.unwrap_or(0);

    let mut approve_tx = None;
    if allowance < amount_raw {
        // Approve unlimited
        eprintln!("Approving cvxCRV for staking (ask user to confirm)...");
        let approve_result = onchainos::erc20_approve(
            chain_id,
            config::CVXCRV_TOKEN,
            config::CVXCRV_STAKING,
            u128::MAX,
            from_ref,
            false,
        ).await?;
        let approve_hash = onchainos::extract_tx_hash(&approve_result);
        approve_tx = Some(approve_hash.clone());
        eprintln!("Approve tx: {}", approve_hash);
        // Wait for approval to propagate
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
    }

    // Stake cvxCRV (ask user to confirm)
    let call = stakeCall { _amount: alloy_primitives::U256::from(amount_raw) };
    let calldata = format!("0x{}", hex::encode(call.abi_encode()));

    let result = onchainos::wallet_contract_call(
        chain_id,
        config::CVXCRV_STAKING,
        &calldata,
        from_ref,
        None,
        false,
    ).await?;

    let tx_hash = onchainos::extract_tx_hash(&result);

    let output = serde_json::json!({
        "ok": true,
        "data": {
            "action": "stake-cvxcrv",
            "amount": args.amount,
            "wallet": wallet,
            "approve_txHash": approve_tx,
            "txHash": tx_hash,
            "explorer": format!("https://etherscan.io/tx/{}", tx_hash)
        }
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
