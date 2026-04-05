use anyhow::Context;
use serde_json::{json, Value};

use crate::calldata;
use crate::config::{get_chain_config, INTEREST_RATE_MODE_VARIABLE};
use crate::onchainos;
use crate::rpc;

/// Borrow an asset from Aave V2 LendingPool.
///
/// WARNING: Borrowing carries liquidation risk. This command is restricted to
/// dry-run mode only to prevent accidental liquidation.
///
/// Calls LendingPool.borrow(asset, amount, interestRateMode, referralCode, onBehalfOf).
/// Selector: 0xa415bcad
///
/// V2 supports both stable (1) and variable (2) rate modes.
/// Default: variable (2).
pub async fn run(
    chain_id: u64,
    asset: &str,
    amount: f64,
    rate_mode: u128,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<Value> {
    if !dry_run {
        anyhow::bail!(
            "borrow is restricted to --dry-run mode to prevent liquidation risk. \
            Add --dry-run to simulate the transaction."
        );
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

    let factor = 10u128.pow(decimals as u32);
    let amount_minimal = (amount * factor as f64) as u128;

    let pool_addr = rpc::get_lending_pool(cfg.lending_pool_addresses_provider, cfg.rpc_url)
        .await
        .unwrap_or_else(|_| cfg.lending_pool_proxy.to_string());

    let borrow_calldata = calldata::encode_borrow(&token_addr, amount_minimal, rate_mode, &from_addr)
        .context("Failed to encode borrow calldata")?;

    let rate_mode_label = if rate_mode == INTEREST_RATE_MODE_VARIABLE {
        "variable"
    } else {
        "stable"
    };

    let cmd = format!(
        "onchainos wallet contract-call --chain {} --to {} --input-data {} --from {} --dry-run --force",
        chain_id, pool_addr, borrow_calldata, from_addr
    );
    eprintln!("[dry-run] borrow: {}", cmd);

    Ok(json!({
        "ok": true,
        "dryRun": true,
        "warning": "borrow is dry-run only — liquidation risk",
        "asset": asset,
        "tokenAddress": token_addr,
        "amount": amount,
        "amountMinimal": amount_minimal.to_string(),
        "rateMode": rate_mode,
        "rateModeLabel": rate_mode_label,
        "lendingPool": pool_addr,
        "simulatedCommand": cmd
    }))
}
