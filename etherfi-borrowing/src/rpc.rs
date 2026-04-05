/// Direct eth_call utilities for Scroll — no wallet required

use anyhow::Result;
use serde_json::{json, Value};

/// Execute a raw eth_call and return the result hex string
pub async fn eth_call(to: &str, data: &str, rpc_url: &str) -> Result<String> {
    let client = build_client()?;
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [{"to": to, "data": data}, "latest"],
        "id": 1
    });
    let resp: Value = client
        .post(rpc_url)
        .json(&payload)
        .send()
        .await?
        .json()
        .await?;
    if let Some(err) = resp.get("error") {
        anyhow::bail!("eth_call error: {}", err);
    }
    Ok(resp["result"].as_str().unwrap_or("0x").to_string())
}

/// Build reqwest client with proxy support
pub fn build_client() -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder();
    if let Ok(proxy_url) = std::env::var("HTTPS_PROXY").or_else(|_| std::env::var("https_proxy")) {
        builder = builder.proxy(reqwest::Proxy::all(&proxy_url)?);
    } else if let Ok(proxy_url) = std::env::var("HTTP_PROXY").or_else(|_| std::env::var("http_proxy")) {
        builder = builder.proxy(reqwest::Proxy::all(&proxy_url)?);
    }
    Ok(builder.build()?)
}

/// Decode hex uint256 to u128 (safe for amounts <= u128::MAX)
pub fn decode_uint256_to_u128(hex: &str) -> u128 {
    let hex = hex.trim_start_matches("0x");
    if hex.is_empty() { return 0; }
    if hex.len() > 32 {
        let trimmed = &hex[hex.len() - 32..];
        return u128::from_str_radix(trimmed, 16).unwrap_or(0);
    }
    u128::from_str_radix(hex, 16).unwrap_or(0)
}

/// Pad an address to 32 bytes (64 hex chars, no 0x prefix)
pub fn pad_address(addr: &str) -> String {
    let clean = addr.trim_start_matches("0x").to_lowercase();
    format!("{:0>64}", clean)
}

/// Decode a 32-byte word as bool (last byte != 0)
pub fn decode_bool(hex: &str) -> bool {
    let h = hex.trim_start_matches("0x");
    if h.len() < 2 { return false; }
    let last_byte = &h[h.len()-2..];
    last_byte != "00"
}

/// Read ERC-20 balanceOf
/// selector: 0x70a08231
pub async fn erc20_balance_of(token: &str, holder: &str, rpc_url: &str) -> Result<u128> {
    let data = format!("0x70a08231{}", pad_address(holder));
    let result = eth_call(token, &data, rpc_url).await?;
    Ok(decode_uint256_to_u128(&result))
}

/// Read ERC-20 allowance
/// selector: 0xdd62ed3e
pub async fn erc20_allowance(token: &str, owner: &str, spender: &str, rpc_url: &str) -> Result<u128> {
    let data = format!("0xdd62ed3e{}{}", pad_address(owner), pad_address(spender));
    let result = eth_call(token, &data, rpc_url).await?;
    Ok(decode_uint256_to_u128(&result))
}

/// Query DebtManager.getBorrowTokens()
/// selector: 0x5a52477a
pub async fn get_borrow_tokens(debt_manager: &str, rpc_url: &str) -> Result<Vec<String>> {
    let result = eth_call(debt_manager, "0x5a52477a", rpc_url).await?;
    decode_address_array(&result)
}

/// Query DebtManager.getCollateralTokens()
/// selector: 0xb58eb63f
pub async fn get_collateral_tokens(debt_manager: &str, rpc_url: &str) -> Result<Vec<String>> {
    let result = eth_call(debt_manager, "0xb58eb63f", rpc_url).await?;
    decode_address_array(&result)
}

/// Query DebtManager.borrowApyPerSecond(borrowToken) -> uint64
/// selector: 0x944e2f5e
pub async fn borrow_apy_per_second(debt_manager: &str, token: &str, rpc_url: &str) -> Result<u64> {
    let data = format!("0x944e2f5e{}", pad_address(token));
    let result = eth_call(debt_manager, &data, rpc_url).await?;
    Ok(decode_uint256_to_u128(&result) as u64)
}

/// Query DebtManager.totalSupplies(borrowToken) -> uint256
/// selector: 0x9782e821
pub async fn total_supplies(debt_manager: &str, token: &str, rpc_url: &str) -> Result<u128> {
    let data = format!("0x9782e821{}", pad_address(token));
    let result = eth_call(debt_manager, &data, rpc_url).await?;
    Ok(decode_uint256_to_u128(&result))
}

/// Query DebtManager.totalBorrowingAmount(borrowToken) -> uint256
/// selector: 0xc94f8d42
pub async fn total_borrowing_amount(debt_manager: &str, token: &str, rpc_url: &str) -> Result<u128> {
    let data = format!("0xc94f8d42{}", pad_address(token));
    let result = eth_call(debt_manager, &data, rpc_url).await?;
    Ok(decode_uint256_to_u128(&result))
}

/// Query DebtManager.collateralTokenConfig(token) -> (uint80 ltv, uint80 liquidationThreshold, uint96 liquidationBonus)
/// selector: 0xf0ba097e
pub async fn collateral_token_config(debt_manager: &str, token: &str, rpc_url: &str) -> Result<(u128, u128, u128)> {
    let data = format!("0xf0ba097e{}", pad_address(token));
    let result = eth_call(debt_manager, &data, rpc_url).await?;
    let hex = result.trim_start_matches("0x");
    if hex.len() < 192 {
        return Ok((0, 0, 0));
    }
    let ltv = u128::from_str_radix(&hex[..64], 16).unwrap_or(0);
    let liq_threshold = u128::from_str_radix(&hex[64..128], 16).unwrap_or(0);
    let liq_bonus = u128::from_str_radix(&hex[128..192], 16).unwrap_or(0);
    Ok((ltv, liq_threshold, liq_bonus))
}

/// Query DebtManager.borrowingOf(user, borrowToken) -> uint256
/// selector: 0x4142152e
pub async fn borrowing_of(debt_manager: &str, user: &str, token: &str, rpc_url: &str) -> Result<u128> {
    let data = format!("0x4142152e{}{}", pad_address(user), pad_address(token));
    let result = eth_call(debt_manager, &data, rpc_url).await?;
    Ok(decode_uint256_to_u128(&result))
}

/// Query DebtManager.remainingBorrowingCapacityInUSD(user) -> uint256
/// selector: 0xf6513bfe
pub async fn remaining_borrowing_capacity(debt_manager: &str, user: &str, rpc_url: &str) -> Result<u128> {
    let data = format!("0xf6513bfe{}", pad_address(user));
    let result = eth_call(debt_manager, &data, rpc_url).await?;
    Ok(decode_uint256_to_u128(&result))
}

/// Query DebtManager.liquidatable(user) -> bool
/// selector: 0xffec70af
pub async fn liquidatable(debt_manager: &str, user: &str, rpc_url: &str) -> Result<bool> {
    let data = format!("0xffec70af{}", pad_address(user));
    let result = eth_call(debt_manager, &data, rpc_url).await?;
    Ok(decode_bool(&result))
}

/// Query DebtManager.supplierBalance(supplier, borrowToken) -> uint256
/// selector: 0x58061652
pub async fn supplier_balance(debt_manager: &str, supplier: &str, token: &str, rpc_url: &str) -> Result<u128> {
    let data = format!("0x58061652{}{}", pad_address(supplier), pad_address(token));
    let result = eth_call(debt_manager, &data, rpc_url).await?;
    Ok(decode_uint256_to_u128(&result))
}

/// Decode ABI-encoded address[] from eth_call result
/// Format: offset(32) + length(32) + [address*32 each]
fn decode_address_array(hex: &str) -> Result<Vec<String>> {
    let hex = hex.trim_start_matches("0x");
    if hex.len() < 128 {
        return Ok(vec![]);
    }
    // Word 0: offset to array data (should be 0x20 = 32)
    // Word 1: length of array
    let length = u128::from_str_radix(&hex[64..128], 16).unwrap_or(0) as usize;
    let mut addrs = Vec::new();
    for i in 0..length {
        let start = 128 + i * 64;
        let end = start + 64;
        if end > hex.len() { break; }
        let addr_hex = &hex[start..end];
        // Address is last 20 bytes (40 chars)
        let addr = format!("0x{}", &addr_hex[24..]);
        addrs.push(addr);
    }
    Ok(addrs)
}
