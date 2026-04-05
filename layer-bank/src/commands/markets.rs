// src/commands/markets.rs — List LayerBank lToken markets with TVL and prices
use anyhow::Result;
use serde_json::{json, Value};

use crate::config::{MARKETS, CORE, PRICE_CALCULATOR, RPC_URL};
use crate::rpc::{total_borrow, get_cash, total_supply_ltoken, exchange_rate, get_underlying_price};

pub async fn run(chain_id: u64) -> Result<Value> {
    if chain_id != 534352 {
        anyhow::bail!(
            "LayerBank is deployed on Scroll (chain 534352). Got chain {}. \
             Please use --chain 534352.",
            chain_id
        );
    }

    let rpc = RPC_URL;
    let mut market_list = Vec::new();

    for m in MARKETS {
        // Fetch lToken metrics
        let er = exchange_rate(m.ltoken, rpc).await.unwrap_or(0);
        let cash = get_cash(m.ltoken, rpc).await.unwrap_or(0);
        let borrow = total_borrow(m.ltoken, rpc).await.unwrap_or(0);
        let ltoken_supply = total_supply_ltoken(m.ltoken, rpc).await.unwrap_or(0);

        // Price in USD (18 decimals)
        let price_raw = get_underlying_price(PRICE_CALCULATOR, m.ltoken, rpc).await.unwrap_or(0);
        let price_usd = price_raw as f64 / 1e18;

        // Supply = lToken supply * exchangeRate / 1e18 → underlying raw units
        let supply_raw = if er > 0 {
            (ltoken_supply as f64) * (er as f64) / 1e18
        } else {
            0.0
        };
        let factor = 10f64.powi(m.underlying_decimals as i32);
        let supply_human = supply_raw / factor;
        let borrow_human = (borrow as f64) / factor;
        let cash_human = (cash as f64) / factor;

        // Utilization rate
        let total_pool = cash_human + borrow_human;
        let utilization = if total_pool > 0.0 { borrow_human / total_pool * 100.0 } else { 0.0 };

        // Exchange rate human (how much underlying per lToken)
        let er_human = (er as f64) / 1e18;

        // Supply TVL in USD
        let tvl_usd = supply_human * price_usd;

        market_list.push(json!({
            "symbol": m.symbol,
            "ltoken": m.ltoken,
            "underlying": m.underlying.unwrap_or("ETH (native)"),
            "price_usd": format!("{:.4}", price_usd),
            "total_supply": format!("{:.6}", supply_human),
            "total_borrow": format!("{:.6}", borrow_human),
            "available_liquidity": format!("{:.6}", cash_human),
            "utilization_pct": format!("{:.2}", utilization),
            "exchange_rate": format!("{:.8}", er_human),
            "tvl_usd": format!("{:.2}", tvl_usd),
            "collateral_factor_note": "see positions command for your LTV"
        }));
    }

    Ok(json!({
        "ok": true,
        "chain_id": chain_id,
        "chain": "Scroll",
        "protocol": "LayerBank",
        "core": CORE,
        "markets": market_list
    }))
}
