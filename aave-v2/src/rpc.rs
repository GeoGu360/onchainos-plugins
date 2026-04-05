use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Raw JSON-RPC request/response
#[derive(Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'a str,
    method: &'a str,
    params: Value,
    id: u64,
}

#[derive(Deserialize)]
struct RpcResponse {
    result: Option<String>,
    error: Option<Value>,
}

/// Poll eth_getTransactionReceipt until the tx is mined (or timeout).
/// Returns true if the tx succeeded (status=0x1), false if reverted.
pub async fn wait_for_tx(rpc_url: &str, tx_hash: &str) -> anyhow::Result<bool> {
    use std::time::{Duration, Instant};
    let deadline = Instant::now() + Duration::from_secs(90);

    loop {
        if Instant::now() > deadline {
            anyhow::bail!("Timeout waiting for tx {} to be mined", tx_hash);
        }

        let req = json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionReceipt",
            "params": [tx_hash],
            "id": 1
        });

        match reqwest::Client::new().post(rpc_url).json(&req).send().await {
            Ok(resp) => {
                if let Ok(body) = resp.json::<Value>().await {
                    let receipt = &body["result"];
                    if !receipt.is_null() {
                        let status = receipt["status"].as_str().unwrap_or("0x1");
                        return Ok(status == "0x1");
                    }
                }
            }
            Err(_) => {}
        }

        tokio::time::sleep(Duration::from_secs(4)).await;
    }
}

/// Perform a raw eth_call against the given RPC endpoint.
/// `to` and `data` are hex strings (0x-prefixed).
pub async fn eth_call(rpc_url: &str, to: &str, data: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let req = RpcRequest {
        jsonrpc: "2.0",
        method: "eth_call",
        params: json!([
            { "to": to, "data": data },
            "latest"
        ]),
        id: 1,
    };
    let resp: RpcResponse = client
        .post(rpc_url)
        .json(&req)
        .send()
        .await
        .context("eth_call HTTP request failed")?
        .json()
        .await
        .context("eth_call response parse failed")?;

    if let Some(err) = resp.error {
        anyhow::bail!("eth_call RPC error: {}", err);
    }
    resp.result
        .ok_or_else(|| anyhow::anyhow!("eth_call returned null result"))
}

/// Resolve the LendingPool address by calling LendingPoolAddressesProvider.getLendingPool()
/// Function: getLendingPool() → selector 0x0261bf8b
/// Verified: keccak256("getLendingPool()") = 0x0261bf8b...
pub async fn get_lending_pool(provider_addr: &str, rpc_url: &str) -> anyhow::Result<String> {
    let data = "0x0261bf8b";
    let hex_result = eth_call(rpc_url, provider_addr, data).await?;
    let addr = decode_address_result(&hex_result)?;
    Ok(addr)
}

/// Account data returned by LendingPool.getUserAccountData(address)
/// Same layout as V3's Pool.getUserAccountData
#[derive(Debug, Clone)]
pub struct UserAccountData {
    pub total_collateral_base: u128,
    pub total_debt_base: u128,
    pub available_borrows_base: u128,
    pub current_liquidation_threshold: u128,
    pub ltv: u128,
    pub health_factor: u128,
}

impl UserAccountData {
    pub fn health_factor_f64(&self) -> f64 {
        self.health_factor as f64 / 1e18
    }

    pub fn health_factor_status(&self) -> &'static str {
        let hf = self.health_factor_f64();
        if hf >= crate::config::HF_WARN_THRESHOLD {
            "safe"
        } else if hf >= 1.05 {
            "warning"
        } else {
            "danger"
        }
    }

    pub fn total_collateral_eth(&self) -> f64 {
        // V2 returns ETH-denominated values (1e18 precision), not USD
        self.total_collateral_base as f64 / 1e18
    }

    pub fn total_debt_eth(&self) -> f64 {
        self.total_debt_base as f64 / 1e18
    }

    pub fn available_borrows_eth(&self) -> f64 {
        self.available_borrows_base as f64 / 1e18
    }
}

/// Call LendingPool.getUserAccountData(address user)
/// Function selector: getUserAccountData(address) → 0xbf92857c
pub async fn get_user_account_data(
    pool_addr: &str,
    user_addr: &str,
    rpc_url: &str,
) -> anyhow::Result<UserAccountData> {
    let addr_bytes = parse_address(user_addr)?;
    let mut data = hex::decode("bf92857c")?;
    data.extend_from_slice(&[0u8; 12]);
    data.extend_from_slice(&addr_bytes);

    let data_hex = format!("0x{}", hex::encode(&data));
    let hex_result = eth_call(rpc_url, pool_addr, &data_hex).await?;

    let raw = strip_0x(&hex_result);
    if raw.len() < 64 * 6 {
        anyhow::bail!(
            "getUserAccountData: short response ({} hex chars, expected {})",
            raw.len(),
            64 * 6
        );
    }

    Ok(UserAccountData {
        total_collateral_base: decode_u128_at(raw, 0)?,
        total_debt_base: decode_u128_at(raw, 1)?,
        available_borrows_base: decode_u128_at(raw, 2)?,
        current_liquidation_threshold: decode_u128_at(raw, 3)?,
        ltv: decode_u128_at(raw, 4)?,
        health_factor: decode_u128_at(raw, 5)?,
    })
}

/// Get ERC-20 token balance: token.balanceOf(account)
/// Function selector: balanceOf(address) → 0x70a08231
#[allow(dead_code)]
pub async fn get_erc20_balance(
    token_addr: &str,
    account: &str,
    rpc_url: &str,
) -> anyhow::Result<u128> {
    let owner = parse_address(account)?;
    let mut data = hex::decode("70a08231")?;
    data.extend_from_slice(&[0u8; 12]);
    data.extend_from_slice(&owner);

    let data_hex = format!("0x{}", hex::encode(&data));
    let hex_result = eth_call(rpc_url, token_addr, &data_hex).await?;
    let raw = strip_0x(&hex_result);
    if raw.len() < 64 {
        anyhow::bail!("balanceOf: short response");
    }
    decode_u128_at(raw, 0)
}

/// Check ERC-20 allowance: token.allowance(owner, spender)
/// Function selector: allowance(address,address) → 0xdd62ed3e
#[allow(dead_code)]
pub async fn get_allowance(
    token_addr: &str,
    owner_addr: &str,
    spender_addr: &str,
    rpc_url: &str,
) -> anyhow::Result<u128> {
    let owner = parse_address(owner_addr)?;
    let spender = parse_address(spender_addr)?;

    let mut data = hex::decode("dd62ed3e")?;
    data.extend_from_slice(&[0u8; 12]);
    data.extend_from_slice(&owner);
    data.extend_from_slice(&[0u8; 12]);
    data.extend_from_slice(&spender);

    let data_hex = format!("0x{}", hex::encode(&data));
    let hex_result = eth_call(rpc_url, token_addr, &data_hex).await?;
    let raw = strip_0x(&hex_result);
    if raw.len() < 64 {
        anyhow::bail!("allowance: short response");
    }
    decode_u128_at(raw, 0)
}

// ── helpers ─────────────────────────────────────────────────────────────────

fn strip_0x(s: &str) -> &str {
    s.strip_prefix("0x").unwrap_or(s)
}

fn decode_address_result(hex_result: &str) -> anyhow::Result<String> {
    let raw = strip_0x(hex_result);
    if raw.len() < 64 {
        anyhow::bail!("decode_address_result: short result '{}'", hex_result);
    }
    let addr_hex = &raw[raw.len() - 40..];
    Ok(format!("0x{}", addr_hex))
}

fn parse_address(addr: &str) -> anyhow::Result<[u8; 20]> {
    let clean = strip_0x(addr);
    if clean.len() != 40 {
        anyhow::bail!("Invalid address (must be 20 bytes / 40 hex chars): {}", addr);
    }
    let bytes = hex::decode(clean).context("Invalid hex address")?;
    let mut out = [0u8; 20];
    out.copy_from_slice(&bytes);
    Ok(out)
}

pub fn decode_u128_at(raw: &str, slot: usize) -> anyhow::Result<u128> {
    let start = slot * 64;
    let end = start + 64;
    if raw.len() < end {
        anyhow::bail!("decode_u128_at: slot {} out of range (raw len {})", slot, raw.len());
    }
    let slot_hex = &raw[start..end];
    let low32 = &slot_hex[32..64];
    let val = u128::from_str_radix(low32, 16)
        .with_context(|| format!("decode_u128_at: invalid hex '{}'", low32))?;
    Ok(val)
}
