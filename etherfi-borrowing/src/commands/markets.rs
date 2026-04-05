/// markets command — list supported borrow and collateral tokens

use anyhow::Result;
use crate::{config, rpc};

pub async fn execute(rpc_url: &str) -> Result<()> {
    // Get borrow tokens
    let borrow_tokens = rpc::get_borrow_tokens(config::DEBT_MANAGER, rpc_url).await
        .unwrap_or_default();

    // Get collateral tokens
    let collateral_tokens = rpc::get_collateral_tokens(config::DEBT_MANAGER, rpc_url).await
        .unwrap_or_default();

    // Resolve total supplies and borrow amounts
    let mut borrow_markets = Vec::new();
    for token_addr in &borrow_tokens {
        let symbol = config::resolve_token_symbol(token_addr);
        let decimals = config::BORROW_TOKENS.iter()
            .find(|t| t.address.to_lowercase() == token_addr.to_lowercase())
            .map(|t| t.decimals)
            .unwrap_or(6);

        let total_supply = rpc::total_supplies(config::DEBT_MANAGER, token_addr, rpc_url).await.unwrap_or(0);
        let total_borrow = rpc::total_borrowing_amount(config::DEBT_MANAGER, token_addr, rpc_url).await.unwrap_or(0);
        let apy_per_sec = rpc::borrow_apy_per_second(config::DEBT_MANAGER, token_addr, rpc_url).await.unwrap_or(0);

        // Annual APY: rate_per_sec * SECONDS_PER_YEAR / 1e18
        const SECONDS_PER_YEAR: f64 = 31_536_000.0;
        let annual_apy_pct = (apy_per_sec as f64 / 1e18) * SECONDS_PER_YEAR * 100.0;

        let divisor = 10u128.pow(decimals as u32) as f64;
        let utilization_pct = if total_supply > 0 {
            (total_borrow as f64 / total_supply as f64) * 100.0
        } else {
            0.0
        };

        borrow_markets.push(serde_json::json!({
            "token": symbol,
            "address": token_addr,
            "total_supply": format!("{:.4}", total_supply as f64 / divisor),
            "total_borrow": format!("{:.4}", total_borrow as f64 / divisor),
            "borrow_apy_pct": format!("{:.4}", annual_apy_pct),
            "utilization_pct": format!("{:.2}", utilization_pct),
        }));
    }

    // Resolve collateral token configs
    let mut collateral_markets = Vec::new();
    for token_addr in &collateral_tokens {
        let symbol = config::resolve_token_symbol(token_addr);
        let (ltv, liq_threshold, liq_bonus) = rpc::collateral_token_config(
            config::DEBT_MANAGER, token_addr, rpc_url
        ).await.unwrap_or((0, 0, 0));

        // HUNDRED_PERCENT = 100e18
        let ltv_pct = ltv as f64 / 1e18;
        let threshold_pct = liq_threshold as f64 / 1e18;
        let bonus_pct = liq_bonus as f64 / 1e18;

        collateral_markets.push(serde_json::json!({
            "token": symbol,
            "address": token_addr,
            "ltv_pct": format!("{:.1}", ltv_pct),
            "liquidation_threshold_pct": format!("{:.1}", threshold_pct),
            "liquidation_bonus_pct": format!("{:.2}", bonus_pct),
        }));
    }

    let output = serde_json::json!({
        "ok": true,
        "chain": "Scroll (534352)",
        "debt_manager": config::DEBT_MANAGER,
        "borrow_markets": borrow_markets,
        "collateral_markets": collateral_markets,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
