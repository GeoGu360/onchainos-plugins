// src/commands/markets.rs -- List Benqi qiToken markets with APR and exchange rates
use anyhow::Result;
use serde_json::{json, Value};

use crate::config::{MARKETS, SECONDS_PER_YEAR, RPC_URL, CHAIN_ID};
use crate::rpc::{supply_rate_per_timestamp, borrow_rate_per_timestamp, exchange_rate_current, rate_to_apr_pct};

pub async fn run(chain_id: u64) -> Result<Value> {
    if chain_id != CHAIN_ID {
        anyhow::bail!("Benqi Lending is only supported on Avalanche C-Chain (chain 43114). Got chain {}.", chain_id);
    }

    let rpc = RPC_URL;
    let mut markets = Vec::new();

    for m in MARKETS {
        let supply_rate = supply_rate_per_timestamp(m.qi_token, rpc).await.unwrap_or(0);
        let borrow_rate = borrow_rate_per_timestamp(m.qi_token, rpc).await.unwrap_or(0);
        let exchange_rate = exchange_rate_current(m.qi_token, rpc).await.unwrap_or(0);

        let supply_apr = rate_to_apr_pct(supply_rate, SECONDS_PER_YEAR);
        let borrow_apr = rate_to_apr_pct(borrow_rate, SECONDS_PER_YEAR);

        // exchange_rate is scaled by 1e18 * 10^(underlying_decimals - qi_token_decimals)
        let exp_diff = m.underlying_decimals as i32 - m.qi_token_decimals as i32;
        let er_human = if exchange_rate > 0 {
            let scale = 10f64.powi(exp_diff);
            (exchange_rate as f64) / 1e18 / scale
        } else {
            0.0
        };

        markets.push(json!({
            "symbol": m.symbol,
            "qi_token": m.qi_token,
            "underlying": m.underlying.unwrap_or("AVAX (native)"),
            "supply_apr_pct": format!("{:.4}", supply_apr),
            "borrow_apr_pct": format!("{:.4}", borrow_apr),
            "exchange_rate": format!("{:.8}", er_human),
            "note": format!("1 qi{} = {:.6} {}", m.symbol, er_human, m.symbol)
        }));
    }

    Ok(json!({
        "ok": true,
        "chain_id": chain_id,
        "chain": "Avalanche C-Chain",
        "protocol": "Benqi Lending",
        "markets": markets
    }))
}
