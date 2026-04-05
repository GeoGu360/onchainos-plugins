use crate::api;
use crate::config::{get_chain_config, resolve_vault_static, ETH_ADDR};
use crate::onchainos;
use crate::rpc;

/// Deposit into a Mellow LRT vault.
///
/// Vault types and deposit paths:
///   - simple-lrt (ERC-4626): approve(vault, amount) → vault.deposit(amount, receiver)
///     selector: 0x6e553f65 (ERC-4626 deposit)
///   - multi-vault/core-vault: approve(vault, amount) → vault.deposit(to, amounts[], minLp, deadline, referral)
///     selector: 0xf379a7d6
///   - ETH/WETH/stETH via EthWrapper (simple-lrt only): EthWrapper.deposit(token, amount, vault, receiver, referral)
///     selector: 0x0bb9f5e1 — ETH: send msg.value
///
/// NOTE: Multi-vault deposits (steakLRT, Re7LRT etc.) require curator/validator authorization.
/// If you get a revert, the vault may restrict public deposits.
///
/// CONFIRM: This is an on-chain write operation. Review amounts before executing.
pub async fn run(
    vault_input: &str,
    token: &str,
    amount: &str,
    chain_id: u64,
    _from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let cfg = get_chain_config(chain_id)?;

    // Resolve vault address and type
    let (vault_addr, base_token_addr, base_decimals, vault_type) =
        resolve_vault(vault_input, chain_id, cfg.rpc_url).await?;

    let token_upper = token.to_uppercase();
    let (deposit_token_addr, deposit_decimals) =
        resolve_deposit_token(&vault_addr, &token_upper, chain_id, cfg.rpc_url).await?;

    let raw_amount = rpc::parse_amount(amount, deposit_decimals)?;

    // dry_run: before resolve_wallet
    if dry_run {
        let calldata = build_deposit_calldata_dry(
            &vault_type,
            &token_upper,
            &deposit_token_addr,
            &vault_addr,
            raw_amount,
            cfg.eth_wrapper,
        );
        let output = serde_json::json!({
            "ok": true,
            "dry_run": true,
            "operation": "deposit",
            "vaultType": vault_type,
            "vault": vault_addr,
            "token": token_upper,
            "amount": amount,
            "rawAmount": raw_amount.to_string(),
            "calldata": calldata,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    let wallet = onchainos::resolve_wallet(chain_id, false)?;
    let wallet_clean = wallet.trim_start_matches("0x").to_lowercase();

    // Validate deposit limits (only for ERC-4626 vaults)
    if vault_type == "simple-lrt" || vault_type == "dvv-vault" {
        let max_dep = rpc::vault_max_deposit(&vault_addr, &wallet, cfg.rpc_url)
            .await
            .unwrap_or(u128::MAX);
        if max_dep < raw_amount {
            anyhow::bail!(
                "Deposit limit exceeded for vault {}. Max: {} (raw: {})",
                vault_input, rpc::format_amount(max_dep, base_decimals), max_dep
            );
        }
    }

    eprintln!(
        "[mellow-lrt] Depositing {} {} into {} vault {} on {}",
        amount, token_upper, vault_type, vault_addr, cfg.name
    );

    let tx_hash = match vault_type.as_str() {
        "simple-lrt" | "dvv-vault" => {
            // ERC-4626 path
            if token_upper == "ETH" {
                // ETH via EthWrapper (only for simple-lrt)
                let calldata = build_wrapper_calldata(ETH_ADDR, raw_amount, &vault_addr, &wallet);
                eprintln!("[mellow-lrt] Depositing ETH via EthWrapper into ERC-4626 vault...");
                let result = onchainos::wallet_contract_call(
                    chain_id, cfg.eth_wrapper, &calldata, Some(raw_amount), false,
                ).await?;
                onchainos::extract_tx_hash(&result)
            } else if token_upper == "WSTETH" {
                // Direct ERC-4626 deposit: approve + deposit(uint256,address)
                eprintln!("[mellow-lrt] Step 1/2: Approving {} to vault...", token_upper);
                let approve_calldata = format!(
                    "0x095ea7b3{:0>64}{:064x}",
                    vault_clean(&vault_addr),
                    raw_amount
                );
                let approve_result = onchainos::wallet_contract_call(
                    chain_id, &base_token_addr, &approve_calldata, None, false,
                ).await?;
                eprintln!("[mellow-lrt] Approve tx: {}", onchainos::extract_tx_hash(&approve_result));
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                // deposit(uint256 assets, address receiver) selector: 0x6e553f65
                let deposit_calldata = format!(
                    "0x6e553f65{:064x}{:0>64}",
                    raw_amount, wallet_clean
                );
                let result = onchainos::wallet_contract_call(
                    chain_id, &vault_addr, &deposit_calldata, None, false,
                ).await?;
                onchainos::extract_tx_hash(&result)
            } else {
                // WETH/stETH via EthWrapper
                eprintln!("[mellow-lrt] Step 1/2: Approving {} to EthWrapper...", token_upper);
                let approve_calldata = format!(
                    "0x095ea7b3{:0>64}{:064x}",
                    vault_clean(cfg.eth_wrapper),
                    raw_amount
                );
                let approve_result = onchainos::wallet_contract_call(
                    chain_id, &deposit_token_addr, &approve_calldata, None, false,
                ).await?;
                eprintln!("[mellow-lrt] Approve tx: {}", onchainos::extract_tx_hash(&approve_result));
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                let calldata = build_wrapper_calldata(&deposit_token_addr, raw_amount, &vault_addr, &wallet);
                let result = onchainos::wallet_contract_call(
                    chain_id, cfg.eth_wrapper, &calldata, None, false,
                ).await?;
                onchainos::extract_tx_hash(&result)
            }
        }
        _ => {
            // multi-vault / core-vault: uses custom deposit(address,uint256[],uint256,uint256,uint256)
            // selector: 0xf379a7d6
            // Requires: 1) approve base token to vault, 2) call vault.deposit with amounts array
            eprintln!("[mellow-lrt] Step 1/2: Approving {} to multi-vault...", token_upper);
            let approve_calldata = format!(
                "0x095ea7b3{:0>64}{:064x}",
                vault_clean(&vault_addr),
                raw_amount
            );
            let approve_result = onchainos::wallet_contract_call(
                chain_id, &base_token_addr, &approve_calldata, None, false,
            ).await?;
            eprintln!("[mellow-lrt] Approve tx: {}", onchainos::extract_tx_hash(&approve_result));
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            // deposit(address to, uint256[] amounts, uint256 minLpAmount, uint256 deadline, uint256 referralCode)
            let calldata = build_multi_vault_deposit_calldata(&wallet, raw_amount);
            eprintln!("[mellow-lrt] Step 2/2: Depositing into multi-vault...");
            let result = onchainos::wallet_contract_call(
                chain_id, &vault_addr, &calldata, None, false,
            ).await?;
            onchainos::extract_tx_hash(&result)
        }
    };

    let output = serde_json::json!({
        "ok": true,
        "operation": "deposit",
        "vaultType": vault_type,
        "vault": vault_addr,
        "vaultSymbol": vault_input,
        "token": token_upper,
        "amount": amount,
        "rawAmount": raw_amount.to_string(),
        "receiver": wallet,
        "chain": cfg.name,
        "chainId": chain_id,
        "txHash": tx_hash,
        "note": "Withdrawal is async: call 'withdraw' to start, then 'claim' after ~14 days."
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Build EthWrapper.deposit calldata (simple-lrt/dvv-vault).
/// deposit(address depositToken, uint256 amount, address vault, address receiver, address referral)
/// selector: 0x0bb9f5e1
fn build_wrapper_calldata(deposit_token: &str, amount: u128, vault: &str, receiver: &str) -> String {
    let token_clean = format!("{:0>64}", deposit_token.trim_start_matches("0x").to_lowercase());
    let amount_hex = format!("{:064x}", amount);
    let vault_pad = vault_clean(vault);
    let receiver_pad = vault_clean(receiver);
    let referral_pad = "0000000000000000000000000000000000000000000000000000000000000000";
    format!("0x0bb9f5e1{}{}{}{}{}", token_clean, amount_hex, vault_pad, receiver_pad, referral_pad)
}

/// Build multi-vault deposit calldata.
/// deposit(address to, uint256[] amounts, uint256 minLpAmount, uint256 deadline, uint256 referralCode)
/// selector: 0xf379a7d6
/// Assumes single underlying token with the given amount.
fn build_multi_vault_deposit_calldata(to: &str, amount: u128) -> String {
    let to_pad = vault_clean(to);
    // amounts[] offset = 5 * 32 = 160 = 0xa0
    let amounts_offset = "00000000000000000000000000000000000000000000000000000000000000a0";
    let min_lp = "0000000000000000000000000000000000000000000000000000000000000000";
    // deadline = now + 1 hour (encoded as timestamp; will be close to current block time)
    let deadline_raw = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() + 3600;
    let deadline = format!("{:064x}", deadline_raw);
    let referral = "0000000000000000000000000000000000000000000000000000000000000000";
    let array_len = "0000000000000000000000000000000000000000000000000000000000000001";
    let amount_pad = format!("{:064x}", amount);
    format!(
        "0xf379a7d6{}{}{}{}{}{}{}",
        to_pad, amounts_offset, min_lp, deadline, referral, array_len, amount_pad
    )
}

/// Build deposit calldata for dry-run display.
fn build_deposit_calldata_dry(
    vault_type: &str,
    token: &str,
    deposit_token_addr: &str,
    vault_addr: &str,
    amount: u128,
    eth_wrapper: &str,
) -> String {
    match vault_type {
        "simple-lrt" | "dvv-vault" => {
            if token == "ETH" {
                build_wrapper_calldata(ETH_ADDR, amount, vault_addr, "0x0000000000000000000000000000000000000000")
            } else if token == "WSTETH" {
                format!(
                    "0x6e553f65{:064x}{}",
                    amount,
                    "0000000000000000000000000000000000000000000000000000000000000000"
                )
            } else {
                let _ = eth_wrapper;
                build_wrapper_calldata(deposit_token_addr, amount, vault_addr, "0x0000000000000000000000000000000000000000")
            }
        }
        _ => {
            // multi-vault path
            build_multi_vault_deposit_calldata(
                "0x0000000000000000000000000000000000000000",
                amount,
            )
        }
    }
}

/// Helper: lowercase and zero-pad an address to 64 hex chars.
fn vault_clean(addr: &str) -> String {
    format!("{:0>64}", addr.trim_start_matches("0x").to_lowercase())
}

/// Resolve vault address, base token, decimals, and vault type.
async fn resolve_vault(input: &str, chain_id: u64, rpc_url: &str)
    -> anyhow::Result<(String, String, u8, String)>
{
    // Try API first for type information
    let vaults = api::fetch_vaults(Some(chain_id)).await.unwrap_or_default();
    let input_lower = input.to_lowercase();

    // Check by symbol or address in API
    for v in &vaults {
        if v.symbol.to_lowercase() == input_lower || v.address.to_lowercase() == input_lower {
            let (base_addr, decimals) = v.base_token.as_ref()
                .map(|t| (t.address.clone(), t.decimals))
                .unwrap_or_else(|| ("0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0".to_string(), 18));
            let vtype = v.vault_type.clone().unwrap_or_else(|| "multi-vault".to_string());
            return Ok((v.address.clone(), base_addr, decimals, vtype));
        }
    }

    // Try static lookup
    if let Some((vault_addr, base_addr, decimals)) = resolve_vault_static(input) {
        let base_final = if base_addr.is_empty() {
            rpc::vault_asset(&vault_addr, rpc_url).await.unwrap_or_default()
        } else {
            base_addr
        };
        let dec_final = if base_final.is_empty() { decimals } else {
            rpc::erc20_decimals(&base_final, rpc_url).await.unwrap_or(decimals)
        };
        return Ok((vault_addr, base_final, dec_final, "multi-vault".to_string()));
    }

    anyhow::bail!(
        "Unknown vault '{}'. Use symbol (e.g. steakLRT, Re7LRT) or address. Run 'mellow-lrt --chain {} vaults'.",
        input, chain_id
    )
}

/// Resolve deposit token address and decimals from token symbol.
async fn resolve_deposit_token(vault_addr: &str, token: &str, chain_id: u64, rpc_url: &str)
    -> anyhow::Result<(String, u8)>
{
    match token {
        "ETH" => Ok(("0x0000000000000000000000000000000000000000".to_string(), 18)),
        "WETH" => Ok(("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(), 18)),
        "STETH" => Ok(("0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84".to_string(), 18)),
        "WSTETH" => Ok(("0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0".to_string(), 18)),
        _ => {
            let vaults = api::fetch_vaults(Some(chain_id)).await.unwrap_or_default();
            for v in &vaults {
                if v.address.to_lowercase() == vault_addr.to_lowercase() {
                    for t in v.deposit_tokens.as_ref().unwrap_or(&vec![]) {
                        if t.symbol.to_uppercase() == token {
                            let dec = rpc::erc20_decimals(&t.address, rpc_url).await.unwrap_or(18);
                            return Ok((t.address.clone(), dec));
                        }
                    }
                }
            }
            anyhow::bail!("Unsupported deposit token '{}'. Supported: ETH, WETH, stETH, wstETH", token)
        }
    }
}
