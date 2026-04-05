// src/commands/pools.rs — List INIT Capital lending pools on Blast
use anyhow::Result;
use serde_json::{json, Value};

use crate::config::{POOLS, RPC_URL, RPC_URL_FALLBACK};
use crate::rpc::{total_assets, get_supply_rate_e18, get_borrow_rate_e18, to_amt};

/// The inToken (pool token) has 34 decimals; 1 inToken share = 1e34 base units
/// Use toAmt(1e34) to get the current exchange rate to underlying
const IN_TOKEN_DECIMALS: u32 = 34;
const ONE_SHARE: u128 = 10u128.pow(34); // Note: may overflow if u128 max is ~3.4e38

pub async fn run(chain_id: u64) -> Result<Value> {
    if chain_id != 81457 {
        anyhow::bail!(
            "INIT Capital Blast deployment uses chain 81457. Got chain {}.",
            chain_id
        );
    }

    let rpc = RPC_URL;
    let mut markets = Vec::new();

    for pool in POOLS {
        let total_raw = match total_assets(pool.pool, rpc).await {
            Ok(v) => v,
            Err(_) => total_assets(pool.pool, RPC_URL_FALLBACK).await.unwrap_or(0),
        };
        let supply_rate_e18 = match get_supply_rate_e18(pool.pool, rpc).await {
            Ok(v) => v,
            Err(_) => get_supply_rate_e18(pool.pool, RPC_URL_FALLBACK).await.unwrap_or(0),
        };
        let borrow_rate_e18 = match get_borrow_rate_e18(pool.pool, rpc).await {
            Ok(v) => v,
            Err(_) => get_borrow_rate_e18(pool.pool, RPC_URL_FALLBACK).await.unwrap_or(0),
        };

        // Rates are per-second in e18, multiply by seconds per year = 31536000
        let supply_apy_pct = (supply_rate_e18 as f64 / 1e18) * 31_536_000.0 * 100.0;
        let borrow_apy_pct = (borrow_rate_e18 as f64 / 1e18) * 31_536_000.0 * 100.0;

        // Convert inToken shares to underlying amount using exchange rate
        // toAmt(1e34) returns how much underlying 1 share is worth (e18 precision)
        let one_share_in_underlying = to_amt(pool.pool, ONE_SHARE, rpc).await.unwrap_or(0);
        let decimals = pool.underlying_decimals;

        // total underlying = total_raw * one_share_in_underlying / 1e34
        // Use f64 for large number arithmetic
        let total_underlying = (total_raw as f64) * (one_share_in_underlying as f64) / 1e34;
        let total_human = total_underlying / 10f64.powi(decimals as i32);

        markets.push(json!({
            "symbol": pool.symbol,
            "pool_address": pool.pool,
            "underlying": pool.underlying,
            "total_supplied": format!("{:.6}", total_human),
            "supply_apy_pct": format!("{:.4}", supply_apy_pct),
            "borrow_apy_pct": format!("{:.4}", borrow_apy_pct),
            "supply_rate_e18": supply_rate_e18.to_string(),
            "borrow_rate_e18": borrow_rate_e18.to_string()
        }));
    }

    Ok(json!({
        "ok": true,
        "chain_id": chain_id,
        "protocol": "INIT Capital",
        "chain": "Blast",
        "pool_count": markets.len(),
        "pools": markets
    }))
}
