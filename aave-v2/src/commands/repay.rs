use anyhow::Context;
use serde_json::{json, Value};

use crate::calldata;
use crate::config::{get_chain_config, INTEREST_RATE_MODE_VARIABLE};
use crate::onchainos;
use crate::rpc;

/// Repay borrowed debt on Aave V2 LendingPool.
///
/// WARNING: Restricted to dry-run mode by default to prevent accidental transactions.
/// Use --dry-run to simulate.
///
/// Calls LendingPool.repay(asset, amount, rateMode, onBehalfOf).
/// Selector: 0x573ade81
pub async fn run(
    chain_id: u64,
    asset: &str,
    amount: Option<f64>,
    all: bool,
    rate_mode: u128,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<Value> {
    if !dry_run {
        anyhow::bail!(
            "repay is restricted to --dry-run mode. Add --dry-run to simulate the transaction."
        );
    }

    if amount.is_none() && !all {
        anyhow::bail!("Must specify --amount or --all for repay");
    }

    let cfg = get_chain_config(chain_id)?;

    let from_addr = if let Some(addr) = from {
        addr.to_string()
    } else {
        onchainos::resolve_wallet(chain_id, true)
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

    let repay_calldata = calldata::encode_repay(&token_addr, amount_minimal, rate_mode, &from_addr)
        .context("Failed to encode repay calldata")?;

    let rate_mode_label = if rate_mode == INTEREST_RATE_MODE_VARIABLE {
        "variable"
    } else {
        "stable"
    };

    let cmd = format!(
        "onchainos wallet contract-call --chain {} --to {} --input-data {} --from {} --dry-run --force",
        chain_id, pool_addr, repay_calldata, from_addr
    );
    eprintln!("[dry-run] repay: {}", cmd);

    Ok(json!({
        "ok": true,
        "dryRun": true,
        "warning": "repay is dry-run only",
        "asset": asset,
        "tokenAddress": token_addr,
        "amount": if all { "all".to_string() } else { amount.unwrap().to_string() },
        "amountMinimal": if amount_minimal == u128::MAX { "uint256.max".to_string() } else { amount_minimal.to_string() },
        "rateMode": rate_mode,
        "rateModeLabel": rate_mode_label,
        "lendingPool": pool_addr,
        "simulatedCommand": cmd
    }))
}
