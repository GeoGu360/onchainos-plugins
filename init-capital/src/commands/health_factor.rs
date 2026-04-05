// src/commands/health_factor.rs — Get health factor for an INIT Capital position
use anyhow::Result;
use serde_json::{json, Value};

use crate::config::{INIT_CORE, RPC_URL, RPC_URL_FALLBACK};
use crate::rpc::get_pos_health_current_e18;

pub async fn run(chain_id: u64, pos_id: u64) -> Result<Value> {
    if chain_id != 81457 {
        anyhow::bail!(
            "INIT Capital Blast deployment uses chain 81457. Got chain {}.",
            chain_id
        );
    }

    let rpc = RPC_URL;
    let health_e18 = match get_pos_health_current_e18(INIT_CORE, pos_id as u128, rpc).await {
        Ok(v) => v,
        Err(_) => get_pos_health_current_e18(INIT_CORE, pos_id as u128, RPC_URL_FALLBACK).await.unwrap_or(0),
    };

    let health_display = if health_e18 == 0 {
        "0.0000 (unhealthy / no collateral)".to_string()
    } else if health_e18 > 100_000_000_000_000_000_000u128 {
        "very high (no borrows)".to_string()
    } else {
        format!("{:.4}", health_e18 as f64 / 1e18)
    };

    let status = if health_e18 == 0 {
        "no_collateral"
    } else if health_e18 < 1_000_000_000_000_000_000u128 {
        "liquidatable"
    } else if health_e18 < 1_100_000_000_000_000_000u128 {
        "at_risk"
    } else {
        "healthy"
    };

    Ok(json!({
        "ok": true,
        "pos_id": pos_id,
        "health_factor": health_display,
        "health_e18": health_e18.to_string(),
        "status": status,
        "note": "Health factor < 1.0 means the position can be liquidated"
    }))
}
