// src/commands/positions.rs — View user's INIT Capital positions on Blast
use anyhow::Result;
use serde_json::{json, Value};

use crate::config::{POOLS, POS_MANAGER, INIT_CORE, RPC_URL, RPC_URL_FALLBACK};
use crate::onchainos::resolve_wallet;
use crate::rpc::{
    get_viewer_pos_ids_length, get_viewer_pos_ids_at, get_pos_mode,
    get_coll_amt, get_pos_debt_shares, get_pos_health_current_e18,
    to_amt, debt_share_to_amt_current,
};

pub async fn run(chain_id: u64, wallet: Option<String>) -> Result<Value> {
    if chain_id != 81457 {
        anyhow::bail!(
            "INIT Capital Blast deployment uses chain 81457. Got chain {}.",
            chain_id
        );
    }

    let rpc = RPC_URL;
    let wallet_addr = match wallet {
        Some(w) => w,
        None => resolve_wallet(chain_id)?,
    };

    // Get number of positions for this viewer
    let pos_count = match get_viewer_pos_ids_length(POS_MANAGER, &wallet_addr, rpc).await {
        Ok(n) => n,
        Err(_) => get_viewer_pos_ids_length(POS_MANAGER, &wallet_addr, RPC_URL_FALLBACK).await.unwrap_or(0),
    };

    if pos_count == 0 {
        return Ok(json!({
            "ok": true,
            "wallet": wallet_addr,
            "position_count": 0,
            "positions": [],
            "message": "No INIT Capital positions found for this wallet on Blast"
        }));
    }

    let mut positions = Vec::new();

    for i in 0..pos_count.min(20) { // cap at 20 positions
        let pos_id = match get_viewer_pos_ids_at(POS_MANAGER, &wallet_addr, i, rpc).await {
            Ok(id) => id,
            Err(_) => match get_viewer_pos_ids_at(POS_MANAGER, &wallet_addr, i, RPC_URL_FALLBACK).await {
                Ok(id) => id,
                Err(_) => continue,
            }
        };

        let mode = get_pos_mode(POS_MANAGER, pos_id, rpc).await.unwrap_or(0);

        // Get health factor (returns very large value if no borrows = healthy)
        let health_e18 = get_pos_health_current_e18(INIT_CORE, pos_id, rpc).await.unwrap_or(u128::MAX);
        let health_display = if health_e18 > 100_000_000_000_000_000_000u128 {
            "infinity (no borrows)".to_string()
        } else {
            format!("{:.4}", health_e18 as f64 / 1e18)
        };

        let mut collaterals = Vec::new();
        let mut debts = Vec::new();

        for pool in POOLS {
            let decimals = pool.underlying_decimals;

            // Check collateral (inToken shares)
            let coll_shares = get_coll_amt(POS_MANAGER, pos_id, pool.pool, rpc).await.unwrap_or(0);
            if coll_shares > 0 {
                let coll_amt = to_amt(pool.pool, coll_shares, rpc).await.unwrap_or(coll_shares);
                let coll_human = coll_amt as f64 / 10f64.powi(decimals as i32);
                collaterals.push(json!({
                    "asset": pool.symbol,
                    "shares": coll_shares.to_string(),
                    "amount": format!("{:.8}", coll_human)
                }));
            }

            // Check debt (borrow shares)
            let debt_shares = get_pos_debt_shares(POS_MANAGER, pos_id, pool.pool, rpc).await.unwrap_or(0);
            if debt_shares > 0 {
                let debt_amt = debt_share_to_amt_current(pool.pool, debt_shares, rpc).await.unwrap_or(debt_shares);
                let debt_human = debt_amt as f64 / 10f64.powi(decimals as i32);
                debts.push(json!({
                    "asset": pool.symbol,
                    "debt_shares": debt_shares.to_string(),
                    "debt_amount": format!("{:.8}", debt_human)
                }));
            }
        }

        positions.push(json!({
            "pos_id": pos_id,
            "mode": mode,
            "health_factor": health_display,
            "collaterals": collaterals,
            "debts": debts
        }));
    }

    Ok(json!({
        "ok": true,
        "wallet": wallet_addr,
        "chain": "Blast (81457)",
        "position_count": pos_count,
        "positions": positions
    }))
}
