/// rates command — show borrowing rates and protocol stats

use anyhow::Result;
use crate::{config, rpc};

pub async fn execute(rpc_url: &str) -> Result<()> {
    let usdc = config::USDC_ADDR;

    let apy_per_sec = rpc::borrow_apy_per_second(config::DEBT_MANAGER, usdc, rpc_url).await?;
    let total_supply = rpc::total_supplies(config::DEBT_MANAGER, usdc, rpc_url).await?;
    let total_borrow = rpc::total_borrowing_amount(config::DEBT_MANAGER, usdc, rpc_url).await?;

    const SECONDS_PER_YEAR: f64 = 31_536_000.0;
    let annual_apy_pct = (apy_per_sec as f64 / 1e18) * SECONDS_PER_YEAR * 100.0;

    let usdc_divisor = 1_000_000.0f64; // 6 decimals

    let utilization_pct = if total_supply > 0 {
        (total_borrow as f64 / total_supply as f64) * 100.0
    } else {
        0.0
    };

    // Collateral token configs
    let mut collateral_info = Vec::new();
    for ct in config::COLLATERAL_TOKENS {
        let (ltv, threshold, bonus) = rpc::collateral_token_config(
            config::DEBT_MANAGER, ct.address, rpc_url
        ).await.unwrap_or((0, 0, 0));
        collateral_info.push(serde_json::json!({
            "token": ct.symbol,
            "ltv_pct": format!("{:.1}", ltv as f64 / 1e18),
            "liquidation_threshold_pct": format!("{:.1}", threshold as f64 / 1e18),
            "liquidation_bonus_pct": format!("{:.2}", bonus as f64 / 1e18),
        }));
    }

    let output = serde_json::json!({
        "ok": true,
        "chain": "Scroll (534352)",
        "borrow_token": "USDC",
        "borrow_apy_pct": format!("{:.6}", annual_apy_pct),
        "total_supply_usdc": format!("{:.4}", total_supply as f64 / usdc_divisor),
        "total_borrow_usdc": format!("{:.4}", total_borrow as f64 / usdc_divisor),
        "utilization_pct": format!("{:.2}", utilization_pct),
        "collateral_tokens": collateral_info,
        "note": "Borrowing requires a UserSafe smart wallet. Use EtherFi Cash app at app.ether.fi to create one.",
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
