/// deposit-collateral — deposit sUSDC as collateral into Synthetix V3 Core
use anyhow::Result;

use crate::config::{CORE_PROXY, SUSDC, SUSDC_DECIMALS};
use crate::onchainos::{erc20_approve, extract_tx_hash, resolve_wallet, wallet_contract_call};

// Verified selectors (cast sig):
// deposit(uint128,address,uint256) → 0x83802968
// approve(address,uint256)         → 0x095ea7b3 (in erc20_approve)

pub async fn execute(
    account_id: u128,
    amount: f64,
    from: Option<String>,
    dry_run: bool,
    chain_id: u64,
) -> Result<()> {
    // Convert to 18-decimal wei for sUSDC
    let amount_raw = (amount * 10f64.powi(SUSDC_DECIMALS as i32)) as u128;

    if dry_run {
        // Build calldata for deposit
        let calldata = build_deposit_calldata(account_id, SUSDC, amount_raw);
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "dry_run": true,
                "action": "deposit-collateral",
                "account_id": account_id,
                "collateral": "sUSDC",
                "amount": amount,
                "amount_raw": amount_raw.to_string(),
                "step1_approve_calldata": format!("approve 0x095ea7b3 on {} spender={}", SUSDC, CORE_PROXY),
                "step2_deposit_calldata": calldata,
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
    let from_addr = from.as_deref().unwrap_or(&wallet);

    // Step 1: Approve sUSDC to CoreProxy
    eprintln!("[1/2] Approving sUSDC...");
    let approve_result = erc20_approve(
        chain_id,
        SUSDC,
        CORE_PROXY,
        amount_raw,
        Some(from_addr),
        false,
    )?;
    let approve_tx = extract_tx_hash(&approve_result);
    eprintln!("Approve tx: {}", approve_tx);

    // Small delay to allow approval to confirm
    std::thread::sleep(std::time::Duration::from_secs(5));

    // Step 2: Deposit
    eprintln!("[2/2] Depositing collateral...");
    let calldata = build_deposit_calldata(account_id, SUSDC, amount_raw);
    let result = wallet_contract_call(chain_id, CORE_PROXY, &calldata, Some(from_addr), None, false)?;
    let tx_hash = extract_tx_hash(&result);

    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "action": "deposit-collateral",
            "account_id": account_id,
            "collateral": "sUSDC",
            "amount": amount,
            "approve_tx": approve_tx,
            "tx_hash": tx_hash,
            "explorer": format!("https://basescan.org/tx/{}", tx_hash)
        })
    );
    Ok(())
}

/// Build deposit(uint128 accountId, address collateralType, uint256 tokenAmount) calldata
/// selector: 0x83802968 (cast-verified)
fn build_deposit_calldata(account_id: u128, collateral: &str, amount_raw: u128) -> String {
    let id_hex = format!("{:064x}", account_id);
    let addr_padded = format!("{:0>64}", &collateral[2..]);
    let amount_hex = format!("{:064x}", amount_raw);
    format!("0x83802968{}{}{}", id_hex, addr_padded, amount_hex)
}
