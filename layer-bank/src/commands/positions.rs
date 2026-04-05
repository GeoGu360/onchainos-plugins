// src/commands/positions.rs — View supplied and borrowed positions for a wallet
use anyhow::Result;
use serde_json::{json, Value};

use crate::config::{MARKETS, CORE, PRICE_CALCULATOR, RPC_URL};
use crate::onchainos::resolve_wallet;
use crate::rpc::{account_snapshot, account_liquidity_of, get_underlying_price};

pub async fn run(chain_id: u64, wallet: Option<String>) -> Result<Value> {
    if chain_id != 534352 {
        anyhow::bail!(
            "LayerBank is deployed on Scroll (chain 534352). Got chain {}.",
            chain_id
        );
    }

    let rpc = RPC_URL;

    // Resolve wallet address
    let address = match wallet {
        Some(ref w) => w.clone(),
        None => resolve_wallet(chain_id)?,
    };

    // Fetch aggregate account liquidity
    let (collateral_usd, supply_usd, borrow_usd) =
        account_liquidity_of(CORE, &address, rpc).await.unwrap_or((0, 0, 0));

    let collateral_human = collateral_usd as f64 / 1e18;
    let supply_human_usd = supply_usd as f64 / 1e18;
    let borrow_human_usd = borrow_usd as f64 / 1e18;

    // Health factor: collateral_usd / borrow_usd (> 1.0 is healthy)
    let health_factor = if borrow_human_usd > 0.0 {
        collateral_human / borrow_human_usd
    } else {
        f64::INFINITY
    };

    let mut supplied = Vec::new();
    let mut borrowed = Vec::new();

    for m in MARKETS {
        let (ltoken_bal, borrow_bal, er) = account_snapshot(m.ltoken, &address, rpc).await.unwrap_or((0, 0, 0));

        if ltoken_bal > 0 || borrow_bal > 0 {
            let price_raw = get_underlying_price(PRICE_CALCULATOR, m.ltoken, rpc).await.unwrap_or(0);
            let price_usd = price_raw as f64 / 1e18;
            let factor = 10f64.powi(m.underlying_decimals as i32);
            let er_float = (er as f64) / 1e18;

            if ltoken_bal > 0 {
                // Supply balance = lTokenBalance * exchangeRate / 1e18
                let supply_underlying = (ltoken_bal as f64) * er_float / factor;
                let supply_val_usd = supply_underlying * price_usd;
                supplied.push(json!({
                    "asset": m.symbol,
                    "ltoken_balance": format!("{:.8}", (ltoken_bal as f64) / 1e18),
                    "supply_balance": format!("{:.6}", supply_underlying),
                    "supply_value_usd": format!("{:.4}", supply_val_usd),
                    "ltoken_address": m.ltoken
                }));
            }

            if borrow_bal > 0 {
                let borrow_underlying = (borrow_bal as f64) / factor;
                let borrow_val_usd = borrow_underlying * price_usd;
                borrowed.push(json!({
                    "asset": m.symbol,
                    "borrow_balance": format!("{:.6}", borrow_underlying),
                    "borrow_value_usd": format!("{:.4}", borrow_val_usd),
                    "ltoken_address": m.ltoken
                }));
            }
        }
    }

    let hf_display = if health_factor.is_infinite() {
        "∞ (no debt)".to_string()
    } else {
        format!("{:.4}", health_factor)
    };

    Ok(json!({
        "ok": true,
        "chain_id": chain_id,
        "chain": "Scroll",
        "protocol": "LayerBank",
        "wallet": address,
        "summary": {
            "total_collateral_usd": format!("{:.4}", collateral_human),
            "total_supply_usd": format!("{:.4}", supply_human_usd),
            "total_borrow_usd": format!("{:.4}", borrow_human_usd),
            "health_factor": hf_display
        },
        "supplied": supplied,
        "borrowed": borrowed
    }))
}
