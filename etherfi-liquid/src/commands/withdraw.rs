/// withdraw — withdraw tokens from an ether.fi Liquid vault
///
/// Flow:
///   1. Read LIQUIDETH share balance of wallet
///   2. Build Teller.bulkWithdraw calldata — ask user to confirm
///   3. Execute bulkWithdraw(weETH, [shares], [minAssetsOut=0], [wallet])
///
/// Default vault: ETH Yield Vault (LIQUIDETH); withdraw to: weETH
/// Shares in human-readable units (e.g. "0.00005" for 0.00005 LIQUIDETH)
/// Or pass --all to withdraw entire balance

use anyhow::Result;
use serde_json::json;

use crate::config::VAULTS;
use crate::onchainos::{
    build_bulk_withdraw_calldata, extract_tx_hash, resolve_wallet, wallet_contract_call,
};
use crate::rpc::{erc20_balance_of, get_rate_in_quote};

pub async fn execute(
    vault_symbol: &str,
    shares_human: Option<f64>,
    withdraw_all: bool,
    chain_id: u64,
    rpc_url: &str,
    dry_run: bool,
) -> Result<()> {
    let vault = VAULTS
        .iter()
        .find(|v| v.symbol.eq_ignore_ascii_case(vault_symbol))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown vault symbol: {}. Use LIQUIDETH, LIQUIDUSD, or LIQUIDBTC",
                vault_symbol
            )
        })?;

    if dry_run {
        // Use placeholders for dry run
        let placeholder_shares: u128 = 50_000_000_000_000; // 0.00005 shares as placeholder
        let shares_to_use = if let Some(sh) = shares_human {
            (sh * 1e18) as u128
        } else {
            placeholder_shares
        };
        let receiver_placeholder = "0x0000000000000000000000000000000000000000";
        let calldata = build_bulk_withdraw_calldata(
            vault.deposit_token,
            shares_to_use,
            0,
            receiver_placeholder,
        );
        let output = json!({
            "ok": true,
            "dry_run": true,
            "vault": vault.symbol,
            "withdraw_token": vault.deposit_token_symbol,
            "shares_wei": shares_to_use.to_string(),
            "calldata": calldata,
            "teller": vault.teller,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    // Resolve wallet (after dry_run guard)
    let wallet = resolve_wallet(chain_id)?;

    // Read share balance
    let balance = erc20_balance_of(vault.vault, &wallet, rpc_url).await?;
    if balance == 0 {
        let output = json!({
            "ok": false,
            "error": format!("No {} shares found in wallet {}", vault.symbol, wallet),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    // Determine how many shares to withdraw
    let shares_to_withdraw: u128 = if withdraw_all {
        balance
    } else if let Some(sh) = shares_human {
        let shares_wei = (sh * 1e18) as u128;
        if shares_wei > balance {
            anyhow::bail!(
                "Requested {} {} shares but wallet only has {} ({})",
                sh,
                vault.symbol,
                balance,
                balance as f64 / 1e18
            );
        }
        shares_wei
    } else {
        anyhow::bail!("Specify --shares <amount> or --all");
    };

    // Estimate expected output
    let rate = get_rate_in_quote(vault.accountant, vault.deposit_token, rpc_url)
        .await
        .unwrap_or(0);
    let decimals = vault.deposit_token_decimals as i32;
    let expected_out = if rate > 0 {
        (shares_to_withdraw as f64 * rate as f64 / 1e18) / 10f64.powi(decimals)
    } else {
        0.0
    };

    eprintln!(
        "[info] Withdrawing {} {} shares (~{:.8} {}), ask user to confirm before proceeding...",
        shares_to_withdraw as f64 / 1e18,
        vault.symbol,
        expected_out,
        vault.deposit_token_symbol
    );

    // Build calldata and execute
    let calldata = build_bulk_withdraw_calldata(vault.deposit_token, shares_to_withdraw, 0, &wallet);

    let result = wallet_contract_call(
        chain_id,
        vault.teller,
        &calldata,
        Some(&wallet),
        None,
        false,
    )
    .await?;
    let tx_hash = extract_tx_hash(&result);

    let output = json!({
        "ok": true,
        "vault": vault.symbol,
        "name": vault.name,
        "withdraw_token": vault.deposit_token_symbol,
        "shares_withdrawn": shares_to_withdraw.to_string(),
        "shares_human": shares_to_withdraw as f64 / 1e18,
        "expected_out_human": expected_out,
        "wallet": wallet,
        "tx_hash": tx_hash,
        "teller": vault.teller,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
