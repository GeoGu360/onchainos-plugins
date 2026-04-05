/// collateral — query collateral deposits for a given account ID
use anyhow::Result;
use serde::Serialize;

use crate::config::{BASE_RPC_URL, CORE_PROXY, SUSDC, SUSDC_DECIMALS, WETH};
use crate::rpc::{decode_uint256_as_u128, decode_uint256_triple, eth_call, format_amount};

// Verified selectors (cast sig):
// getAccountCollateral(uint128,address)          → 0xef45148e
// getAccountAvailableCollateral(uint128,address) → 0x927482ff

#[derive(Serialize)]
pub struct CollateralInfo {
    pub token: String,
    pub address: String,
    pub total_deposited: f64,
    pub total_assigned: f64,
    pub total_locked: f64,
    pub available_to_withdraw: f64,
}

pub async fn execute(account_id: u128) -> Result<()> {
    let id_hex = format!("{:064x}", account_id);

    let tokens = vec![
        ("sUSDC", SUSDC, SUSDC_DECIMALS),
        ("WETH", WETH, 18),
    ];

    let mut collaterals: Vec<CollateralInfo> = Vec::new();

    for (symbol, addr, decimals) in &tokens {
        let addr_padded = format!("{:0>64}", &addr[2..]);

        // getAccountCollateral(uint128,address) → (uint256 totalDeposited, uint256 totalAssigned, uint256 totalLocked)
        let calldata = format!("0xef45148e{}{}", id_hex, addr_padded);
        let raw = eth_call(CORE_PROXY, &calldata, BASE_RPC_URL).await.unwrap_or_default();
        let (total_deposited, total_assigned, total_locked) = decode_uint256_triple(&raw);

        // getAccountAvailableCollateral(uint128,address) → uint256
        let avail_calldata = format!("0x927482ff{}{}", id_hex, addr_padded);
        let avail_raw = eth_call(CORE_PROXY, &avail_calldata, BASE_RPC_URL)
            .await
            .unwrap_or_default();
        let available = decode_uint256_as_u128(&avail_raw);

        if total_deposited > 0 || available > 0 {
            collaterals.push(CollateralInfo {
                token: symbol.to_string(),
                address: addr.to_string(),
                total_deposited: format_amount(total_deposited, *decimals),
                total_assigned: format_amount(total_assigned, *decimals),
                total_locked: format_amount(total_locked, *decimals),
                available_to_withdraw: format_amount(available, *decimals),
            });
        }
    }

    let output = serde_json::json!({
        "ok": true,
        "chain": 8453,
        "account_id": account_id.to_string(),
        "collaterals": collaterals
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
