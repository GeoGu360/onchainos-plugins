/// deposit — deposit tokens into an ether.fi Liquid vault
///
/// Flow:
///   1. Check ERC-20 allowance on deposit token for Teller
///   2. If insufficient: ERC-20 approve(teller, amount) — ask user to confirm
///   3. Teller.deposit(depositAsset, amount, minSharesOut=0, receiver=wallet) — ask user to confirm
///
/// Default vault: ETH Yield Vault (LIQUIDETH); deposit token: weETH
/// Amount in human-readable units (e.g. "0.00005" for 0.00005 weETH)

use anyhow::Result;
use serde_json::json;

use crate::config::{ETH_VAULT_TELLER, WEETH_ADDR, VAULTS};
use crate::onchainos::{
    build_approve_calldata, build_deposit_calldata, erc20_approve, extract_tx_hash,
    resolve_wallet, wallet_contract_call,
};
use crate::rpc::erc20_allowance;

pub async fn execute(
    vault_symbol: &str,
    token_symbol: &str,
    amount_human: f64,
    chain_id: u64,
    rpc_url: &str,
    dry_run: bool,
) -> Result<()> {
    // Resolve vault config
    let vault = VAULTS
        .iter()
        .find(|v| v.symbol.eq_ignore_ascii_case(vault_symbol))
        .ok_or_else(|| anyhow::anyhow!("Unknown vault symbol: {}. Use LIQUIDETH, LIQUIDUSD, or LIQUIDBTC", vault_symbol))?;

    // Resolve deposit token: default to vault's primary token
    let (deposit_token_addr, token_decimals) = if token_symbol.is_empty()
        || token_symbol.eq_ignore_ascii_case(vault.deposit_token_symbol)
    {
        (vault.deposit_token, vault.deposit_token_decimals)
    } else {
        // Support WETH for ETH vault
        match token_symbol.to_uppercase().as_str() {
            "WETH" => ("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", 18u8),
            "WEETH" => (WEETH_ADDR, 18u8),
            other => anyhow::bail!("Unsupported deposit token: {}. Use {} or WETH for ETH vault.", other, vault.deposit_token_symbol),
        }
    };

    // Convert human amount to wei
    let amount_wei: u128 = (amount_human * 10f64.powi(token_decimals as i32)) as u128;
    if amount_wei == 0 {
        anyhow::bail!("Amount too small: {} {} = 0 wei", amount_human, token_symbol);
    }

    // dry_run: show calldata without executing
    if dry_run {
        // Use zero address as placeholder receiver
        let receiver_placeholder = "0x0000000000000000000000000000000000000000";
        let approve_calldata = build_approve_calldata(vault.teller, amount_wei);
        let deposit_calldata = build_deposit_calldata(
            deposit_token_addr,
            amount_wei,
            0,
            receiver_placeholder,
        );
        let output = json!({
            "ok": true,
            "dry_run": true,
            "vault": vault.symbol,
            "deposit_token": token_symbol,
            "amount_human": amount_human,
            "amount_wei": amount_wei.to_string(),
            "approve_calldata": approve_calldata,
            "deposit_calldata": deposit_calldata,
            "teller": vault.teller,
            "deposit_token_addr": deposit_token_addr,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    // Resolve wallet (after dry_run guard)
    let wallet = resolve_wallet(chain_id)?;

    // Step 1: Check allowance
    let allowance = erc20_allowance(deposit_token_addr, &wallet, vault.teller, rpc_url).await?;
    eprintln!(
        "[info] Current allowance: {} wei; needed: {} wei",
        allowance, amount_wei
    );

    // Step 2: Approve if needed
    let mut approve_tx_hash = String::new();
    if allowance < amount_wei {
        eprintln!("[info] Approving {} {} to Teller {}...", amount_human, token_symbol, vault.teller);
        // Ask user to confirm is documented in SKILL.md — here we proceed after dry_run check
        let approve_result = erc20_approve(
            chain_id,
            deposit_token_addr,
            vault.teller,
            amount_wei,
            Some(&wallet),
            false,
        )
        .await?;
        approve_tx_hash = extract_tx_hash(&approve_result);
        eprintln!("[info] Approve txHash: {}", approve_tx_hash);
    } else {
        eprintln!("[info] Allowance sufficient, skipping approve");
    }

    // Step 3: Deposit
    eprintln!("[info] Depositing {} {} into {}...", amount_human, token_symbol, vault.symbol);
    let deposit_calldata = build_deposit_calldata(deposit_token_addr, amount_wei, 0, &wallet);
    let deposit_result = wallet_contract_call(
        chain_id,
        vault.teller,
        &deposit_calldata,
        Some(&wallet),
        None,
        false,
    )
    .await?;
    let deposit_tx_hash = extract_tx_hash(&deposit_result);

    let output = json!({
        "ok": true,
        "vault": vault.symbol,
        "name": vault.name,
        "deposit_token": token_symbol,
        "amount_human": amount_human,
        "amount_wei": amount_wei.to_string(),
        "wallet": wallet,
        "approve_tx_hash": approve_tx_hash,
        "deposit_tx_hash": deposit_tx_hash,
        "teller": vault.teller,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
