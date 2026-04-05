/// Direct eth_call helpers — no onchainos involved for reads
use anyhow::Result;
use serde_json::{json, Value};

use crate::config::BASE_RPC_URL;

/// Generic eth_call
pub async fn eth_call(to: &str, data: &str, rpc_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_call",
        "params": [
            {"to": to, "data": data},
            "latest"
        ]
    });
    let resp: Value = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    if let Some(err) = resp.get("error") {
        anyhow::bail!("eth_call error: {}", err);
    }
    Ok(resp["result"].as_str().unwrap_or("0x").to_string())
}

/// Decode uint256 from eth_call result (32 bytes hex -> u128)
pub fn decode_uint128(hex: &str) -> u128 {
    let clean = hex.trim_start_matches("0x");
    u128::from_str_radix(clean.trim_start_matches('0').get(..32).unwrap_or(clean), 16)
        .or_else(|_| u128::from_str_radix(clean, 16))
        .unwrap_or(0)
}

/// Decode uint256 from eth_call result as u128 (last 32 hex chars)
pub fn decode_uint256_as_u128(hex: &str) -> u128 {
    let clean = hex.trim_start_matches("0x");
    // take last 32 hex chars (16 bytes) to avoid overflow for large values
    let len = clean.len();
    let start = if len > 32 { len - 32 } else { 0 };
    u128::from_str_radix(&clean[start..], 16).unwrap_or(0)
}

/// Decode int256 from eth_call (signed, returns as i128 for display)
pub fn decode_int256(hex: &str) -> i128 {
    let clean = hex.trim_start_matches("0x");
    // Check sign bit
    if clean.len() == 64 {
        let high = u8::from_str_radix(&clean[..2], 16).unwrap_or(0);
        if high >= 0x80 {
            // Negative: compute two's complement
            let val = u128::from_str_radix(&clean[32..], 16).unwrap_or(0);
            // For display purposes, treat as signed
            let full = i128::from_str_radix(&clean[32..], 16).unwrap_or(0);
            // If the upper half is all f's, it's a small negative number
            let upper = &clean[..32];
            if upper.chars().all(|c| c == 'f' || c == 'F') {
                return -((!full).wrapping_add(1));
            }
            return full;
        }
    }
    i128::from_str_radix(clean.trim_start_matches('0'), 16).unwrap_or(0)
}

/// Decode tuple of 3 uint256 values (e.g. getAccountCollateral returns totalDeposited, totalAssigned, totalLocked)
pub fn decode_uint256_triple(hex: &str) -> (u128, u128, u128) {
    let clean = hex.trim_start_matches("0x");
    if clean.len() < 192 {
        return (0, 0, 0);
    }
    let a = u128::from_str_radix(&clean[32..64], 16).unwrap_or(0);
    let b = u128::from_str_radix(&clean[96..128], 16).unwrap_or(0);
    let c = u128::from_str_radix(&clean[160..192], 16).unwrap_or(0);
    (a, b, c)
}

/// Decode array of uint256 from eth_call result (for getMarkets / getAccountOpenPositions)
pub fn decode_uint256_array(hex: &str) -> Vec<u128> {
    let clean = hex.trim_start_matches("0x");
    if clean.len() < 128 {
        return vec![];
    }
    // ABI encoding: offset (32 bytes) + length (32 bytes) + elements
    let offset_bytes = 64; // skip offset pointer
    let length = usize::from_str_radix(&clean[offset_bytes..offset_bytes + 64], 16).unwrap_or(0);
    let mut result = Vec::new();
    for i in 0..length {
        let start = offset_bytes + 64 + i * 64;
        let end = start + 64;
        if end > clean.len() {
            break;
        }
        if let Ok(val) = u128::from_str_radix(&clean[start..end], 16) {
            result.push(val);
        }
    }
    result
}

/// ERC-20 balanceOf
/// selector: 0x70a08231 (balanceOf(address))
pub async fn erc20_balance_of(token: &str, owner: &str) -> Result<u128> {
    let owner_padded = format!("{:0>64}", &owner[2..]);
    let calldata = format!("0x70a08231{}", owner_padded);
    let result = eth_call(token, &calldata, BASE_RPC_URL).await?;
    Ok(decode_uint256_as_u128(&result))
}

/// Format token amount with decimals
pub fn format_amount(raw: u128, decimals: u32) -> f64 {
    raw as f64 / 10f64.powi(decimals as i32)
}
