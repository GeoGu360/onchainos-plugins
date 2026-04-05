use crate::api;
use crate::config::{get_chain_config, resolve_vault_static};
use crate::onchainos;
use crate::rpc;

/// Withdraw (redeem) LRT shares from a Mellow vault.
/// Initiates the 2-step async withdrawal process.
/// Step 1: redeem(shares, receiver, owner) → funds enter withdrawal queue
/// Step 2: after ~14 days, call 'claim' to receive wstETH
///
/// CONFIRM: This is an on-chain write operation. Review amounts before executing.
pub async fn run(
    vault_input: &str,
    amount: Option<&str>,
    all: bool,
    chain_id: u64,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let cfg = get_chain_config(chain_id)?;

    // Resolve vault
    let (vault_addr, _base_addr, base_decimals) = resolve_vault(vault_input, chain_id, cfg.rpc_url).await?;

    // dry_run guard — before resolve_wallet
    if dry_run {
        let shares_raw: u128 = if all {
            u128::MAX // placeholder for dry-run
        } else {
            let amt = amount.ok_or_else(|| anyhow::anyhow!("Provide --amount or --all"))?;
            rpc::parse_amount(amt, base_decimals)?
        };
        // redeem(uint256,address,address) selector: 0xba087652
        let calldata = format!(
            "0xba087652{:064x}{}{}",
            shares_raw,
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
        let output = serde_json::json!({
            "ok": true,
            "dry_run": true,
            "operation": "withdraw",
            "vault": vault_addr,
            "calldata": calldata,
            "note": "Withdrawal is async. Funds enter Symbiotic withdrawal queue. Call 'claim' after ~14 days."
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    let wallet = from.map(|s| s.to_string())
        .unwrap_or_else(|| onchainos::resolve_wallet(chain_id, false).unwrap_or_default());
    if wallet.is_empty() {
        anyhow::bail!("Could not resolve wallet address. Pass --from or ensure onchainos is logged in.");
    }
    let wallet_clean = wallet.trim_start_matches("0x").to_lowercase();

    let shares = if all {
        let bal = rpc::erc20_balance_of(&vault_addr, &wallet, cfg.rpc_url).await.unwrap_or(0);
        if bal == 0 {
            anyhow::bail!("No shares to redeem in vault {}", vault_addr);
        }
        bal
    } else {
        let amt = amount.ok_or_else(|| anyhow::anyhow!("Provide --amount <n> or --all"))?;
        rpc::parse_amount(amt, base_decimals)?
    };

    let estimated_assets = rpc::vault_convert_to_assets(&vault_addr, shares, cfg.rpc_url)
        .await
        .unwrap_or(shares);
    let display_assets = rpc::format_amount(estimated_assets, base_decimals);
    let display_shares = rpc::format_amount(shares, base_decimals);

    eprintln!(
        "[mellow-lrt] Withdrawing {} shares (≈ {} wstETH) from vault {} on {}",
        display_shares, display_assets, vault_addr, cfg.name
    );

    // redeem(uint256 shares, address receiver, address owner)  selector: 0xba087652
    let calldata = format!(
        "0xba087652{:064x}{:0>64}{:0>64}",
        shares, wallet_clean, wallet_clean
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
        "operation": "withdraw",
        "vault": vault_addr,
        "vaultSymbol": vault_input,
        "shares": shares.to_string(),
        "sharesHuman": display_shares,
        "estimatedAssets": display_assets,
        "receiver": wallet,
        "chain": cfg.name,
        "chainId": chain_id,
        "txHash": tx_hash,
        "note": "Withdrawal queued. Funds pass through Symbiotic withdrawal queue (~14 days). Call 'mellow-lrt claim' after claimableAssetsOf > 0."
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
