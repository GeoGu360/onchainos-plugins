use std::process::Command;
use serde_json::Value;

/// Resolve the active EVM wallet address for a given chain ID.
/// Uses `onchainos wallet balance --chain <id> --output json`
pub fn resolve_wallet(chain_id: u64) -> anyhow::Result<String> {
    let chain_str = chain_id.to_string();
    let output = Command::new("onchainos")
        .args(["wallet", "balance", "--chain", &chain_str, "--output", "json"])
        .output()?;
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;
    // Try data.address (common)
    if let Some(addr) = json["data"]["address"].as_str() {
        if !addr.is_empty() && addr.starts_with("0x") {
            return Ok(addr.to_string());
        }
    }
    // Fallback: data.details[0].tokenAssets[0].address
    if let Some(addr) = json["data"]["details"][0]["tokenAssets"][0]["address"].as_str() {
        if !addr.is_empty() {
            return Ok(addr.to_string());
        }
    }
    anyhow::bail!("Could not resolve wallet address for chain {}", chain_id)
}

/// Resolve wallet via `onchainos wallet addresses` (more reliable for EVM chains).
pub fn resolve_wallet_from_addresses(chain_index: &str) -> anyhow::Result<String> {
    let output = Command::new("onchainos")
        .args(["wallet", "addresses"])
        .output()?;
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;
    if let Some(evm_list) = json["data"]["evm"].as_array() {
        for entry in evm_list {
            if entry["chainIndex"].as_str() == Some(chain_index) {
                if let Some(addr) = entry["address"].as_str() {
                    return Ok(addr.to_string());
                }
            }
        }
    }
    anyhow::bail!("Could not find address for chainIndex {}", chain_index)
}

/// Submit a contract-call via onchainos.
///
/// ⚠️  dry_run=true returns a simulated response WITHOUT calling onchainos.
///     onchainos wallet contract-call does NOT support --dry-run.
pub async fn wallet_contract_call(
    chain_id: u64,
    to: &str,
    input_data: &str,
    from: Option<&str>,
    amt_wei: Option<u128>,
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
    let mut args: Vec<String> = vec![
        "wallet".into(),
        "contract-call".into(),
        "--chain".into(),
        chain_str.clone(),
        "--to".into(),
        to.to_string(),
        "--input-data".into(),
        input_data.to_string(),
        "--force".into(),
    ];
    if let Some(v) = amt_wei {
        args.push("--amt".into());
        args.push(v.to_string());
    }
    if let Some(f) = from {
        args.push("--from".into());
        args.push(f.to_string());
    }

    let output = Command::new("onchainos")
        .args(&args)
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|_| serde_json::json!({"ok": false, "error": &*stdout}));
    Ok(result)
}

/// Extract txHash from onchainos response.
/// Checks data.txHash first, then root txHash.
pub fn extract_tx_hash(result: &Value) -> String {
    result["data"]["txHash"]
        .as_str()
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("pending")
        .to_string()
}
