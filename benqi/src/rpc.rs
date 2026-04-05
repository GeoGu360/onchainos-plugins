// src/rpc.rs -- Direct eth_call queries for Benqi (Compound V2 fork, Avalanche)
// Benqi uses per-timestamp rates (supplyRatePerTimestamp / borrowRatePerTimestamp)
// instead of Compound V2's per-block rates.

use anyhow::Context;
use serde_json::{json, Value};

/// Low-level eth_call
pub async fn eth_call(to: &str, data: &str, rpc_url: &str) -> anyhow::Result<String> {
    let client = build_client()?;
    let body = json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [
            { "to": to, "data": data },
            "latest"
        ],
        "id": 1
    });
    let resp: Value = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .context("RPC request failed")?
        .json()
        .await
        .context("RPC response parse failed")?;

    if let Some(err) = resp.get("error") {
        anyhow::bail!("RPC error: {}", err);
    }
    Ok(resp["result"]
        .as_str()
        .unwrap_or("0x")
        .to_string())
}

fn build_client() -> anyhow::Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder();
    if let Ok(proxy_url) = std::env::var("HTTPS_PROXY").or_else(|_| std::env::var("https_proxy")) {
        builder = builder.proxy(reqwest::Proxy::https(&proxy_url)?);
    }
    Ok(builder.build()?)
}

/// Parse a uint256 from a 32-byte ABI-encoded hex result (fits in u128)
pub fn parse_u128(hex_result: &str) -> anyhow::Result<u128> {
    let clean = hex_result.trim_start_matches("0x");
    if clean.is_empty() || clean == "0" {
        return Ok(0);
    }
    let trimmed = if clean.len() > 32 { &clean[clean.len() - 32..] } else { clean };
    Ok(u128::from_str_radix(trimmed, 16).context("parse u128 failed")?)
}

/// Pad an address to 32 bytes (ABI encoding)
pub fn pad_address(addr: &str) -> String {
    let clean = addr.trim_start_matches("0x");
    format!("{:0>64}", clean)
}

// -- qiToken read calls --

/// qiToken.supplyRatePerTimestamp() -> u128 (scaled by 1e18)
/// selector: 0xd3bd2c72
pub async fn supply_rate_per_timestamp(qi_token: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let result = eth_call(qi_token, "0xd3bd2c72", rpc_url).await?;
    parse_u128(&result)
}

/// qiToken.borrowRatePerTimestamp() -> u128 (scaled by 1e18)
/// selector: 0xcd91801c
pub async fn borrow_rate_per_timestamp(qi_token: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let result = eth_call(qi_token, "0xcd91801c", rpc_url).await?;
    parse_u128(&result)
}

/// qiToken.exchangeRateCurrent() -> u128
/// selector: 0xbd6d894d
pub async fn exchange_rate_current(qi_token: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let result = eth_call(qi_token, "0xbd6d894d", rpc_url).await?;
    parse_u128(&result)
}

/// qiToken.balanceOf(address) -> u128 (qiToken units, 8 decimals)
/// selector: 0x70a08231
pub async fn balance_of(qi_token: &str, wallet: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0x70a08231{}", pad_address(wallet));
    let result = eth_call(qi_token, &data, rpc_url).await?;
    parse_u128(&result)
}

/// qiToken.borrowBalanceCurrent(address) -> u128 (underlying units)
/// selector: 0x17bfdfbc
pub async fn borrow_balance_current(qi_token: &str, wallet: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0x17bfdfbc{}", pad_address(wallet));
    let result = eth_call(qi_token, &data, rpc_url).await?;
    parse_u128(&result)
}

/// ERC-20 balanceOf(address) -> u128
/// selector: 0x70a08231
pub async fn erc20_balance_of(token: &str, wallet: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0x70a08231{}", pad_address(wallet));
    let result = eth_call(token, &data, rpc_url).await?;
    parse_u128(&result)
}

/// Comptroller.getAccountLiquidity(address) -> (uint256 error, uint256 liquidity, uint256 shortfall)
/// selector: 0x5ec88c79
/// Returns liquidity in USD with 18 decimal precision
pub async fn get_account_liquidity(comptroller: &str, wallet: &str, rpc_url: &str) -> anyhow::Result<(u128, u128, u128)> {
    let data = format!("0x5ec88c79{}", pad_address(wallet));
    let result = eth_call(comptroller, &data, rpc_url).await?;
    let hex = result.trim_start_matches("0x");
    if hex.len() < 192 {
        return Ok((0, 0, 0));
    }
    let error = parse_u128(&hex[..64])?;
    let liquidity = parse_u128(&hex[64..128])?;
    let shortfall = parse_u128(&hex[128..192])?;
    Ok((error, liquidity, shortfall))
}

/// Comptroller.getAllMarkets() -> address[]
/// selector: 0xb0772d0b
pub async fn get_all_markets(comptroller: &str, rpc_url: &str) -> anyhow::Result<Vec<String>> {
    let result = eth_call(comptroller, "0xb0772d0b", rpc_url).await?;
    decode_address_array(&result)
}

/// Decode a dynamic ABI-encoded address[] array from eth_call result
fn decode_address_array(hex_result: &str) -> anyhow::Result<Vec<String>> {
    let clean = hex_result.trim_start_matches("0x");
    if clean.len() < 128 {
        return Ok(vec![]);
    }
    // offset (32 bytes), length (32 bytes), then addresses
    let len_hex = &clean[64..128];
    let count = usize::from_str_radix(len_hex.trim_start_matches('0').max("0"), 16)
        .unwrap_or(0);
    let mut addrs = Vec::new();
    for i in 0..count {
        let start = 128 + i * 64;
        let end = start + 64;
        if end > clean.len() {
            break;
        }
        let addr_hex = &clean[start..end];
        // Take last 40 hex chars (20 bytes = address)
        let addr = format!("0x{}", &addr_hex[24..]);
        addrs.push(addr);
    }
    Ok(addrs)
}

/// Convert per-timestamp rate to APR percentage
/// APR% = rate_per_second * seconds_per_year / 1e18 * 100
pub fn rate_to_apr_pct(rate_per_second: u128, seconds_per_year: u128) -> f64 {
    (rate_per_second as f64) * (seconds_per_year as f64) / 1e18 * 100.0
}

/// Convert qiToken balance to underlying human-readable amount
/// underlying = qitoken_balance * exchange_rate / 1e18 / underlying_decimals_scale
pub fn qi_token_to_underlying(qi_balance: u128, exchange_rate: u128, underlying_decimals: u8, qi_decimals: u8) -> f64 {
    // exchange_rate is scaled by 1e18 * 10^(underlying_decimals - qi_decimals)
    let exp_diff = underlying_decimals as i32 - qi_decimals as i32;
    let raw = (qi_balance as f64) * (exchange_rate as f64) / 1e18;
    raw / 10f64.powi(exp_diff)
}

/// eth_call returning raw hex for multi-value decoding
pub async fn eth_call_raw(to: &str, data: &str, rpc_url: &str) -> anyhow::Result<Value> {
    let client = build_client()?;
    let body = json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [
            { "to": to, "data": data },
            "latest"
        ],
        "id": 1
    });
    let resp: Value = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .context("RPC request failed")?
        .json()
        .await
        .context("RPC response parse failed")?;
    Ok(resp)
}
