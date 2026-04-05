/// onchainos CLI wrapper for EVM (Scroll) contract calls

use anyhow::Result;
use serde_json::Value;
use std::process::Command;

/// Resolve active EVM wallet address via `onchainos wallet addresses`
/// Parses data.evm[].address entries
pub fn resolve_wallet(chain_id: u64) -> Result<String> {
    let output = Command::new("onchainos")
        .args(["wallet", "addresses"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout)
        .map_err(|e| anyhow::anyhow!("Failed to parse onchainos response: {} — got: {}", e, &stdout[..stdout.len().min(200)]))?;

    // Try data.evm[] array
    if let Some(evm_arr) = json["data"]["evm"].as_array() {
        for entry in evm_arr {
            if let Some(addr) = entry["address"].as_str() {
                if !addr.is_empty() {
                    return Ok(addr.to_string());
                }
            }
        }
    }

    // Fallback: wallet balance for chain
    let chain_str = chain_id.to_string();
    let output2 = Command::new("onchainos")
        .args(["wallet", "balance", "--chain", &chain_str])
        .output()?;
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    let json2: Value = serde_json::from_str(&stdout2)
        .unwrap_or(serde_json::json!({}));

    if let Some(addr) = json2["data"]["details"]
        .get(0)
        .and_then(|d| d["tokenAssets"].get(0))
        .and_then(|t| t["address"].as_str())
    {
        if !addr.is_empty() {
            return Ok(addr.to_string());
        }
    }
    if let Some(addr) = json2["data"]["address"].as_str() {
        if !addr.is_empty() {
            return Ok(addr.to_string());
        }
    }

    anyhow::bail!("Could not resolve wallet address. Is onchainos logged in?")
}

/// Call onchainos wallet contract-call for EVM
/// dry_run=true returns simulated response without broadcasting
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

/// Build ERC-20 approve calldata
/// approve(address,uint256) selector: 0x095ea7b3
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

/// Build supply(address,address,uint256) calldata
/// selector: 0x0c0a769b
pub fn build_supply_calldata(user: &str, borrow_token: &str, amount: u128) -> String {
    let user_padded = format!("{:0>64}", user.trim_start_matches("0x").to_lowercase());
    let token_padded = format!("{:0>64}", borrow_token.trim_start_matches("0x").to_lowercase());
    let amount_hex = format!("{:064x}", amount);
    format!("0x0c0a769b{}{}{}", user_padded, token_padded, amount_hex)
}

/// Build withdrawBorrowToken(address,uint256) calldata
/// selector: 0xa56c8ff7
pub fn build_withdraw_calldata(borrow_token: &str, amount: u128) -> String {
    let token_padded = format!("{:0>64}", borrow_token.trim_start_matches("0x").to_lowercase());
    let amount_hex = format!("{:064x}", amount);
    format!("0xa56c8ff7{}{}", token_padded, amount_hex)
}

/// Build repay(address,address,uint256) calldata
/// selector: 0x1da649cf
pub fn build_repay_calldata(user_safe: &str, token: &str, amount: u128) -> String {
    let user_padded = format!("{:0>64}", user_safe.trim_start_matches("0x").to_lowercase());
    let token_padded = format!("{:0>64}", token.trim_start_matches("0x").to_lowercase());
    let amount_hex = format!("{:064x}", amount);
    format!("0x1da649cf{}{}{}", user_padded, token_padded, amount_hex)
}
