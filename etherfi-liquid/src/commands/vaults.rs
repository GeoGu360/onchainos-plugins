/// vaults — list available ether.fi Liquid vaults with APY and TVL

use anyhow::Result;
use serde_json::{json, Value};

use crate::config::{DEFILLAMA_YIELDS_BASE, VAULTS};
use crate::rpc::{fetch_defillama_chart, get_rate_in_quote};

pub async fn execute(rpc_url: &str) -> Result<()> {
    let mut vault_list: Vec<Value> = Vec::new();

    for v in VAULTS {
        // Fetch APY from DefiLlama
        let llama = fetch_defillama_chart(v.defillama_pool_id, DEFILLAMA_YIELDS_BASE).await;
        let (apy, apy_7d, tvl_usd) = match llama {
            Ok(dp) => (dp.apy, dp.apy_base7d, dp.tvl_usd),
            Err(_) => (None, None, None),
        };

        // Fetch current share price from Accountant
        let rate = get_rate_in_quote(v.accountant, v.deposit_token, rpc_url).await;
        let rate_display = match rate {
            Ok(r) => {
                let decimals = v.deposit_token_decimals as u32;
                r as f64 / 10f64.powi(decimals as i32)
            }
            Err(_) => 0.0,
        };

        vault_list.push(json!({
            "name": v.name,
            "symbol": v.symbol,
            "vault_address": v.vault,
            "teller_address": v.teller,
            "deposit_token": v.deposit_token_symbol,
            "apy_pct": apy,
            "apy_7d_pct": apy_7d,
            "tvl_usd": tvl_usd,
            "share_price": format!("{:.8}", rate_display),
            "share_price_unit": format!("{} per {} share", v.deposit_token_symbol, v.symbol),
        }));
    }

    let output = json!({
        "ok": true,
        "vaults": vault_list,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
