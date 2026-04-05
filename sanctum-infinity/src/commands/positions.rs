// positions — query user's INF holdings
use anyhow::Result;
use serde_json::json;
use std::process::Command;

use crate::api;
use crate::config::{INF_MINT, LST_DECIMALS};

pub async fn execute(client: &reqwest::Client) -> Result<()> {
    // Read wallet balance via onchainos
    let output = Command::new("onchainos")
        .args(["wallet", "balance", "--chain", "501"])
        .output()?;
    let wallet_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;

    // Find INF token in wallet assets
    let token_assets = wallet_json["data"]["details"][0]["tokenAssets"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let inf_asset = token_assets.iter().find(|a| {
        a["tokenAddress"].as_str().unwrap_or("") == INF_MINT
    });

    let inf_balance: f64 = inf_asset
        .and_then(|a| a["balance"].as_str())
        .and_then(|b| b.parse().ok())
        .unwrap_or(0.0);

    let inf_balance_raw: u64 = (inf_balance * 1e9).round() as u64;

    // Get INF NAV in SOL
    let sol_value_str = api::get_sol_value(client, "INF").await?;
    let nav_lamports: u64 = sol_value_str.parse().unwrap_or(0);
    let nav_sol = api::atomics_to_ui(nav_lamports, LST_DECIMALS);
    let inf_value_sol = inf_balance * nav_sol;

    let output = json!({
        "ok": true,
        "data": {
            "wallet": wallet_json["data"]["details"][0]["tokenAssets"][0]["address"].as_str().unwrap_or("unknown"),
            "inf_balance": inf_balance,
            "inf_mint": INF_MINT,
            "nav_sol_per_inf": nav_sol,
            "value_in_sol": inf_value_sol,
            "raw_balance_atomics": inf_balance_raw.to_string()
        }
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
