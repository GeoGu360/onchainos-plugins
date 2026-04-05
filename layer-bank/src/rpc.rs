// src/rpc.rs — Direct eth_call queries for LayerBank on Scroll
use anyhow::Context;
use serde_json::{json, Value};

/// Low-level eth_call helper
pub async fn eth_call(to: &str, data: &str, rpc_url: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
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

// ── LayerBank lToken read helpers ─────────────────────────────────────────────

/// lToken.exchangeRate() → u128 (18-decimal scaled)
pub async fn exchange_rate(ltoken: &str, rpc_url: &str) -> anyhow::Result<u128> {
    // 0x3ba0b9a9 = exchangeRate()
    let res = eth_call(ltoken, "0x3ba0b9a9", rpc_url).await?;
    parse_u128(&res)
}

/// lToken.totalBorrow() → u128 (raw underlying units)
pub async fn total_borrow(ltoken: &str, rpc_url: &str) -> anyhow::Result<u128> {
    // 0x8285ef40 = totalBorrow()
    let res = eth_call(ltoken, "0x8285ef40", rpc_url).await?;
    parse_u128(&res)
}

/// lToken.getCash() → u128 (raw underlying units available)
pub async fn get_cash(ltoken: &str, rpc_url: &str) -> anyhow::Result<u128> {
    // 0x3b1d21a2 = getCash()
    let res = eth_call(ltoken, "0x3b1d21a2", rpc_url).await?;
    parse_u128(&res)
}

/// lToken.totalSupply() → u128 (lToken units)
pub async fn total_supply_ltoken(ltoken: &str, rpc_url: &str) -> anyhow::Result<u128> {
    // 0x18160ddd = totalSupply()
    let res = eth_call(ltoken, "0x18160ddd", rpc_url).await?;
    parse_u128(&res)
}

/// lToken.borrowBalanceOf(account) → u128
pub async fn borrow_balance_of(ltoken: &str, account: &str, rpc_url: &str) -> anyhow::Result<u128> {
    // 0x374c49b4 = borrowBalanceOf(address)
    let data = format!("0x374c49b4{}", pad_address(account));
    let res = eth_call(ltoken, &data, rpc_url).await?;
    parse_u128(&res)
}

/// lToken.balanceOf(account) → u128 (lToken balance)
pub async fn ltoken_balance_of(ltoken: &str, account: &str, rpc_url: &str) -> anyhow::Result<u128> {
    // 0x70a08231 = balanceOf(address)
    let data = format!("0x70a08231{}", pad_address(account));
    let res = eth_call(ltoken, &data, rpc_url).await?;
    parse_u128(&res)
}

/// lToken.accountSnapshot(address) → (lTokenBalance, borrowBalance, exchangeRate)
pub async fn account_snapshot(ltoken: &str, account: &str, rpc_url: &str) -> anyhow::Result<(u128, u128, u128)> {
    // 0x014a296f = accountSnapshot(address)
    let data = format!("0x014a296f{}", pad_address(account));
    let res = eth_call(ltoken, &data, rpc_url).await?;
    let hex = res.trim_start_matches("0x");
    if hex.len() < 192 {
        return Ok((0, 0, 0));
    }
    let ltoken_bal = u128::from_str_radix(&hex[0..64], 16).unwrap_or(0);
    let borrow_bal = u128::from_str_radix(&hex[64..128], 16).unwrap_or(0);
    let ex_rate = u128::from_str_radix(&hex[128..192], 16).unwrap_or(0);
    Ok((ltoken_bal, borrow_bal, ex_rate))
}

// ── Core / PriceCalculator read helpers ──────────────────────────────────────

/// Core.allMarkets() → Vec<address>
pub async fn all_markets(core: &str, rpc_url: &str) -> anyhow::Result<Vec<String>> {
    // 0x375a7cba = allMarkets()
    let res = eth_call(core, "0x375a7cba", rpc_url).await?;
    let hex = res.trim_start_matches("0x");
    if hex.len() < 128 {
        return Ok(vec![]);
    }
    let len = usize::from_str_radix(&hex[64..128], 16).unwrap_or(0);
    let mut addrs = Vec::with_capacity(len);
    for i in 0..len {
        let start = 128 + i * 64;
        if start + 64 > hex.len() { break; }
        let addr = format!("0x{}", &hex[start + 24..start + 64]);
        addrs.push(addr);
    }
    Ok(addrs)
}

/// Core.marketInfoOf(address) → (isListed, supplyCap, borrowCap, collateralFactor)
pub async fn market_info_of(core: &str, ltoken: &str, rpc_url: &str) -> anyhow::Result<(bool, u128, u128, u128)> {
    // 0x6e8584fd = marketInfoOf(address)
    let data = format!("0x6e8584fd{}", pad_address(ltoken));
    let res = eth_call(core, &data, rpc_url).await?;
    let hex = res.trim_start_matches("0x");
    if hex.len() < 256 {
        return Ok((false, 0, 0, 0));
    }
    let is_listed = u128::from_str_radix(&hex[0..64], 16).unwrap_or(0) == 1;
    let supply_cap = u128::from_str_radix(&hex[64..128], 16).unwrap_or(0);
    let borrow_cap = u128::from_str_radix(&hex[128..192], 16).unwrap_or(0);
    let collateral_factor = u128::from_str_radix(&hex[192..256], 16).unwrap_or(0);
    Ok((is_listed, supply_cap, borrow_cap, collateral_factor))
}

/// Core.accountLiquidityOf(address) → (collateralInUSD, supplyInUSD, borrowInUSD)
pub async fn account_liquidity_of(core: &str, account: &str, rpc_url: &str) -> anyhow::Result<(u128, u128, u128)> {
    // 0xf8982e7a = accountLiquidityOf(address)
    let data = format!("0xf8982e7a{}", pad_address(account));
    let res = eth_call(core, &data, rpc_url).await?;
    let hex = res.trim_start_matches("0x");
    if hex.len() < 192 {
        return Ok((0, 0, 0));
    }
    let collateral = u128::from_str_radix(&hex[0..64], 16).unwrap_or(0);
    let supply = u128::from_str_radix(&hex[64..128], 16).unwrap_or(0);
    let borrow = u128::from_str_radix(&hex[128..192], 16).unwrap_or(0);
    Ok((collateral, supply, borrow))
}

/// PriceCalculator.getUnderlyingPrice(lToken) → uint256 (price in USD, 18 decimals)
pub async fn get_underlying_price(price_calc: &str, ltoken: &str, rpc_url: &str) -> anyhow::Result<u128> {
    // 0xfc57d4df = getUnderlyingPrice(address)
    let data = format!("0xfc57d4df{}", pad_address(ltoken));
    let res = eth_call(price_calc, &data, rpc_url).await?;
    parse_u128(&res)
}

/// ERC-20 balanceOf(address) → u128
pub async fn erc20_balance_of(token: &str, wallet: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0x70a08231{}", pad_address(wallet));
    let res = eth_call(token, &data, rpc_url).await?;
    parse_u128(&res)
}

/// lToken.symbol() → String
pub async fn ltoken_symbol(ltoken: &str, rpc_url: &str) -> anyhow::Result<String> {
    // 0x95d89b41 = symbol()
    let res = eth_call(ltoken, "0x95d89b41", rpc_url).await?;
    let hex = res.trim_start_matches("0x");
    if hex.len() < 128 {
        return Ok(ltoken[..8].to_string());
    }
    let length = usize::from_str_radix(&hex[64..128], 16).unwrap_or(0);
    if length == 0 || 128 + length * 2 > hex.len() {
        return Ok("?".to_string());
    }
    let str_hex = &hex[128..128 + length * 2];
    Ok(String::from_utf8_lossy(&hex::decode(str_hex).unwrap_or_default()).to_string())
}
