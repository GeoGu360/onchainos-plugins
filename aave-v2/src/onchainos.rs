use anyhow::Context;
use serde_json::Value;
use std::process::Command;

/// Build a base Command for onchainos, explicitly adding ~/.local/bin to PATH.
fn base_cmd() -> Command {
    let mut cmd = Command::new("onchainos");
    let home = std::env::var("HOME").unwrap_or_default();
    let existing_path = std::env::var("PATH").unwrap_or_default();
    let path = format!("{}/.local/bin:{}", home, existing_path);
    cmd.env("PATH", path);
    cmd
}

/// Run a Command and return its stdout as a parsed JSON Value.
/// Handles exit code 2 (onchainos confirming response): retries with --force.
fn run_cmd(cmd: Command) -> anyhow::Result<Value> {
    let mut cmd = cmd;
    let output = cmd.output().context("Failed to spawn onchainos process")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let exit_code = output.status.code().unwrap_or(-1);

    if exit_code == 2 {
        let confirming: Value = serde_json::from_str(stdout.trim())
            .unwrap_or(serde_json::json!({"confirming": true}));
        if confirming.get("confirming").and_then(|v| v.as_bool()).unwrap_or(false) {
            let mut force_cmd = cmd;
            force_cmd.arg("--force");
            let force_output = force_cmd.output().context("Failed to spawn onchainos --force process")?;
            let force_stdout = String::from_utf8_lossy(&force_output.stdout);
            return serde_json::from_str(force_stdout.trim())
                .with_context(|| format!("Failed to parse onchainos --force JSON output: {}", force_stdout.trim()));
        }
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "onchainos exited with status {}: stderr={} stdout={}",
            exit_code,
            stderr.trim(),
            stdout.trim()
        );
    }
    serde_json::from_str(stdout.trim())
        .with_context(|| format!("Failed to parse onchainos JSON output: {}", stdout.trim()))
}

/// Get DeFi positions for a wallet address on a given chain.
pub fn defi_positions(chain_id: u64, wallet_addr: &str) -> anyhow::Result<Value> {
    let chain_name = chain_id_to_name(chain_id);
    let mut cmd = base_cmd();
    cmd.args([
        "defi",
        "positions",
        "--address",
        wallet_addr,
        "--chains",
        chain_name,
    ]);
    run_cmd(cmd)
}

/// Resolve a token symbol or address to (contract_address, decimals).
/// If `asset` is already a 0x-prefixed 42-char hex address, returns it as-is with decimals=18.
pub fn resolve_token(asset: &str, chain_id: u64) -> anyhow::Result<(String, u8)> {
    if asset.starts_with("0x") && asset.len() == 42 {
        return Ok((asset.to_lowercase(), 18));
    }
    let chain_name = chain_id_to_name(chain_id);
    let mut cmd = base_cmd();
    cmd.args(["token", "search", "--query", asset, "--chain", chain_name]);
    let result = run_cmd(cmd)?;

    let tokens = result
        .as_array()
        .or_else(|| result.get("data").and_then(|d| d.as_array()))
        .ok_or_else(|| anyhow::anyhow!("No tokens found for symbol '{}' on chain {}", asset, chain_id))?;

    let first = tokens.first().ok_or_else(|| {
        anyhow::anyhow!("No token match for '{}' on chain {}", asset, chain_id)
    })?;

    let addr = first["tokenContractAddress"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing tokenContractAddress in token search result"))?
        .to_lowercase();

    let decimals = first["decimal"]
        .as_str()
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(18);

    Ok((addr, decimals))
}

/// Resolve the active wallet address from onchainos.
/// If dry_run is true, returns the zero address.
pub fn resolve_wallet(chain_id: u64, dry_run: bool) -> anyhow::Result<String> {
    if dry_run {
        return Ok("0x0000000000000000000000000000000000000000".to_string());
    }
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

/// Submit a contract call via onchainos wallet contract-call.
/// Always passes --force to avoid interactive confirmation prompts.
pub fn wallet_contract_call(
    chain_id: u64,
    to: &str,
    input_data: &str,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<Value> {
    if dry_run {
        let from_str = from.unwrap_or("0x0000000000000000000000000000000000000000");
        let cmd_str = format!(
            "onchainos wallet contract-call --chain {} --to {} --input-data {} --from {} --dry-run --force",
            chain_id, to, input_data, from_str
        );
        eprintln!("[dry-run] would execute: {}", cmd_str);
        return Ok(serde_json::json!({
            "ok": true,
            "dryRun": true,
            "simulatedCommand": cmd_str
        }));
    }

    let mut cmd = base_cmd();
    cmd.args([
        "wallet",
        "contract-call",
        "--chain",
        &chain_id.to_string(),
        "--to",
        to,
        "--input-data",
        input_data,
        "--force",
    ]);
    if let Some(addr) = from {
        cmd.args(["--from", addr]);
    }
    run_cmd(cmd)
}

/// Approve an ERC-20 token spend via wallet contract-call.
#[allow(dead_code)]
pub fn erc20_approve(
    chain_id: u64,
    token: &str,
    spender: &str,
    amount: u128,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<Value> {
    let calldata = crate::calldata::encode_erc20_approve(spender, amount)
        .map_err(|e| anyhow::anyhow!("Failed to encode approve calldata: {}", e))?;
    wallet_contract_call(chain_id, token, &calldata, from, dry_run)
}

/// Get wallet balance for the active wallet.
#[allow(dead_code)]
pub fn wallet_balance(chain_id: u64) -> anyhow::Result<Value> {
    let mut cmd = base_cmd();
    cmd.args([
        "wallet",
        "balance",
        "--chain",
        &chain_id.to_string(),
        "--output",
        "json",
    ]);
    run_cmd(cmd)
}

pub fn chain_id_to_name(chain_id: u64) -> &'static str {
    match chain_id {
        1 => "ethereum",
        137 => "polygon",
        42161 => "arbitrum",
        8453 => "base",
        56 => "bsc",
        _ => "ethereum",
    }
}
