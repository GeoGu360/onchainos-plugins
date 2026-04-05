use anyhow::Context;
use serde_json::{json, Value};

use crate::calldata;
use crate::config::get_chain_config;
use crate::onchainos;
use crate::rpc;

/// Withdraw supplied assets from the Aave V2 LendingPool.
///
/// Calls LendingPool.withdraw(asset, amount, to).
/// Selector: 0x69328dec
///
/// For full withdrawal, pass --all flag which sends uint256.max.
pub async fn run(
    chain_id: u64,
    asset: &str,
    amount: Option<f64>,
    all: bool,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<Value> {
    let cfg = get_chain_config(chain_id)?;

    if amount.is_none() && !all {
        anyhow::bail!("Must specify --amount or --all for withdraw");
    }

    let from_addr = if let Some(addr) = from {
        addr.to_string()
    } else {
        onchainos::resolve_wallet(chain_id, dry_run)
            .context("No --from address and could not resolve active wallet")?
    };

    let (token_addr, decimals) = onchainos::resolve_token(asset, chain_id)
        .with_context(|| format!("Could not resolve token address for '{}'", asset))?;

    let amount_minimal: u128 = if all {
        u128::MAX
    } else {
        let amt = amount.unwrap();
        let factor = 10u128.pow(decimals as u32);
        (amt * factor as f64) as u128
    };

    let pool_addr = rpc::get_lending_pool(cfg.lending_pool_addresses_provider, cfg.rpc_url)
        .await
        .unwrap_or_else(|_| cfg.lending_pool_proxy.to_string());

    let withdraw_calldata = calldata::encode_withdraw(&token_addr, amount_minimal, &from_addr)
        .context("Failed to encode withdraw calldata")?;

    if dry_run {
        let cmd = format!(
            "onchainos wallet contract-call --chain {} --to {} --input-data {} --from {} --dry-run --force",
            chain_id, pool_addr, withdraw_calldata, from_addr
        );
        eprintln!("[dry-run] withdraw: {}", cmd);
        return Ok(json!({
            "ok": true,
            "dryRun": true,
            "asset": asset,
            "tokenAddress": token_addr,
            "amount": if all { "all".to_string() } else { amount.unwrap().to_string() },
            "lendingPool": pool_addr,
            "simulatedCommand": cmd
        }));
    }

    let result = onchainos::wallet_contract_call(
        chain_id,
        &pool_addr,
        &withdraw_calldata,
        Some(&from_addr),
        false,
    )
    .context("LendingPool.withdraw() failed")?;

    let tx_hash = result["data"]["txHash"]
        .as_str()
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("pending")
        .to_string();

    Ok(json!({
        "ok": true,
        "asset": asset,
        "tokenAddress": token_addr,
        "amount": if all { "all".to_string() } else { amount.unwrap().to_string() },
        "lendingPool": pool_addr,
        "withdrawTxHash": tx_hash,
        "dryRun": false
    }))
}
