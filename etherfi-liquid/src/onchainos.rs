/// onchainos CLI wrapper — EVM contract calls

use anyhow::Result;
use serde_json::Value;
use std::process::Command;

/// Resolve the active EVM wallet address for the given chain
/// NOTE: chain 1 (Ethereum) does not support --output json; parse response directly
pub fn resolve_wallet(chain_id: u64) -> Result<String> {
    let chain_str = chain_id.to_string();
    // Do NOT use --output json — not supported on all chains (e.g. chain 1)
    let output = Command::new("onchainos")
        .args(["wallet", "balance", "--chain", &chain_str])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout)
        .map_err(|e| anyhow::anyhow!("Failed to parse onchainos response: {} — output was: {}", e, stdout))?;

    // Try data.address first, then data.details[0].tokenAssets[0].address
    if let Some(addr) = json["data"]["address"].as_str() {
        if !addr.is_empty() {
            return Ok(addr.to_string());
        }
    }
    // Fallback: data.details[0].tokenAssets[0].address
    if let Some(addr) = json["data"]["details"]
        .get(0)
        .and_then(|d| d["tokenAssets"].get(0))
        .and_then(|t| t["address"].as_str())
    {
        if !addr.is_empty() {
            return Ok(addr.to_string());
        }
    }
    anyhow::bail!("Could not resolve wallet address for chain {}. Is onchainos logged in?", chain_id)
}

/// Call onchainos wallet contract-call for EVM
/// dry_run=true → returns simulated response without calling onchainos
/// onchainos wallet contract-call does NOT accept --dry-run
pub async fn wallet_contract_call(
    chain_id: u64,
    to: &str,
    input_data: &str,
    from: Option<&str>,
    amt: Option<u64>,
    dry_run: bool,
) -> Result<Value> {
    if dry_run {
        return Ok(serde_json::json!({
            "ok": true,
            "dry_run": true,
            "data": { "txHash": "0x0000000000000000000000000000000000000000000000000000000000000000" },
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
    let from_str;
    if let Some(f) = from {
        from_str = f.to_string();
        args.extend_from_slice(&["--from", &from_str]);
    }

    let output = Command::new("onchainos").args(&args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|_| serde_json::json!({"ok": false, "error": stdout.to_string()}));
    Ok(result)
}

/// Extract txHash from onchainos response
pub fn extract_tx_hash(result: &Value) -> String {
    result["data"]["swapTxHash"]
        .as_str()
        .or_else(|| result["data"]["txHash"].as_str())
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("pending")
        .to_string()
}

/// ERC-20 approve calldata builder
/// selector: 0x095ea7b3
pub fn build_approve_calldata(spender: &str, amount: u128) -> String {
    let spender_padded = format!("{:0>64}", spender.trim_start_matches("0x").to_lowercase());
    let amount_hex = format!("{:064x}", amount);
    format!("0x095ea7b3{}{}", spender_padded, amount_hex)
}

/// ERC-20 approve via contract-call
pub async fn erc20_approve(
    chain_id: u64,
    token_addr: &str,
    spender: &str,
    amount: u128,
    from: Option<&str>,
    dry_run: bool,
) -> Result<Value> {
    let calldata = build_approve_calldata(spender, amount);
    wallet_contract_call(chain_id, token_addr, &calldata, from, None, dry_run).await
}

/// Build Teller.deposit calldata
/// deposit(address depositAsset, uint256 assets, uint256 minSharesOut, address receiver)
/// selector: 0x8b6099db
pub fn build_deposit_calldata(
    deposit_asset: &str,
    assets: u128,
    min_shares_out: u128,
    receiver: &str,
) -> String {
    let asset_padded = format!("{:0>64}", deposit_asset.trim_start_matches("0x").to_lowercase());
    let assets_hex = format!("{:064x}", assets);
    let min_shares_hex = format!("{:064x}", min_shares_out);
    let receiver_padded = format!("{:0>64}", receiver.trim_start_matches("0x").to_lowercase());
    format!(
        "0x8b6099db{}{}{}{}",
        asset_padded, assets_hex, min_shares_hex, receiver_padded
    )
}

/// Build Teller.bulkWithdraw calldata for a single withdrawal
/// bulkWithdraw(address withdrawAsset, uint256[] shares, uint256[] minAssetsOut, address[] receivers)
/// selector: 0x8432f02b
///
/// ABI encoding for single-entry arrays (all dynamic types):
///   param1: withdrawAsset (32 bytes address)
///   param2: offset to shares[] = 0x80 (4 params * 32 bytes = 128 bytes = 0x80)
///   param3: offset to minAssetsOut[] = 0xC0
///   param4: offset to receivers[] = 0x100
///   shares[]: length=1, value
///   minAssetsOut[]: length=1, value=0
///   receivers[]: length=1, address
pub fn build_bulk_withdraw_calldata(
    withdraw_asset: &str,
    shares: u128,
    min_assets_out: u128,
    receiver: &str,
) -> String {
    let asset_padded = format!("{:0>64}", withdraw_asset.trim_start_matches("0x").to_lowercase());
    let shares_hex = format!("{:064x}", shares);
    let min_assets_hex = format!("{:064x}", min_assets_out);
    let receiver_padded = format!("{:0>64}", receiver.trim_start_matches("0x").to_lowercase());

    // Dynamic array offsets: each of the 4 params takes 32 bytes in the static section
    // param1 (address) at offset 0 — static, 32 bytes
    // param2 (array offset) at offset 32 — points to start of shares[] data
    // param3 (array offset) at offset 64 — points to start of minAssetsOut[] data
    // param4 (array offset) at offset 96 — points to start of receivers[] data
    // After static section (4*32=128 bytes), dynamic data starts:
    //   offset 128 (0x80): shares[] length + data (2*32 = 64 bytes)
    //   offset 192 (0xC0): minAssetsOut[] length + data (2*32 = 64 bytes)
    //   offset 256 (0x100): receivers[] length + data (2*32 = 64 bytes)

    format!(
        "0x8432f02b\
        {}\
        0000000000000000000000000000000000000000000000000000000000000080\
        00000000000000000000000000000000000000000000000000000000000000c0\
        0000000000000000000000000000000000000000000000000000000000000100\
        0000000000000000000000000000000000000000000000000000000000000001\
        {}\
        0000000000000000000000000000000000000000000000000000000000000001\
        {}\
        0000000000000000000000000000000000000000000000000000000000000001\
        {}",
        asset_padded, shares_hex, min_assets_hex, receiver_padded
    )
}
