use anyhow::Result;
use clap::Args;
use alloy_sol_types::{sol, SolCall};
use crate::{config, onchainos, rpc};

sol! {
    function withdraw(uint256 amount, address _to, bool claim) external;
}

#[derive(Args, Debug)]
pub struct UnstakeCvxCrvArgs {
    /// Amount of cvxCRV to unstake (in token units, e.g. 1.5)
    #[arg(long)]
    pub amount: f64,

    /// Recipient address (defaults to calling wallet)
    #[arg(long)]
    pub to: Option<String>,

    /// Wallet address override
    #[arg(long)]
    pub from: Option<String>,

    /// Also claim pending rewards when withdrawing
    #[arg(long, default_value = "false")]
    pub claim: bool,
}

pub async fn run(args: UnstakeCvxCrvArgs, chain_id: u64, dry_run: bool) -> Result<()> {
    let amount_raw = (args.amount * 1e18) as u128;
    if amount_raw == 0 {
        anyhow::bail!("Amount must be greater than 0");
    }

    if dry_run {
        let call = withdrawCall {
            amount: alloy_primitives::U256::from(amount_raw),
            _to: alloy_primitives::Address::ZERO,
            claim: args.claim,
        };
        let calldata = format!("0x{}", hex::encode(call.abi_encode()));
        let output = serde_json::json!({
            "ok": true,
            "dry_run": true,
            "data": { "txHash": "0x0000000000000000000000000000000000000000000000000000000000000000" },
            "calldata": calldata,
            "contract": config::CVXCRV_STAKING
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    let wallet = match &args.from {
        Some(addr) => addr.clone(),
        None => onchainos::resolve_wallet(chain_id)?,
    };
    let recipient = args.to.clone().unwrap_or_else(|| wallet.clone());

    // Check staked balance
    let staked = rpc::erc20_balance_of(config::CVXCRV_STAKING, &wallet).await?;
    if staked < amount_raw {
        anyhow::bail!(
            "Insufficient staked cvxCRV. Staked: {}, Requested: {}",
            rpc::format_amount(staked, 18),
            args.amount
        );
    }

    let recipient_addr: alloy_primitives::Address = recipient.parse()
        .map_err(|_| anyhow::anyhow!("Invalid recipient address: {}", recipient))?;

    // Build calldata (ask user to confirm before executing)
    let call = withdrawCall {
        amount: alloy_primitives::U256::from(amount_raw),
        _to: recipient_addr,
        claim: args.claim,
    };
    let calldata = format!("0x{}", hex::encode(call.abi_encode()));

    let result = onchainos::wallet_contract_call(
        chain_id,
        config::CVXCRV_STAKING,
        &calldata,
        Some(&wallet),
        None,
        false,
    ).await?;

    let tx_hash = onchainos::extract_tx_hash(&result);

    let output = serde_json::json!({
        "ok": true,
        "data": {
            "action": "unstake-cvxcrv",
            "amount": args.amount,
            "wallet": wallet,
            "recipient": recipient,
            "claimed_rewards": args.claim,
            "txHash": tx_hash,
            "explorer": format!("https://etherscan.io/tx/{}", tx_hash)
        }
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
