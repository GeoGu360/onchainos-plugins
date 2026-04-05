/// Direct eth_call utilities — no onchainos, no wallet required

use anyhow::Result;
use serde_json::{json, Value};

/// Execute a raw eth_call and return the result hex string
pub async fn eth_call(to: &str, data: &str, rpc_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
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

/// Decode a hex-encoded uint256 to u128 (safe for token amounts ≤ u128::MAX)
pub fn decode_uint256_to_u128(hex: &str) -> Result<u128> {
    let hex = hex.trim_start_matches("0x");
    if hex.len() > 32 {
        // Take the last 32 chars (lower 128 bits)
        let trimmed = &hex[hex.len() - 32..];
        return Ok(u128::from_str_radix(trimmed, 16)?);
    }
    Ok(u128::from_str_radix(hex, 16).unwrap_or(0))
}

/// Pad an address to 32 bytes (64 hex chars, no 0x prefix)
pub fn pad_address(addr: &str) -> String {
    let clean = addr.trim_start_matches("0x").to_lowercase();
    format!("{:0>64}", clean)
}

/// Pad a u128 to 32 bytes (64 hex chars, no 0x prefix)
pub fn pad_u128(val: u128) -> String {
    format!("{:064x}", val)
}

/// Read ERC-20 balanceOf
/// selector: 0x70a08231
pub async fn erc20_balance_of(token: &str, holder: &str, rpc_url: &str) -> Result<u128> {
    let data = format!("0x70a08231{}", pad_address(holder));
    let result = eth_call(token, &data, rpc_url).await?;
    decode_uint256_to_u128(&result)
}

/// Read ERC-20 allowance
/// selector: 0xdd62ed3e
pub async fn erc20_allowance(token: &str, owner: &str, spender: &str, rpc_url: &str) -> Result<u128> {
    let data = format!("0xdd62ed3e{}{}", pad_address(owner), pad_address(spender));
    let result = eth_call(token, &data, rpc_url).await?;
    decode_uint256_to_u128(&result)
}

/// Read ERC-20 totalSupply
/// selector: 0x18160ddd
pub async fn erc20_total_supply(token: &str, rpc_url: &str) -> Result<u128> {
    let result = eth_call(token, "0x18160ddd", rpc_url).await?;
    decode_uint256_to_u128(&result)
}

/// Read Accountant.getRateInQuote(quoteAsset)
/// Returns: uint256 rate in 18 decimals (quote token units per share)
/// selector: 0x1dcbb110
pub async fn get_rate_in_quote(accountant: &str, quote_asset: &str, rpc_url: &str) -> Result<u128> {
    let data = format!("0x1dcbb110{}", pad_address(quote_asset));
    let result = eth_call(accountant, &data, rpc_url).await?;
    decode_uint256_to_u128(&result)
}

/// Read Teller.assetData(asset)
/// Returns: (bool allowDeposits, bool allowWithdrawals, uint112 sharePremium)
/// selector: 0x41fee44a
pub async fn asset_data(teller: &str, asset: &str, rpc_url: &str) -> Result<(bool, bool)> {
    let data = format!("0x41fee44a{}", pad_address(asset));
    let result = eth_call(teller, &data, rpc_url).await?;
    let hex = result.trim_start_matches("0x");
    if hex.len() < 64 {
        return Ok((false, false));
    }
    // First word: allowDeposits (bool = uint256)
    let allow_deposits = !hex[..64].chars().all(|c| c == '0') && hex[63..64] == *"1";
    // Second word: allowWithdrawals
    let allow_withdrawals = hex.len() >= 128 && !hex[64..128].chars().all(|c| c == '0') && hex[127..128] == *"1";
    Ok((allow_deposits, allow_withdrawals))
}

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DefiLlamaDataPoint {
    pub timestamp: String,
    #[serde(rename = "tvlUsd")]
    pub tvl_usd: Option<f64>,
    pub apy: Option<f64>,
    #[serde(rename = "apyBase")]
    pub apy_base: Option<f64>,
    #[serde(rename = "apyBase7d")]
    pub apy_base7d: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct DefiLlamaChartResponse {
    pub data: Vec<DefiLlamaDataPoint>,
}

/// Fetch current APY and TVL from DefiLlama chart API
pub async fn fetch_defillama_chart(pool_id: &str, base_url: &str) -> Result<DefiLlamaDataPoint> {
    let url = format!("{}/chart/{}", base_url, pool_id);
    let client = reqwest::Client::new();
    let resp: DefiLlamaChartResponse = client.get(&url).send().await?.json().await?;
    resp.data
        .into_iter()
        .last()
        .ok_or_else(|| anyhow::anyhow!("No data points in DefiLlama response for pool {}", pool_id))
}
