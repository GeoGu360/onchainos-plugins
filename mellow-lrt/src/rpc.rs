use anyhow::Context;

/// Make a raw eth_call via JSON-RPC.
pub async fn eth_call(to: &str, data: &str, rpc_url: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [
            { "to": to, "data": data },
            "latest"
        ],
        "id": 1
    });
    let resp: serde_json::Value = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .context("RPC request failed")?
        .json()
        .await
        .context("RPC response parse failed")?;

    if let Some(err) = resp.get("error") {
        anyhow::bail!("eth_call error: {}", err);
    }
    Ok(resp["result"].as_str().unwrap_or("0x").to_string())
}

/// Read ERC-20 balanceOf(address) → uint256.
/// selector: 0x70a08231
pub async fn erc20_balance_of(token: &str, owner: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let owner_clean = format!("{:0>64}", owner.trim_start_matches("0x").to_lowercase());
    let data = format!("0x70a08231{}", owner_clean);
    let hex = eth_call(token, &data, rpc_url).await?;
    parse_u128_from_hex(&hex)
}

/// Read ERC-20 decimals() → uint8.
/// selector: 0x313ce567
pub async fn erc20_decimals(token: &str, rpc_url: &str) -> anyhow::Result<u8> {
    let hex = eth_call(token, "0x313ce567", rpc_url).await?;
    let hex_clean = hex.trim_start_matches("0x");
    if hex_clean.is_empty() {
        return Ok(18);
    }
    let padded = format!("{:0>64}", hex_clean);
    Ok(u8::from_str_radix(&padded[padded.len().saturating_sub(2)..], 16).unwrap_or(18))
}

/// Read ERC-20 symbol() → string.
/// selector: 0x95d89b41
#[allow(dead_code)]
pub async fn erc20_symbol(token: &str, rpc_url: &str) -> anyhow::Result<String> {
    let hex = eth_call(token, "0x95d89b41", rpc_url).await?;
    decode_string_from_hex(&hex)
}

/// Read ERC-4626 asset() → address.
/// selector: 0x38d52e0f
pub async fn vault_asset(vault: &str, rpc_url: &str) -> anyhow::Result<String> {
    let hex = eth_call(vault, "0x38d52e0f", rpc_url).await?;
    let clean = hex.trim_start_matches("0x");
    if clean.len() < 40 {
        anyhow::bail!("Invalid asset() response: {}", hex);
    }
    Ok(format!("0x{}", &clean[clean.len() - 40..]))
}

/// Read ERC-4626 totalAssets() → uint256.
/// selector: 0x01e1d114
pub async fn vault_total_assets(vault: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let hex = eth_call(vault, "0x01e1d114", rpc_url).await?;
    parse_u128_from_hex(&hex)
}

/// Read ERC-4626 totalSupply() → uint256.
/// selector: 0x18160ddd
#[allow(dead_code)]
pub async fn vault_total_supply(vault: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let hex = eth_call(vault, "0x18160ddd", rpc_url).await?;
    parse_u128_from_hex(&hex)
}

/// Read ERC-4626 convertToAssets(uint256 shares) → uint256.
/// selector: 0x07a2d13a
pub async fn vault_convert_to_assets(vault: &str, shares: u128, rpc_url: &str) -> anyhow::Result<u128> {
    let data = format!("0x07a2d13a{:064x}", shares);
    let hex = eth_call(vault, &data, rpc_url).await?;
    parse_u128_from_hex(&hex)
}

/// Read ERC-4626 maxDeposit(address) → uint256.
/// selector: 0x402d267d
pub async fn vault_max_deposit(vault: &str, receiver: &str, rpc_url: &str) -> anyhow::Result<u128> {
    let rec_clean = format!("{:0>64}", receiver.trim_start_matches("0x").to_lowercase());
    let data = format!("0x402d267d{}", rec_clean);
    let hex = eth_call(vault, &data, rpc_url).await?;
    parse_u128_from_hex(&hex)
}

/// Read MellowSymbioticVault claimableAssetsOf(address) → uint256.
/// selector: 0xe7beaf9d
/// Returns None if vault doesn't support this function.
pub async fn vault_claimable_assets_of(vault: &str, account: &str, rpc_url: &str) -> Option<u128> {
    let acc_clean = format!("{:0>64}", account.trim_start_matches("0x").to_lowercase());
    let data = format!("0xe7beaf9d{}", acc_clean);
    eth_call(vault, &data, rpc_url).await.ok()
        .and_then(|hex| parse_u128_from_hex(&hex).ok())
}

/// Read MellowSymbioticVault pendingAssetsOf(address) → uint256.
/// selector: 0x63c6b4eb
/// Returns None if vault doesn't support this function.
pub async fn vault_pending_assets_of(vault: &str, account: &str, rpc_url: &str) -> Option<u128> {
    let acc_clean = format!("{:0>64}", account.trim_start_matches("0x").to_lowercase());
    let data = format!("0x63c6b4eb{}", acc_clean);
    eth_call(vault, &data, rpc_url).await.ok()
        .and_then(|hex| parse_u128_from_hex(&hex).ok())
}

/// Parse a u128 from a 32-byte hex eth_call result.
pub fn parse_u128_from_hex(hex: &str) -> anyhow::Result<u128> {
    let hex_clean = hex.trim_start_matches("0x");
    if hex_clean.is_empty() || hex_clean == "0" {
        return Ok(0);
    }
    let padded = format!("{:0>64}", hex_clean);
    let tail = &padded[padded.len().saturating_sub(32)..];
    Ok(u128::from_str_radix(tail, 16).unwrap_or(0))
}

/// Decode ABI-encoded string from eth_call result.
fn decode_string_from_hex(hex: &str) -> anyhow::Result<String> {
    let hex_clean = hex.trim_start_matches("0x");
    if hex_clean.len() < 128 {
        return Ok("UNKNOWN".to_string());
    }
    let offset = usize::from_str_radix(&hex_clean[0..64], 16).unwrap_or(32);
    let len_pos = offset * 2;
    if hex_clean.len() < len_pos + 64 {
        return Ok("UNKNOWN".to_string());
    }
    let len = usize::from_str_radix(&hex_clean[len_pos..len_pos + 64], 16).unwrap_or(0);
    if len == 0 {
        return Ok("".to_string());
    }
    let data_start = len_pos + 64;
    let data_end = data_start + len * 2;
    if data_end > hex_clean.len() {
        return Ok("UNKNOWN".to_string());
    }
    let bytes = hex::decode(&hex_clean[data_start..data_end]).unwrap_or_default();
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

/// Format a raw token amount to human-readable string.
pub fn format_amount(raw: u128, decimals: u8) -> String {
    if decimals == 0 {
        return raw.to_string();
    }
    let d = decimals as u32;
    let divisor = 10u128.pow(d);
    let whole = raw / divisor;
    let frac = raw % divisor;
    if frac == 0 {
        format!("{}", whole)
    } else {
        let frac_str = format!("{:0>width$}", frac, width = d as usize);
        let trimmed = frac_str.trim_end_matches('0');
        format!("{}.{}", whole, trimmed)
    }
}

/// Parse human-readable amount string to raw u128.
pub fn parse_amount(s: &str, decimals: u8) -> anyhow::Result<u128> {
    let s = s.trim();
    if s.is_empty() {
        anyhow::bail!("Empty amount string");
    }
    let d = decimals as u32;
    let multiplier = 10u128.pow(d);
    if let Some(dot_pos) = s.find('.') {
        let whole: u128 = s[..dot_pos].parse().context("Invalid whole part")?;
        let frac_str = &s[dot_pos + 1..];
        let frac_len = frac_str.len() as u32;
        let frac: u128 = frac_str.parse().context("Invalid fractional part")?;
        if frac_len > d {
            anyhow::bail!("Too many decimal places (max {})", d);
        }
        let frac_scaled = frac * 10u128.pow(d - frac_len);
        Ok(whole * multiplier + frac_scaled)
    } else {
        let whole: u128 = s.parse().context("Invalid integer amount")?;
        Ok(whole * multiplier)
    }
}
