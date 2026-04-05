/// withdraw-collateral — withdraw available sUSDC from Synthetix V3 Core
use anyhow::Result;

use crate::config::{CORE_PROXY, SUSDC, SUSDC_DECIMALS, BASE_RPC_URL};
use crate::onchainos::{extract_tx_hash, resolve_wallet, wallet_contract_call};
use crate::rpc::{decode_uint256_as_u128, eth_call, format_amount};

// Verified selectors (cast sig):
// withdraw(uint128,address,uint256)              → 0x95997c51
// getAccountAvailableCollateral(uint128,address) → 0x927482ff

pub async fn execute(
    account_id: u128,
    amount: f64,
    from: Option<String>,
    dry_run: bool,
    chain_id: u64,
) -> Result<()> {
    let amount_raw = (amount * 10f64.powi(SUSDC_DECIMALS as i32)) as u128;

    if dry_run {
        let calldata = build_withdraw_calldata(account_id, SUSDC, amount_raw);
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "dry_run": true,
                "action": "withdraw-collateral",
                "account_id": account_id,
                "collateral": "sUSDC",
                "amount": amount,
                "amount_raw": amount_raw.to_string(),
                "calldata": calldata,
                "data": {
                    "txHash": "0x0000000000000000000000000000000000000000000000000000000000000000"
                }
            })
        );
        return Ok(());
    }

    let wallet = resolve_wallet(chain_id)?;
    if wallet.is_empty() {
        anyhow::bail!("Cannot resolve wallet address. Ensure onchainos is logged in.");
    }

    // Check available collateral before withdrawing
    let id_hex = format!("{:064x}", account_id);
    let addr_padded = format!("{:0>64}", &SUSDC[2..]);
    let avail_calldata = format!("0x927482ff{}{}", id_hex, addr_padded);
    let avail_raw = eth_call(CORE_PROXY, &avail_calldata, BASE_RPC_URL).await?;
    let available_raw = decode_uint256_as_u128(&avail_raw);
    let available = format_amount(available_raw, SUSDC_DECIMALS);

    if amount_raw > available_raw {
        anyhow::bail!(
            "Insufficient available collateral. Requested: {} sUSDC, Available: {} sUSDC",
            amount,
            available
        );
    }

    let from_addr = from.as_deref().unwrap_or(&wallet);
    let calldata = build_withdraw_calldata(account_id, SUSDC, amount_raw);
    let result = wallet_contract_call(chain_id, CORE_PROXY, &calldata, Some(from_addr), None, false)?;
    let tx_hash = extract_tx_hash(&result);

    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "action": "withdraw-collateral",
            "account_id": account_id,
            "collateral": "sUSDC",
            "amount": amount,
            "tx_hash": tx_hash,
            "explorer": format!("https://basescan.org/tx/{}", tx_hash)
        })
    );
    Ok(())
}

/// Build withdraw(uint128 accountId, address collateralType, uint256 tokenAmount) calldata
/// selector: 0x95997c51 (cast-verified)
fn build_withdraw_calldata(account_id: u128, collateral: &str, amount_raw: u128) -> String {
    let id_hex = format!("{:064x}", account_id);
    let addr_padded = format!("{:0>64}", &collateral[2..]);
    let amount_hex = format!("{:064x}", amount_raw);
    format!("0x95997c51{}{}{}", id_hex, addr_padded, amount_hex)
}
