use anyhow::Result;
use serde_json::{json, Value};
use crate::config::ETH_RPC;

fn build_client() -> reqwest::Client {
    let mut builder = reqwest::Client::builder();
    if let Ok(proxy_url) = std::env::var("HTTPS_PROXY")
        .or_else(|_| std::env::var("https_proxy"))
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .or_else(|_| std::env::var("http_proxy"))
    {
        if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
            builder = builder.proxy(proxy);
        }
    }
    builder.build().unwrap_or_default()
}

/// Execute an eth_call on Ethereum mainnet
pub async fn eth_call(to: &str, data: &str) -> Result<String> {
    let client = build_client();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [{"to": to, "data": data}, "latest"],
        "id": 1
    });
    let resp: Value = client.post(ETH_RPC)
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

/// Pad address to 32 bytes (remove 0x prefix, left-pad with zeros)
pub fn pad_address(addr: &str) -> String {
    let clean = addr.strip_prefix("0x").unwrap_or(addr).to_lowercase();
    format!("{:0>64}", clean)
}

/// Decode uint256 from ABI-encoded result
pub fn decode_u256(hex: &str) -> u128 {
    let clean = hex.strip_prefix("0x").unwrap_or(hex);
    if clean.len() < 64 {
        return 0;
    }
    u128::from_str_radix(&clean[..32], 16).unwrap_or(0)
}

/// Read ERC-20 balanceOf(address) — returns raw balance as u128
pub async fn erc20_balance_of(token: &str, wallet: &str) -> Result<u128> {
    let data = format!("0x70a08231{}", pad_address(wallet));
    let result = eth_call(token, &data).await?;
    Ok(decode_u256(&result))
}

/// Read ERC-20 allowance(owner, spender) — returns raw allowance as u128
pub async fn erc20_allowance(token: &str, owner: &str, spender: &str) -> Result<u128> {
    let data = format!("0xdd62ed3e{}{}", pad_address(owner), pad_address(spender));
    let result = eth_call(token, &data).await?;
    Ok(decode_u256(&result))
}

/// Read CvxCrvStaking.earned(address) — pending CRV rewards
pub async fn cvxcrv_earned(wallet: &str) -> Result<u128> {
    let data = format!("0x008cc262{}", pad_address(wallet));
    let result = eth_call(crate::config::CVXCRV_STAKING, &data).await?;
    Ok(decode_u256(&result))
}

/// Format token amount with decimals for display
pub fn format_amount(raw: u128, decimals: u32) -> String {
    let divisor = 10u128.pow(decimals);
    let whole = raw / divisor;
    let frac = raw % divisor;
    if frac == 0 {
        format!("{}", whole)
    } else {
        format!("{}.{:0>width$}", whole, frac, width = decimals as usize)
            .trim_end_matches('0')
            .to_string()
    }
}
