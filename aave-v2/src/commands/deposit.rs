use anyhow::Context;
use serde_json::{json, Value};

use crate::calldata;
use crate::config::get_chain_config;
use crate::onchainos;
use crate::rpc;

/// Deposit (supply) assets to the Aave V2 LendingPool.
///
/// Aave V2 uses `deposit()` not `supply()` — selector 0xe8eda9df.
///
/// Flow:
/// 1. Resolve token address and decimals
/// 2. Resolve LendingPool address via LendingPoolAddressesProvider
/// 3. ERC-20 approve LendingPool for the deposit amount
/// 4. Call LendingPool.deposit(asset, amount, onBehalfOf, referralCode=0)
pub async fn run(
    chain_id: u64,
    asset: &str,
    amount: f64,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<Value> {
    let cfg = get_chain_config(chain_id)?;

    let from_addr = if let Some(addr) = from {
        addr.to_string()
    } else {
        onchainos::resolve_wallet(chain_id, dry_run)
            .context("No --from address and could not resolve active wallet")?
    };

    // Resolve token
    let (token_addr, decimals) = onchainos::resolve_token(asset, chain_id)
        .with_context(|| format!("Could not resolve token address for '{}'", asset))?;

    let amount_minimal = human_to_minimal(amount, decimals as u64);

    // Resolve LendingPool address at runtime
    let pool_addr = rpc::get_lending_pool(cfg.lending_pool_addresses_provider, cfg.rpc_url)
        .await
        .unwrap_or_else(|_| cfg.lending_pool_proxy.to_string());

    if dry_run {
        let approve_calldata = calldata::encode_erc20_approve(&pool_addr, amount_minimal)
            .context("Failed to encode approve calldata")?;
        let deposit_calldata = calldata::encode_deposit(&token_addr, amount_minimal, &from_addr)
            .context("Failed to encode deposit calldata")?;
        let approve_cmd = format!(
            "onchainos wallet contract-call --chain {} --to {} --input-data {} --from {} --dry-run --force",
            chain_id, token_addr, approve_calldata, from_addr
        );
        let deposit_cmd = format!(
            "onchainos wallet contract-call --chain {} --to {} --input-data {} --from {} --dry-run --force",
            chain_id, pool_addr, deposit_calldata, from_addr
        );
        eprintln!("[dry-run] step 1 approve:  {}", approve_cmd);
        eprintln!("[dry-run] step 2 deposit:  {}", deposit_cmd);
        return Ok(json!({
            "ok": true,
            "dryRun": true,
            "asset": asset,
            "tokenAddress": token_addr,
            "amount": amount,
            "amountMinimal": amount_minimal.to_string(),
            "lendingPool": pool_addr,
            "steps": [
                {"step": 1, "action": "approve",  "simulatedCommand": approve_cmd},
                {"step": 2, "action": "deposit",  "simulatedCommand": deposit_cmd}
            ]
        }));
    }

    // Step 1: ERC-20 approve
    let approve_calldata = calldata::encode_erc20_approve(&pool_addr, amount_minimal)
        .context("Failed to encode approve calldata")?;
    let approve_result = onchainos::wallet_contract_call(
        chain_id,
        &token_addr,
        &approve_calldata,
        Some(&from_addr),
        false,
    )
    .context("ERC-20 approve failed")?;

    let approve_tx = approve_result["data"]["txHash"]
        .as_str()
        .or_else(|| approve_result["txHash"].as_str())
        .unwrap_or("pending")
        .to_string();

    // Wait for approve to be mined before deposit
    if approve_tx.starts_with("0x") && approve_tx.len() > 10 {
        let confirmed = rpc::wait_for_tx(cfg.rpc_url, &approve_tx)
            .await
            .context("Approve tx did not confirm in time")?;
        if !confirmed {
            anyhow::bail!("Approve tx reverted: {}", approve_tx);
        }
    }

    // Step 2: deposit
    let deposit_calldata = calldata::encode_deposit(&token_addr, amount_minimal, &from_addr)
        .context("Failed to encode deposit calldata")?;
    let deposit_result = onchainos::wallet_contract_call(
        chain_id,
        &pool_addr,
        &deposit_calldata,
        Some(&from_addr),
        false,
    )
    .context("LendingPool.deposit() failed")?;

    let deposit_tx = deposit_result["data"]["txHash"]
        .as_str()
        .or_else(|| deposit_result["txHash"].as_str())
        .unwrap_or("pending")
        .to_string();

    Ok(json!({
        "ok": true,
        "asset": asset,
        "tokenAddress": token_addr,
        "amount": amount,
        "amountMinimal": amount_minimal.to_string(),
        "lendingPool": pool_addr,
        "approveTxHash": approve_tx,
        "depositTxHash": deposit_tx,
        "dryRun": false
    }))
}

pub fn human_to_minimal(amount: f64, decimals: u64) -> u128 {
    let factor = 10u128.pow(decimals as u32);
    (amount * factor as f64) as u128
}
