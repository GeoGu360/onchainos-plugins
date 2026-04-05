// src/commands/positions.rs -- Show user's supplied and borrowed positions on Benqi
use anyhow::Result;
use serde_json::{json, Value};

use crate::config::{MARKETS, COMPTROLLER, RPC_URL, CHAIN_ID};
use crate::onchainos::resolve_wallet;
use crate::rpc::{balance_of, borrow_balance_current, exchange_rate_current, get_account_liquidity, qi_token_to_underlying};

pub async fn run(chain_id: u64, wallet: Option<String>) -> Result<Value> {
    if chain_id != CHAIN_ID {
        anyhow::bail!("Benqi Lending is only supported on Avalanche C-Chain (chain 43114). Got chain {}.", chain_id);
    }

    let address = match wallet {
        Some(w) => w,
        None => resolve_wallet(chain_id)?,
    };

    let rpc = RPC_URL;
    let mut positions = Vec::new();

    for m in MARKETS {
        let qi_bal = balance_of(m.qi_token, &address, rpc).await.unwrap_or(0);
        let borrow_bal = borrow_balance_current(m.qi_token, &address, rpc).await.unwrap_or(0);

        if qi_bal > 0 || borrow_bal > 0 {
            let exchange_rate = exchange_rate_current(m.qi_token, rpc).await.unwrap_or(0);
            let underlying_human = qi_token_to_underlying(qi_bal, exchange_rate, m.underlying_decimals, m.qi_token_decimals);
            let borrow_human = (borrow_bal as f64) / 10f64.powi(m.underlying_decimals as i32);
            let qi_human = (qi_bal as f64) / 10f64.powi(m.qi_token_decimals as i32);

            positions.push(json!({
                "asset": m.symbol,
                "qi_token_address": m.qi_token,
                "qi_token_balance": format!("{:.8}", qi_human),
                "supplied_underlying": format!("{:.8}", underlying_human),
                "borrowed": format!("{:.8}", borrow_human)
            }));
        }
    }

    // Get account liquidity from Comptroller
    let (err_code, liquidity, shortfall) = get_account_liquidity(COMPTROLLER, &address, rpc).await.unwrap_or((0, 0, 0));
    let liquidity_usd = (liquidity as f64) / 1e18;
    let shortfall_usd = (shortfall as f64) / 1e18;
    let health_note = if err_code != 0 {
        "error fetching liquidity".to_string()
    } else if shortfall > 0 {
        format!("UNDERCOLLATERALIZED — shortfall ${:.4}", shortfall_usd)
    } else {
        format!("healthy — available borrow capacity ${:.4}", liquidity_usd)
    };

    if positions.is_empty() {
        return Ok(json!({
            "ok": true,
            "chain_id": chain_id,
            "wallet": address,
            "positions": [],
            "account_liquidity_usd": format!("{:.4}", liquidity_usd),
            "message": "No active positions found on Benqi Lending."
        }));
    }

    Ok(json!({
        "ok": true,
        "chain_id": chain_id,
        "chain": "Avalanche C-Chain",
        "wallet": address,
        "positions": positions,
        "account_status": health_note,
        "available_borrow_usd": format!("{:.4}", liquidity_usd),
        "shortfall_usd": format!("{:.4}", shortfall_usd)
    }))
}
