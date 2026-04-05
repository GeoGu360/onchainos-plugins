/// withdraw command — withdraw USDC liquidity from EtherFi Cash debt manager

use anyhow::Result;
use crate::{config, onchainos, rpc};

pub async fn execute(
    amount: f64,
    chain_id: u64,
    rpc_url: &str,
    dry_run: bool,
) -> Result<()> {
    let wallet = if dry_run {
        "0x0000000000000000000000000000000000000000".to_string()
    } else {
        onchainos::resolve_wallet(chain_id)?
    };

    let usdc = config::USDC_ADDR;
    let debt_manager = config::DEBT_MANAGER;

    // Convert amount to 6-decimal raw units
    let amount_raw = (amount * 1_000_000.0).round() as u128;

    if !dry_run {
        // Check supplier balance
        let balance = rpc::supplier_balance(debt_manager, &wallet, usdc, rpc_url).await?;
        if balance < amount_raw {
            anyhow::bail!(
                "Insufficient supplier balance. Have: {:.6} USDC supplied, need: {:.6} USDC",
                balance as f64 / 1_000_000.0,
                amount
            );
        }
    }

    // Build withdrawBorrowToken calldata
    // withdrawBorrowToken(address borrowToken, uint256 amount)
    // selector: 0xa56c8ff7
    let calldata = onchainos::build_withdraw_calldata(usdc, amount_raw);

    eprintln!("Withdrawing {:.6} USDC liquidity from EtherFi Cash on Scroll...", amount);
    eprintln!("Please confirm the withdrawal transaction.");

    let result = onchainos::wallet_contract_call(
        chain_id,
        debt_manager,
        &calldata,
        Some(&wallet),
        None,
        dry_run,
    ).await?;

    let tx_hash = onchainos::extract_tx_hash(&result);

    let output = serde_json::json!({
        "ok": true,
        "action": "withdraw-liquidity",
        "chain": "Scroll (534352)",
        "debt_manager": debt_manager,
        "token": "USDC",
        "amount": format!("{:.6}", amount),
        "amount_raw": amount_raw.to_string(),
        "wallet": wallet,
        "dry_run": dry_run,
        "tx_hash": tx_hash,
        "calldata": calldata,
        "explorer": if dry_run { "".to_string() } else { format!("https://scrollscan.com/tx/{}", tx_hash) },
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
