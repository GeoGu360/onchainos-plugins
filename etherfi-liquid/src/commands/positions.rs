/// positions — show user's current positions across all Liquid vaults

use anyhow::Result;
use serde_json::{json, Value};

use crate::config::VAULTS;
use crate::rpc::{erc20_balance_of, get_rate_in_quote};

pub async fn execute(wallet: &str, rpc_url: &str) -> Result<()> {
    let mut position_list: Vec<Value> = Vec::new();

    for v in VAULTS {
        // Read share balance
        let shares = erc20_balance_of(v.vault, wallet, rpc_url).await.unwrap_or(0);

        // Read rate for value calculation
        let rate = get_rate_in_quote(v.accountant, v.deposit_token, rpc_url)
            .await
            .unwrap_or(0);

        let decimals = v.deposit_token_decimals as i32;
        let shares_display = shares as f64 / 1e18; // shares always 18 dec

        // rate is in quote token units (scaled by token decimals, NOT 18)
        // e.g. USDC rate = 1151993 means 1.151993 USDC per share (1e6 scaling)
        // value = (shares / 1e18) * (rate / 10^token_decimals)
        let value_in_token = if rate > 0 {
            (shares as f64 / 1e18) * (rate as f64 / 10f64.powi(decimals))
        } else {
            0.0
        };

        position_list.push(json!({
            "vault": v.symbol,
            "name": v.name,
            "vault_address": v.vault,
            "shares": shares.to_string(),
            "shares_display": format!("{:.8}", shares_display),
            "value_in_token": format!("{:.8}", value_in_token),
            "value_token_symbol": v.deposit_token_symbol,
            "has_position": shares > 0,
        }));
    }

    let output = json!({
        "ok": true,
        "wallet": wallet,
        "positions": position_list,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
