use std::process::Command;
use serde_json::Value;

/// Resolve the EVM wallet address for the given chain.
/// Uses `onchainos wallet addresses` and finds the first EVM entry.
pub fn resolve_wallet(chain_id: u64) -> anyhow::Result<String> {
    let output = Command::new("onchainos")
        .args(["wallet", "addresses"])
        .output()?;
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;
    let chain_id_str = chain_id.to_string();
    if let Some(evm_list) = json["data"]["evm"].as_array() {
        for entry in evm_list {
            if entry["chainIndex"].as_str() == Some(&chain_id_str) {
                if let Some(addr) = entry["address"].as_str() {
                    return Ok(addr.to_string());
                }
            }
        }
        // fallback: first EVM entry
        if let Some(first) = evm_list.first() {
            if let Some(addr) = first["address"].as_str() {
                return Ok(addr.to_string());
            }
        }
    }
    anyhow::bail!("Could not resolve wallet address for chain {}", chain_id)
}

/// Submit a contract call via onchainos wallet contract-call.
/// dry_run=true returns a mock response without calling onchainos.
/// amt: ETH value in wei (for payable calls like createIncreasePosition).
pub async fn wallet_contract_call(
    chain_id: u64,
    to: &str,
    input_data: &str,
    amt: Option<u64>,
    dry_run: bool,
) -> anyhow::Result<Value> {
    if dry_run {
        return Ok(serde_json::json!({
            "ok": true,
            "dry_run": true,
            "data": {
                "txHash": "0x0000000000000000000000000000000000000000000000000000000000000000"
            },
            "calldata": input_data
        }));
    }
    let chain_str = chain_id.to_string();
    let mut args = vec![
        "wallet",
        "contract-call",
        "--chain",
        &chain_str,
        "--to",
        to,
        "--input-data",
        input_data,
        "--force",
    ];
    let amt_str;
    if let Some(v) = amt {
        amt_str = v.to_string();
        args.extend_from_slice(&["--amt", &amt_str]);
    }
    let output = Command::new("onchainos").args(&args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: Value = serde_json::from_str(&stdout)?;
    // Check ok field: if ok=false, propagate the error message instead of silently returning
    if result["ok"].as_bool() == Some(false) {
        let msg = result["msg"]
            .as_str()
            .or_else(|| result["message"].as_str())
            .or_else(|| result["error"].as_str())
            .unwrap_or("onchainos returned ok:false with no message");
        anyhow::bail!("onchainos contract-call failed: {}", msg);
    }
    Ok(result)
}

/// Extract txHash from onchainos response: data.txHash -> txHash (root fallback).
/// Returns an error if no txHash is found, to prevent silent failures.
pub fn extract_tx_hash(result: &Value) -> anyhow::Result<String> {
    result["data"]["txHash"]
        .as_str()
        .or_else(|| result["txHash"].as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("No txHash in onchainos response: {}", result))
}

/// ERC-20 approve calldata: approve(address,uint256) = 0x095ea7b3
/// Uses type(uint256).max (all 32 bytes = 0xff) for true unlimited approval.
pub fn encode_approve(spender: &str) -> anyhow::Result<String> {
    let spender_clean = spender.trim_start_matches("0x");
    if spender_clean.len() != 40 {
        anyhow::bail!("Invalid spender address: {}", spender);
    }
    let spender_padded = format!("{:0>64}", spender_clean);
    // type(uint256).max = 2^256 - 1 = 32 bytes of 0xff
    let amount_hex = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    Ok(format!("0x095ea7b3{}{}", spender_padded, amount_hex))
}
