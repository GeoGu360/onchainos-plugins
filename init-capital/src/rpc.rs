// src/rpc.rs — Direct eth_call queries for INIT Capital on Blast
use anyhow::Context;
use serde_json::{json, Value};

/// eth_call with automatic fallback to secondary RPC
pub async fn eth_call_with_fallback(to: &str, data: &str, primary: &str, fallback: &str) -> anyhow::Result<String> {
    match eth_call(to, data, primary).await {
        Ok(r) => Ok(r),
        Err(_) => eth_call(to, data, fallback).await,
    }
}

/// Build a reqwest client with proxy support (reads HTTPS_PROXY env var)
fn build_client() -> reqwest::Client {
    let mut builder = reqwest::Client::builder();
    if let Ok(proxy_url) = std::env::var("HTTPS_PROXY").or_else(|_| std::env::var("https_proxy")) {
        if let Ok(proxy) = reqwest::Proxy::https(&proxy_url) {
            builder = builder.proxy(proxy);
        }
    }
    builder.build().unwrap_or_default()
}

/// Low-level eth_call helper with fallback RPC
pub async fn eth_call(to: &str, data: &str, rpc_url: &str) -> anyhow::Result<String> {
    let client = build_client();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [{ "to": to, "data": data }, "latest"],
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
    Ok(resp["result"].as_str().unwrap_or("0x").to_string())
}

/// Parse a uint256 from a 32-byte ABI-encoded hex result
pub fn parse_u128(hex_result: &str) -> anyhow::Result<u128> {
    let clean = hex_result.trim_start_matches("0x");
    if clean.is_empty() || clean == "0" {
        return Ok(0);
    }
    let trimmed = if clean.len() > 32 { &clean[clean.len() - 32..] } else { clean };
    Ok(u128::from_str_radix(trimmed, 16).context("parse u128 failed")?)
}

/// Pad an address to 32 bytes (strip 0x, left-pad with zeros)
pub fn pad_address(addr: &str) -> String {
    let clean = addr.trim_start_matches("0x");
    format!("{:0>64}", clean)
}

/// Pad a u128 to 32-byte hex
pub fn pad_u128(val: u128) -> String {
    format!("{:064x}", val)
}

/// Pad a u64 to 32-byte hex
pub fn pad_u64(val: u64) -> String {
    format!("{:064x}", val)
}

// ── LendingPool read helpers ──────────────────────────────────────────────────

/// pool.totalAssets() → u128 (total underlying supplied, 0x01e1d114)
pub async fn total_assets(pool: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let res = eth_call(pool, "0x01e1d114", rpc_url).await?;
    parse_u128(&res)
}

/// pool.getSupplyRate_e18() → u128 (supply APR in e18, 0xbea205a2)
pub async fn get_supply_rate_e18(pool: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let res = eth_call(pool, "0xbea205a2", rpc_url).await?;
    parse_u128(&res)
}

/// pool.getBorrowRate_e18() → u128 (borrow APR in e18, 0x8d80c344)
pub async fn get_borrow_rate_e18(pool: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let res = eth_call(pool, "0x8d80c344", rpc_url).await?;
    parse_u128(&res)
}

/// pool.decimals() → u8 (0x313ce567)
pub async fn pool_decimals(pool: &str, rpc_url: &str) -> anyhow::Result<u8> {
    let res = eth_call(pool, "0x313ce567", rpc_url).await?;
    let val = parse_u128(&res)?;
    Ok(val as u8)
}

/// pool.toAmt(shares) → u128 underlying amount (0x183c268e)
pub async fn to_amt(pool: &str, shares: u128, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0x183c268e{}", pad_u128(shares));
    let res = eth_call(pool, &data, rpc_url).await?;
    parse_u128(&res)
}

/// pool.toShares(amt) → u128 shares (0x9e57c975)
pub async fn to_shares(pool: &str, amt: u128, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0x9e57c975{}", pad_u128(amt));
    let res = eth_call(pool, &data, rpc_url).await?;
    parse_u128(&res)
}

/// pool.debtShareToAmtCurrent(shares) → u128 debt amount (0x31a86fe1)
pub async fn debt_share_to_amt_current(pool: &str, shares: u128, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0x31a86fe1{}", pad_u128(shares));
    let res = eth_call(pool, &data, rpc_url).await?;
    parse_u128(&res)
}

// ── POS_MANAGER read helpers ──────────────────────────────────────────────────

/// posManager.getViewerPosIdsLength(address) → u128 (0x0c4478c7)
pub async fn get_viewer_pos_ids_length(pos_manager: &str, viewer: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0x0c4478c7{}", pad_address(viewer));
    let res = eth_call(pos_manager, &data, rpc_url).await?;
    parse_u128(&res)
}

/// posManager.getViewerPosIdsAt(address, index) → u128 posId (0xfd379b17)
pub async fn get_viewer_pos_ids_at(pos_manager: &str, viewer: &str, index: u128, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0xfd379b17{}{}", pad_address(viewer), pad_u128(index));
    let res = eth_call(pos_manager, &data, rpc_url).await?;
    parse_u128(&res)
}

/// posManager.getPosMode(posId) → u16 mode (0xf92c4d4c)
pub async fn get_pos_mode(pos_manager: &str, pos_id: u128, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0xf92c4d4c{}", pad_u128(pos_id));
    let res = eth_call(pos_manager, &data, rpc_url).await?;
    parse_u128(&res)
}

/// posManager.getCollAmt(posId, pool) → u128 shares (0x402414b3)
pub async fn get_coll_amt(pos_manager: &str, pos_id: u128, pool: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0x402414b3{}{}", pad_u128(pos_id), pad_address(pool));
    let res = eth_call(pos_manager, &data, rpc_url).await?;
    parse_u128(&res)
}

/// posManager.getPosDebtShares(posId, pool) → u128 debt shares (0x10e28e71)
pub async fn get_pos_debt_shares(pos_manager: &str, pos_id: u128, pool: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0x10e28e71{}{}", pad_u128(pos_id), pad_address(pool));
    let res = eth_call(pos_manager, &data, rpc_url).await?;
    parse_u128(&res)
}

// ── INIT_CORE read helpers ─────────────────────────────────────────────────────

/// initCore.getPosHealthCurrent_e18(posId) → u128 health (1e18 = 1.0, 0xa72ca39b)
pub async fn get_pos_health_current_e18(core: &str, pos_id: u128, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0xa72ca39b{}", pad_u128(pos_id));
    let res = eth_call(core, &data, rpc_url).await?;
    parse_u128(&res)
}

// ── ERC-20 helpers ─────────────────────────────────────────────────────────────

/// ERC-20 balanceOf(address) → u128
pub async fn erc20_balance_of(token: &str, wallet: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0x70a08231{}", pad_address(wallet));
    let res = eth_call(token, &data, rpc_url).await?;
    parse_u128(&res)
}

/// ERC-20 allowance(owner, spender) → u128
pub async fn erc20_allowance(token: &str, owner: &str, spender: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0xdd62ed3e{}{}", pad_address(owner), pad_address(spender));
    let res = eth_call(token, &data, rpc_url).await?;
    parse_u128(&res)
}
