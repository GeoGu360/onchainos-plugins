use crate::api;
use crate::rpc;
use crate::config::get_chain_config;

/// List Mellow LRT vaults on the given chain, sorted by TVL.
/// Shows symbol, name, APR, TVL, underlying token, deposit_tokens.
pub async fn run(chain_id: u64, limit: usize) -> anyhow::Result<()> {
    let cfg = get_chain_config(chain_id)?;
    let mut vaults = api::fetch_vaults(Some(chain_id)).await?;

    // Sort by tvl_usd descending
    vaults.sort_by(|a, b| {
        b.tvl_usd.unwrap_or(0.0)
            .partial_cmp(&a.tvl_usd.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total = vaults.len();
    let showing = vaults.len().min(limit);
    let mut vault_list = Vec::new();

    for vault in vaults.iter().take(showing) {
        let base = vault.base_token.as_ref().map(|t| serde_json::json!({
            "address": t.address,
            "symbol": t.symbol,
            "decimals": t.decimals,
        })).unwrap_or_else(|| serde_json::json!(null));

        let deposit_symbols: Vec<String> = vault.deposit_tokens.as_ref()
            .map(|tokens| tokens.iter().map(|t| t.symbol.clone()).collect())
            .unwrap_or_default();

        let withdraw_symbols: Vec<String> = vault.withdraw_tokens.as_ref()
            .map(|tokens| tokens.iter().map(|t| t.symbol.clone()).collect())
            .unwrap_or_default();

        // Fetch on-chain totalAssets for live data
        let total_assets_raw = rpc::vault_total_assets(&vault.address, cfg.rpc_url).await.unwrap_or(0);
        let decimals = vault.base_token.as_ref().map(|t| t.decimals).unwrap_or(18);
        let total_assets_human = rpc::format_amount(total_assets_raw, decimals);

        vault_list.push(serde_json::json!({
            "id": vault.id,
            "symbol": vault.symbol,
            "name": vault.name,
            "address": vault.address,
            "layer": vault.layer,
            "baseToken": base,
            "depositTokens": deposit_symbols,
            "withdrawTokens": withdraw_symbols,
            "apr": vault.apr.map(|v| format!("{:.3}", v)).unwrap_or_else(|| "N/A".to_string()),
            "aprPct": vault.apr.unwrap_or(0.0),
            "tvlUsd": vault.tvl_usd.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "N/A".to_string()),
            "totalAssets": total_assets_human,
            "price": vault.price.map(|v| format!("{:.4}", v)).unwrap_or_else(|| "N/A".to_string()),
            "withdrawAvgTimeDays": vault.withdraw_avg_time_seconds.map(|s| s as f64 / 86400.0).unwrap_or(0.0),
            "depositInstruction": format!(
                "mellow-lrt --chain {} deposit --vault {} --token ETH --amount <n>",
                chain_id, vault.symbol
            ),
        }));
    }

    let output = serde_json::json!({
        "ok": true,
        "chain": cfg.name,
        "chainId": chain_id,
        "totalVaults": total,
        "showing": showing,
        "vaults": vault_list,
        "note": "Mellow LRT vaults accept ETH/WETH/stETH/wstETH deposits via EthWrapper. Withdrawals are async (2-step: redeem + claim after ~14 days)."
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
