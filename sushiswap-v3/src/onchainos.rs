use std::process::Command;
use serde_json::Value;

/// Resolve the EVM wallet address for a given chain_id via onchainos CLI.
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
        if let Some(first) = evm_list.first() {
            if let Some(addr) = first["address"].as_str() {
                return Ok(addr.to_string());
            }
        }
    }
    anyhow::bail!("Could not resolve wallet address for chain {}", chain_id)
}

/// Submit a contract call via onchainos CLI.
/// If dry_run is true, returns a mock response without broadcasting.
pub async fn wallet_contract_call(
    chain_id: u64,
    to: &str,
    input_data: &str,
    from: Option<&str>,
    amt: Option<u128>,
    force: bool,
    dry_run: bool,
) -> anyhow::Result<Value> {
    if dry_run {
        return Ok(serde_json::json!({
            "ok": true,
            "dry_run": true,
            "data": {"txHash": "0x0000000000000000000000000000000000000000000000000000000000000000"},
            "calldata": input_data
        }));
    }
    let chain_str = chain_id.to_string();
    let mut args = vec![
        "wallet", "contract-call",
        "--chain", &chain_str,
        "--to", to,
        "--input-data", input_data,
    ];
    let amt_str;
    if let Some(v) = amt {
        amt_str = v.to_string();
        args.extend_from_slice(&["--amt", &amt_str]);
    }
    let from_str;
    if let Some(f) = from {
        from_str = f.to_string();
        args.extend_from_slice(&["--from", &from_str]);
    }
    if force {
        args.push("--force");
    }
    let output = Command::new("onchainos").args(&args).output()?;
    if !output.status.success() {
        anyhow::bail!(
            "onchainos wallet contract-call exited with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let result: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .map_err(|e| anyhow::anyhow!("Failed to parse onchainos response: {}", e))?;
    if result["ok"].as_bool() != Some(true) {
        anyhow::bail!(
            "onchainos wallet contract-call returned ok=false: {}",
            result
        );
    }
    Ok(result)
}

/// Extract txHash from onchainos CLI response.
/// Returns Err if the hash is missing or is the sentinel "pending" value.
pub fn extract_tx_hash(result: &Value) -> anyhow::Result<String> {
    let hash = result["data"]["txHash"]
        .as_str()
        .or_else(|| result["txHash"].as_str());
    match hash {
        Some(h) if !h.is_empty() && h != "pending" => Ok(h.to_string()),
        Some(_) => anyhow::bail!("onchainos returned 'pending' — transaction may not have been broadcast"),
        None => anyhow::bail!("No txHash in onchainos response: {}", result),
    }
}
