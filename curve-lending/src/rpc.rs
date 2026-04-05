/// ABI encoding and direct eth_call helpers for Curve Lending

use serde_json::Value;

// ── ABI encoding helpers ──────────────────────────────────────────────────────

/// Pad an address (with or without 0x) to 64 hex chars (32-byte word).
pub fn encode_address(addr: &str) -> String {
    let addr = addr.trim_start_matches("0x").trim_start_matches("0X");
    format!("{:0>64}", addr)
}

/// Encode a u128 as a 32-byte big-endian hex word (no 0x prefix).
pub fn encode_u256(val: u128) -> String {
    format!("{:064x}", val)
}

/// Encode a u64 as a 32-byte big-endian hex word (no 0x prefix).
pub fn encode_u64(val: u64) -> String {
    format!("{:064x}", val)
}

/// Encode an i256 (signed) as 32-byte two's-complement hex word.
/// For positive values (e.g., max_active_band = i256::MAX), this is just the value.
pub fn encode_i256_max() -> String {
    // 2^255 - 1 = 7fff...ffff
    "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string()
}

/// Encode bool as 32-byte word (false = 0, true = 1).
pub fn encode_bool(val: bool) -> String {
    if val {
        "0000000000000000000000000000000000000000000000000000000000000001".to_string()
    } else {
        "0000000000000000000000000000000000000000000000000000000000000000".to_string()
    }
}

// ── Calldata builders ─────────────────────────────────────────────────────────

/// `approve(address spender, uint256 amount)` — ERC-20 approve
pub fn calldata_approve(spender: &str, amount: u128) -> String {
    format!("0x095ea7b3{}{}", encode_address(spender), encode_u256(amount))
}

/// `create_loan(uint256 collateral, uint256 debt, uint256 N)`
pub fn calldata_create_loan(collateral: u128, debt: u128, n: u64) -> String {
    format!(
        "0x23cfed03{}{}{}",
        encode_u256(collateral),
        encode_u256(debt),
        encode_u64(n),
    )
}

/// `add_collateral(uint256 collateral, address _for)`
pub fn calldata_add_collateral(collateral: u128, for_addr: &str) -> String {
    format!(
        "0x24049e57{}{}",
        encode_u256(collateral),
        encode_address(for_addr),
    )
}

/// `borrow_more(uint256 collateral, uint256 debt)`
pub fn calldata_borrow_more(collateral: u128, debt: u128) -> String {
    format!(
        "0xdd171e7c{}{}",
        encode_u256(collateral),
        encode_u256(debt),
    )
}

/// `repay(uint256 _d_debt)` — simplest 1-param variant (verified live on chain)
pub fn calldata_repay(d_debt: u128) -> String {
    format!("0x371fd8e6{}", encode_u256(d_debt))
}

/// `loan_exists(address)` → single address param
pub fn calldata_loan_exists(addr: &str) -> String {
    format!("0xa21adb9e{}", encode_address(addr))
}

/// `debt(address)`
pub fn calldata_debt(addr: &str) -> String {
    format!("0x9b6c56ec{}", encode_address(addr))
}

/// `user_state(address)`
pub fn calldata_user_state(addr: &str) -> String {
    format!("0xec74d0a8{}", encode_address(addr))
}

/// `user_prices(address)`
pub fn calldata_user_prices(addr: &str) -> String {
    format!("0x2c5089c3{}", encode_address(addr))
}

/// `health(address user, bool full)` — full=false for basic health
pub fn calldata_health(addr: &str, full: bool) -> String {
    format!("0x8908ea82{}{}", encode_address(addr), encode_bool(full))
}

/// `max_borrowable(uint256 collateral, uint256 N)`
pub fn calldata_max_borrowable(collateral: u128, n: u64) -> String {
    format!(
        "0x9a497196{}{}",
        encode_u256(collateral),
        encode_u64(n),
    )
}

/// Factory: `names(uint256 index)` — returns string
pub fn calldata_factory_names(index: u64) -> String {
    format!("0x4622ab03{}", encode_u64(index))
}

/// Factory: `controllers(uint256 index)` — returns address
pub fn calldata_factory_controllers(index: u64) -> String {
    format!("0xe94b0dd2{}", encode_u64(index))
}

/// Factory: `vaults(uint256 index)` — returns address
pub fn calldata_factory_vaults(index: u64) -> String {
    format!("0x8c64ea4a{}", encode_u64(index))
}

/// Factory: `collateral_tokens(uint256 index)` — returns address
pub fn calldata_factory_collateral_tokens(index: u64) -> String {
    format!("0x49b89984{}", encode_u64(index))
}

/// Factory: `borrowed_tokens(uint256 index)` — returns address
pub fn calldata_factory_borrowed_tokens(index: u64) -> String {
    format!("0x6fe4501f{}", encode_u64(index))
}

/// Factory: `monetary_policies(uint256 index)` — returns address
pub fn calldata_factory_monetary_policies(index: u64) -> String {
    format!("0x762e7b92{}", encode_u64(index))
}

/// MonetaryPolicy: `rate(address controller)` → per-second rate (1e18 scaled)
pub fn calldata_mp_rate(controller: &str) -> String {
    format!("0x0ba9d8ca{}", encode_address(controller))
}

/// `symbol()` on ERC-20
pub fn calldata_symbol() -> &'static str {
    "0x95d89b41"
}

/// `decimals()` on ERC-20
pub fn calldata_decimals() -> &'static str {
    "0x313ce567"
}

/// `totalAssets()` on ERC-4626 vault
pub fn calldata_total_assets() -> &'static str {
    "0x01e1d114"
}

/// `n_loans()` on Controller
pub fn calldata_n_loans() -> &'static str {
    "0x6cce39be"
}

/// `total_debt()` on Controller
pub fn calldata_total_debt() -> &'static str {
    "0x31dc3ca8"
}

// ── ABI decoding helpers ──────────────────────────────────────────────────────

/// Decode a single uint256 from 32-byte (64 hex-char) ABI return data.
pub fn decode_uint256(hex: &str) -> u128 {
    let hex = hex.trim().trim_start_matches("0x");
    if hex.len() < 64 {
        return 0;
    }
    u128::from_str_radix(&hex[hex.len() - 64..], 16).unwrap_or(0)
}

/// Decode a single int256 as i128 (safe for health factor which is typically small).
pub fn decode_int256_as_i128(hex: &str) -> i128 {
    let hex = hex.trim().trim_start_matches("0x");
    if hex.len() < 64 {
        return 0;
    }
    let word = &hex[hex.len() - 64..];
    // Check sign bit
    let high_nibble = u8::from_str_radix(&word[0..1], 16).unwrap_or(0);
    if high_nibble >= 8 {
        // Negative: two's complement
        let val = u128::from_str_radix(word, 16).unwrap_or(0);
        // Convert from two's complement u128 to negative i128
        let val_i = val as i128;
        val_i // Already correct as i128 since it's signed 256-bit
    } else {
        u128::from_str_radix(word, 16).unwrap_or(0) as i128
    }
}

/// Decode an ABI-encoded string from return data.
/// Layout: offset(32) | length(32) | UTF-8 bytes padded to 32
pub fn decode_string(hex: &str) -> String {
    let hex = hex.trim().trim_start_matches("0x");
    if hex.len() < 128 {
        return String::new();
    }
    // offset at [0..64], length at [64..128]
    let length = usize::from_str_radix(&hex[64..128], 16).unwrap_or(0);
    if length == 0 || hex.len() < 128 + length * 2 {
        return String::new();
    }
    let str_hex = &hex[128..128 + length * 2];
    hex::decode(str_hex)
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
        .unwrap_or_default()
}

/// Decode an address from 32-byte ABI-encoded return data (take last 20 bytes).
pub fn decode_address(hex: &str) -> String {
    let hex = hex.trim().trim_start_matches("0x");
    if hex.len() < 64 {
        return String::new();
    }
    format!("0x{}", &hex[hex.len() - 40..])
}

/// Decode user_state(address) return for LlamaLend Lending markets:
/// (collateral uint256, stablecoin uint256, debt uint256, N uint256)
/// Note: LlamaLend lending market Controller returns 4 words (not 6 like crvUSD mint markets).
/// Returns (collateral, stablecoin, debt, n_bands, n_bands) — duplicate n for band range display.
pub fn decode_user_state(hex: &str) -> Option<(u128, u128, u128, i64, i64)> {
    let hex = hex.trim().trim_start_matches("0x");
    if hex.len() < 64 * 3 {
        return None;
    }
    let collateral = u128::from_str_radix(&hex[0..64], 16).unwrap_or(0);
    let stablecoin = u128::from_str_radix(&hex[64..128], 16).unwrap_or(0);
    let debt = u128::from_str_radix(&hex[128..192], 16).unwrap_or(0);
    // 4th word = N (number of bands) if present
    let n = if hex.len() >= 64 * 4 {
        u128::from_str_radix(&hex[192..256], 16).unwrap_or(0) as i64
    } else {
        0
    };
    Some((collateral, stablecoin, debt, n, n))
}

/// Decode user_prices(address) return: (price_high uint256, price_low uint256)
pub fn decode_user_prices(hex: &str) -> Option<(u128, u128)> {
    let hex = hex.trim().trim_start_matches("0x");
    if hex.len() < 128 {
        return None;
    }
    let high = u128::from_str_radix(&hex[0..64], 16).unwrap_or(0);
    let low = u128::from_str_radix(&hex[64..128], 16).unwrap_or(0);
    Some((high, low))
}

// ── Direct eth_call ───────────────────────────────────────────────────────────

/// Synchronous direct eth_call via JSON-RPC.
/// Returns the raw hex result string (e.g., "0x000...").
pub fn eth_call_raw(rpc_url: &str, to: &str, data: &str) -> anyhow::Result<String> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [
            { "to": to, "data": data },
            "latest"
        ],
        "id": 1
    });
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let resp: Value = client.post(rpc_url).json(&body).send()?.json()?;
    if let Some(err) = resp.get("error") {
        anyhow::bail!("eth_call error on {to}: {err}");
    }
    Ok(resp["result"].as_str().unwrap_or("0x").to_string())
}

/// Convenience: eth_call and decode as u128.
pub fn eth_call_uint256(rpc: &str, to: &str, data: &str) -> anyhow::Result<u128> {
    let raw = eth_call_raw(rpc, to, data)?;
    Ok(decode_uint256(&raw))
}

/// Convenience: eth_call and decode as address string.
pub fn eth_call_address(rpc: &str, to: &str, data: &str) -> anyhow::Result<String> {
    let raw = eth_call_raw(rpc, to, data)?;
    Ok(decode_address(&raw))
}

/// Convenience: eth_call and decode as String.
pub fn eth_call_string(rpc: &str, to: &str, data: &str) -> anyhow::Result<String> {
    let raw = eth_call_raw(rpc, to, data)?;
    Ok(decode_string(&raw))
}
