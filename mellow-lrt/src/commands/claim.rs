use crate::api;
use crate::config::{get_chain_config, resolve_vault_static};
use crate::onchainos;
use crate::rpc;

/// Claim previously queued withdrawal from a Mellow LRT vault.
/// Call this after ~14 days since the 'withdraw' operation.
/// Checks claimableAssetsOf first; will fail if nothing is claimable.
///
/// CONFIRM: This is an on-chain write operation. Review amounts before executing.
pub async fn run(
    vault_input: &str,
    chain_id: u64,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let cfg = get_chain_config(chain_id)?;

    // Resolve vault
    let (vault_addr, _base_addr, base_decimals) = resolve_vault(vault_input, chain_id, cfg.rpc_url).await?;

    // dry_run guard — before resolve_wallet
    if dry_run {
        // claim(address account, address recipient, uint256 maxAmount) selector: 0x996cba68
        let calldata = format!(
            "0x996cba68{}{}{}",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        );
        let output = serde_json::json!({
            "ok": true,
            "dry_run": true,
            "operation": "claim",
            "vault": vault_addr,
            "calldata": calldata,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    let wallet = from.map(|s| s.to_string())
        .unwrap_or_else(|| onchainos::resolve_wallet(chain_id, false).unwrap_or_default());
    if wallet.is_empty() {
        anyhow::bail!("Could not resolve wallet address.");
    }
    let wallet_clean = wallet.trim_start_matches("0x").to_lowercase();

    // Check claimable amount
    let claimable = rpc::vault_claimable_assets_of(&vault_addr, &wallet, cfg.rpc_url).await;
    let claimable_amount = claimable.unwrap_or(0);

    if claimable_amount == 0 {
        let pending = rpc::vault_pending_assets_of(&vault_addr, &wallet, cfg.rpc_url).await;
        let output = serde_json::json!({
            "ok": false,
            "operation": "claim",
            "vault": vault_addr,
            "claimableAssets": "0",
            "pendingAssets": pending.map(|v| rpc::format_amount(v, base_decimals)).unwrap_or_else(|| "N/A".to_string()),
            "message": "Nothing to claim yet. If you have pending assets, wait for the withdrawal queue to process (~14 days).",
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    eprintln!(
        "[mellow-lrt] Claiming {} wstETH from vault {} on {}",
        rpc::format_amount(claimable_amount, base_decimals),
        vault_addr,
        cfg.name
    );

    // claim(address account, address recipient, uint256 maxAmount)  selector: 0x996cba68
    let max_amount_hex = format!("{:064x}", u128::MAX);
    let calldata = format!(
        "0x996cba68{:0>64}{:0>64}{}",
        wallet_clean, wallet_clean, max_amount_hex
    );

    let result = onchainos::wallet_contract_call(
        chain_id,
        &vault_addr,
        &calldata,
        None,
        false,
    ).await?;
    let tx_hash = onchainos::extract_tx_hash(&result);

    let output = serde_json::json!({
        "ok": true,
        "operation": "claim",
        "vault": vault_addr,
        "vaultSymbol": vault_input,
        "claimedAssets": rpc::format_amount(claimable_amount, base_decimals),
        "recipient": wallet,
        "chain": cfg.name,
        "chainId": chain_id,
        "txHash": tx_hash,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Resolve vault address from symbol or address.
async fn resolve_vault(input: &str, chain_id: u64, rpc_url: &str) -> anyhow::Result<(String, String, u8)> {
    if let Some((vault_addr, base_addr, decimals)) = resolve_vault_static(input) {
        if !base_addr.is_empty() {
            return Ok((vault_addr, base_addr, decimals));
        }
        let base = rpc::vault_asset(&vault_addr, rpc_url).await?;
        let dec = rpc::erc20_decimals(&base, rpc_url).await.unwrap_or(18);
        return Ok((vault_addr, base, dec));
    }
    let vaults = api::fetch_vaults(Some(chain_id)).await.unwrap_or_default();
    let input_lower = input.to_lowercase();
    for v in &vaults {
        if v.symbol.to_lowercase() == input_lower || v.address.to_lowercase() == input_lower {
            let (base_addr, decimals) = v.base_token.as_ref()
                .map(|t| (t.address.clone(), t.decimals))
                .unwrap_or_else(|| ("0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0".to_string(), 18));
            return Ok((v.address.clone(), base_addr, decimals));
        }
    }
    anyhow::bail!(
        "Unknown vault '{}'. Run 'mellow-lrt --chain {} vaults' to list vaults.",
        input, chain_id
    )
}
