// src/rpc.rs — direct eth_call via public RPC (no onchainos needed for reads)
use anyhow::Context;
use serde_json::json;

/// Perform an eth_call and return the raw hex result string.
pub async fn eth_call(to: &str, data: &str, rpc_url: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let body = json!({
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

    Ok(resp["result"]
        .as_str()
        .unwrap_or("0x")
        .to_string())
}

/// Decode a uint256 from a 32-byte hex response (with or without 0x prefix).
pub fn decode_uint256(hex: &str) -> u128 {
    let clean = hex.trim_start_matches("0x");
    if clean.is_empty() {
        return 0;
    }
    // Take last 32 chars (16 bytes) to avoid overflow from big uint256
    let tail = if clean.len() > 32 {
        &clean[clean.len() - 32..]
    } else {
        clean
    };
    u128::from_str_radix(tail, 16).unwrap_or(0)
}


/// Format wei as human-readable ETH (18 decimals).
pub fn format_18dec(wei: u128) -> String {
    let eth = wei / 1_000_000_000_000_000_000u128;
    let frac = (wei % 1_000_000_000_000_000_000u128) / 1_000_000_000_000u128; // 6 decimal places
    format!("{}.{:06}", eth, frac)
}


/// Encode a uint256 value as a 32-byte ABI parameter.
pub fn encode_uint256(val: u128) -> String {
    format!("{:064x}", val)
}
