// onchainos CLI wrapper — Solana-specific for Sanctum Infinity
use anyhow::Result;
use serde_json::Value;
use std::process::Command;

/// Resolve the active Solana wallet address via onchainos.
/// ⚠️  Solana chain 501 does NOT support --output json flag.
/// Address lives at data.details[0].tokenAssets[0].address
pub fn resolve_wallet_solana() -> Result<String> {
    let output = Command::new("onchainos")
        .args(["wallet", "balance", "--chain", "501"])
        .output()?;
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;
    // Try details[0].tokenAssets[0].address first
    if let Some(addr) = json["data"]["details"]
        .get(0)
        .and_then(|d| d["tokenAssets"].get(0))
        .and_then(|t| t["address"].as_str())
    {
        if !addr.is_empty() {
            return Ok(addr.to_string());
        }
    }
    // fallback
    let addr = json["data"]["address"].as_str().unwrap_or("").to_string();
    if addr.is_empty() {
        anyhow::bail!("Could not resolve Solana wallet address. Ensure onchainos is logged in.");
    }
    Ok(addr)
}

/// Convert base64-encoded serialized transaction to base58.
/// onchainos --unsigned-tx expects base58; Sanctum API returns base64.
pub fn base64_to_base58(b64: &str) -> Result<String> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let bytes = STANDARD
        .decode(b64.trim())
        .map_err(|e| anyhow::anyhow!("Failed to decode base64 tx: {}", e))?;
    Ok(bs58::encode(bytes).into_string())
}

/// Submit a Sanctum serialized transaction via onchainos.
/// serialized_tx: base64-encoded VersionedTransaction (from Sanctum API).
/// program_id: the Sanctum program to route to (INF_PROGRAM_ID).
/// ⚠️  dry_run=true returns a simulated response without calling onchainos.
/// ⚠️  onchainos wallet contract-call does NOT accept --dry-run flag.
pub async fn wallet_contract_call_solana(
    program_id: &str,
    serialized_tx: &str, // base64-encoded (from Sanctum API response)
    dry_run: bool,
) -> Result<Value> {
    if dry_run {
        return Ok(serde_json::json!({
            "ok": true,
            "dry_run": true,
            "data": { "txHash": "" },
            "serialized_tx": serialized_tx
        }));
    }

    // Convert base64 → base58 (onchainos requires base58)
    let tx_base58 = base64_to_base58(serialized_tx)?;

    let output = Command::new("onchainos")
        .args([
            "wallet",
            "contract-call",
            "--chain",
            "501",
            "--to",
            program_id,
            "--unsigned-tx",
            &tx_base58,
            "--force",
        ])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: Value = serde_json::from_str(&stdout)
        .map_err(|e| anyhow::anyhow!("Failed to parse onchainos response: {}. Stdout: {}", e, stdout))?;
    Ok(result)
}

/// Extract txHash from onchainos response.
/// Checks data.swapTxHash → data.txHash → txHash (root).
pub fn extract_tx_hash(result: &Value) -> String {
    result["data"]["swapTxHash"]
        .as_str()
        .or_else(|| result["data"]["txHash"].as_str())
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("pending")
        .to_string()
}
