use anyhow::Context;
use serde_json::{json, Value};

use crate::config::get_chain_config;
use crate::onchainos;
use crate::rpc;

/// View current Aave V2 positions.
///
/// Returns health factor, collateral, debt, and enriched positions
/// from onchainos defi positions.
///
/// Note: Aave V2 getUserAccountData returns ETH-denominated values (not USD).
pub async fn run(chain_id: u64, from: Option<&str>) -> anyhow::Result<Value> {
    let cfg = get_chain_config(chain_id)?;

    let user_addr = if let Some(addr) = from {
        addr.to_string()
    } else {
        onchainos::resolve_wallet(chain_id, false)
            .context("No --from address specified and could not resolve active wallet.")?
    };

    // Get positions from onchainos defi
    let positions_result = onchainos::defi_positions(chain_id, &user_addr);
    let positions = match positions_result {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "Warning: onchainos defi positions failed: {}. Showing health factor only.",
                e
            );
            json!(null)
        }
    };

    // Resolve LendingPool at runtime
    let pool_addr = rpc::get_lending_pool(cfg.lending_pool_addresses_provider, cfg.rpc_url)
        .await
        .unwrap_or_else(|_| cfg.lending_pool_proxy.to_string());

    let account_data = rpc::get_user_account_data(&pool_addr, &user_addr, cfg.rpc_url)
        .await
        .context("Failed to fetch user account data from LendingPool")?;

    let hf = account_data.health_factor_f64();

    Ok(json!({
        "ok": true,
        "chain": cfg.name,
        "chainId": chain_id,
        "userAddress": user_addr,
        "lendingPool": pool_addr,
        "healthFactor": if hf > 1e10 { "∞".to_string() } else { format!("{:.4}", hf) },
        "healthFactorStatus": account_data.health_factor_status(),
        "totalCollateralETH": format!("{:.6}", account_data.total_collateral_eth()),
        "totalDebtETH": format!("{:.6}", account_data.total_debt_eth()),
        "availableBorrowsETH": format!("{:.6}", account_data.available_borrows_eth()),
        "ltvBps": account_data.ltv,
        "liquidationThresholdBps": account_data.current_liquidation_threshold,
        "positions": positions
    }))
}
