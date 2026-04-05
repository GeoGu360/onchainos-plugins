/// supply command — supply USDC liquidity to EtherFi Cash debt manager

use anyhow::Result;
use crate::{config, onchainos, rpc};

pub async fn execute(
    amount: f64,
    chain_id: u64,
    rpc_url: &str,
    dry_run: bool,
) -> Result<()> {
    // Resolve wallet after dry-run guard
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
        // Check USDC balance on Scroll
        let balance = rpc::erc20_balance_of(usdc, &wallet, rpc_url).await?;
        if balance < amount_raw {
            anyhow::bail!(
                "Insufficient USDC balance on Scroll. Have: {:.6} USDC, need: {:.6} USDC",
                balance as f64 / 1_000_000.0,
                amount
            );
        }

        // Check allowance
        let allowance = rpc::erc20_allowance(usdc, &wallet, debt_manager, rpc_url).await?;
        if allowance < amount_raw {
            eprintln!("Approving USDC for DebtManager ({:.6} USDC)...", amount);
            eprintln!("Please confirm the approval transaction.");
            let approve_result = onchainos::erc20_approve(
                chain_id,
                usdc,
                debt_manager,
                u128::MAX,
                Some(&wallet),
                dry_run,
            ).await?;
            let approve_hash = onchainos::extract_tx_hash(&approve_result);
            eprintln!("Approve tx: {}", approve_hash);

            // Wait for approval to confirm
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        }
    }

    // Build supply calldata: supply(address user, address borrowToken, uint256 amount)
    // selector: 0x0c0a769b
    let calldata = onchainos::build_supply_calldata(&wallet, usdc, amount_raw);

    eprintln!("Supplying {:.6} USDC to EtherFi Cash on Scroll...", amount);
    eprintln!("Please confirm the supply transaction.");

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
        "action": "supply-liquidity",
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
