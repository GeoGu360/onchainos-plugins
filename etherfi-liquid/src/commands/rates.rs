/// rates — show current exchange rates for each vault

use anyhow::Result;
use serde_json::{json, Value};

use crate::config::{DEFILLAMA_YIELDS_BASE, VAULTS};
use crate::rpc::{fetch_defillama_chart, get_rate_in_quote};

pub async fn execute(rpc_url: &str) -> Result<()> {
    let mut rate_list: Vec<Value> = Vec::new();

    for v in VAULTS {
        // Fetch share price from Accountant
        let rate_result = get_rate_in_quote(v.accountant, v.deposit_token, rpc_url).await;

        // Fetch APY from DefiLlama
        let llama = fetch_defillama_chart(v.defillama_pool_id, DEFILLAMA_YIELDS_BASE).await;
        let (apy, apy_7d) = match llama {
            Ok(dp) => (dp.apy, dp.apy_base7d),
            Err(_) => (None, None),
        };

        let (rate_raw, rate_display) = match rate_result {
            Ok(r) => {
                let decimals = v.deposit_token_decimals as i32;
                (r.to_string(), format!("{:.8}", r as f64 / 10f64.powi(decimals)))
            }
            Err(e) => ("0".to_string(), format!("error: {}", e)),
        };

        rate_list.push(json!({
            "vault": v.symbol,
            "name": v.name,
            "deposit_token": v.deposit_token_symbol,
            "rate_raw": rate_raw,
            "rate_display": rate_display,
            "rate_unit": format!("{} per {} share", v.deposit_token_symbol, v.symbol),
            "apy_pct": apy,
            "apy_7d_pct": apy_7d,
        }));
    }

    let output = json!({
        "ok": true,
        "rates": rate_list,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
