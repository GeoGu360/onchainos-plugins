use crate::api;
use crate::config::get_chain_config;
use crate::rpc;

/// Show user's current positions across all Mellow LRT vaults.
/// Queries balanceOf, convertToAssets, claimableAssetsOf, pendingAssetsOf for each vault.
pub async fn run(chain_id: u64, wallet: &str) -> anyhow::Result<()> {
    let cfg = get_chain_config(chain_id)?;
    let vaults = api::fetch_vaults(Some(chain_id)).await?;

    let mut positions = Vec::new();

    for vault in &vaults {
        // Check shares balance
        let shares = rpc::erc20_balance_of(&vault.address, wallet, cfg.rpc_url)
            .await
            .unwrap_or(0);

        let claimable = rpc::vault_claimable_assets_of(&vault.address, wallet, cfg.rpc_url).await;
        let pending = rpc::vault_pending_assets_of(&vault.address, wallet, cfg.rpc_url).await;

        let has_position = shares > 0
            || claimable.map_or(false, |v| v > 0)
            || pending.map_or(false, |v| v > 0);

        if !has_position {
            continue;
        }

        let decimals = vault.base_token.as_ref().map(|t| t.decimals).unwrap_or(18);
        let base_symbol = vault.base_token.as_ref().map(|t| t.symbol.as_str()).unwrap_or("TOKEN");

        // Convert shares -> underlying assets
        let assets = if shares > 0 {
            rpc::vault_convert_to_assets(&vault.address, shares, cfg.rpc_url)
                .await
                .unwrap_or(shares)
        } else {
            0
        };

        positions.push(serde_json::json!({
            "vault": vault.symbol,
            "vaultName": vault.name,
            "vaultAddress": vault.address,
            "shares": shares.to_string(),
            "sharesHuman": rpc::format_amount(shares, decimals),
            "estimatedAssets": rpc::format_amount(assets, decimals),
            "baseToken": base_symbol,
            "claimableAssets": claimable.map(|v| rpc::format_amount(v, decimals)).unwrap_or_else(|| "N/A".to_string()),
            "pendingAssets": pending.map(|v| rpc::format_amount(v, decimals)).unwrap_or_else(|| "N/A".to_string()),
            "priceUsd": vault.price,
            "withdrawAction": format!(
                "mellow-lrt --chain {} withdraw --vault {} --all",
                chain_id, vault.symbol
            ),
        }));
    }

    let output = serde_json::json!({
        "ok": true,
        "wallet": wallet,
        "chain": cfg.name,
        "chainId": chain_id,
        "totalPositions": positions.len(),
        "positions": positions,
        "note": if positions.is_empty() {
            "No active Mellow LRT positions found."
        } else {
            "claimableAssets: ready to claim now. pendingAssets: in withdrawal queue (may take ~14 days)."
        }
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
