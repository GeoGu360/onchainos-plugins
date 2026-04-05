// Sanctum API client — REST API calls to Extra API and S Router API
//
// Verified response shapes (2026-04-05):
//
// GET https://extra-api.sanctum.so/v1/sol-value/current?lst=INF
// → {"solValues":{"INF":"1407837461"},"errs":{}}
//
// GET https://extra-api.sanctum.so/v1/apy/latest?lst=INF
// → {"apys":{"INF":0.0},"errs":{}}
//
// GET https://extra-api.sanctum.so/v1/infinity/allocation/current
// → {"infinity":{"<mint>":{"amt":"...","solValue":"...","share":0.15}}}
//
// GET https://sanctum-s-api.fly.dev/v2/swap/quote?input=...&outputLstMint=...&amount=...
// → {"inAmount":"...","outAmount":"...","swapSrc":"SPool","fees":[...]}
//
// POST https://sanctum-s-api.fly.dev/v1/swap → {"tx":"<base64>"}
// POST https://sanctum-s-api.fly.dev/v1/liquidity/add → {"tx":"<base64>"}
// POST https://sanctum-s-api.fly.dev/v1/liquidity/remove → {"tx":"<base64>"}

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::config::{EXTRA_API_BASE, ROUTER_API_BASE};

// ──────────────────────── Extra API responses ────────────────────────

#[derive(Debug, Deserialize)]
pub struct SolValueResp {
    #[serde(rename = "solValues")]
    pub sol_values: HashMap<String, String>,
    pub errs: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct ApyResp {
    pub apys: HashMap<String, f64>,
    pub errs: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct InfAllocResp {
    #[serde(default)]
    pub infinity: HashMap<String, InfLstAlloc>,
    // When endpoint has no data, it returns {"message":null,"code":"NO_DATA_AVAILABLE"}
    #[serde(default)]
    pub code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TvlResp {
    #[serde(default)]
    pub tvls: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InfLstAlloc {
    pub amt: String,
    #[serde(rename = "solValue")]
    pub sol_value: String,
    pub share: f64,
}

// LSTs listing
#[derive(Debug, Deserialize)]
pub struct LstsResp {
    pub lsts: Vec<LstInfo>,
}

#[derive(Debug, Deserialize)]
pub struct LstInfo {
    pub mint: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
}

// ──────────────────────── Router API responses ────────────────────────

#[derive(Debug, Deserialize)]
pub struct SwapQuoteRespV2 {
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "swapSrc")]
    pub swap_src: String,
    pub fees: Vec<FeeEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FeeEntry {
    pub code: String,
    pub rate: String,
    pub amt: String,
    pub mint: String,
}

#[derive(Debug, Deserialize)]
pub struct LiquidityAddQuoteResp {
    #[serde(rename = "lpAmount")]
    pub lp_amount: String,
}

#[derive(Debug, Deserialize)]
pub struct LiquidityRemoveQuoteResp {
    #[serde(rename = "lstAmount")]
    pub lst_amount: String,
}

#[derive(Debug, Deserialize)]
pub struct TxResp {
    pub tx: String, // base64-encoded VersionedTransaction
}

// ──────────────────────── API functions ────────────────────────

/// Fetch INF SOL value (lamports per 1 INF token in atomic units).
pub async fn get_sol_value(client: &reqwest::Client, lst: &str) -> Result<String> {
    let url = format!("{}/v1/sol-value/current?lst={}", EXTRA_API_BASE, lst);
    let resp: SolValueResp = client.get(&url).send().await?.json().await?;
    resp.sol_values
        .get(lst)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("SOL value not found for {}", lst))
}

/// Fetch latest APY for an LST.
pub async fn get_apy(client: &reqwest::Client, lsts: &[&str]) -> Result<ApyResp> {
    let lst_params = lsts.iter().map(|l| format!("lst={}", l)).collect::<Vec<_>>().join("&");
    let url = format!("{}/v1/apy/latest?{}", EXTRA_API_BASE, lst_params);
    let resp: ApyResp = client.get(&url).send().await?.json().await?;
    Ok(resp)
}

/// Fetch Infinity pool allocation.
pub async fn get_infinity_allocation(client: &reqwest::Client) -> Result<InfAllocResp> {
    let url = format!("{}/v1/infinity/allocation/current", EXTRA_API_BASE);
    let resp: InfAllocResp = client.get(&url).send().await?.json().await?;
    Ok(resp)
}

/// Fetch TVL for specified LSTs.
pub async fn get_tvl(client: &reqwest::Client, lst: &str) -> Result<String> {
    let url = format!("{}/v1/tvl/current?lst={}", EXTRA_API_BASE, lst);
    let resp: TvlResp = client.get(&url).send().await?.json().await?;
    resp.tvls
        .get(lst)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("TVL not found for {}", lst))
}

/// Fetch all available LSTs from Sanctum.
pub async fn get_lsts(client: &reqwest::Client) -> Result<Vec<LstInfo>> {
    let url = format!("{}/v1/lsts", EXTRA_API_BASE);
    let resp: LstsResp = client.get(&url).send().await?.json().await?;
    Ok(resp.lsts)
}

/// Get swap quote from Router API (v2).
/// input: input LST mint (or wSOL mint for native SOL)
/// output_lst_mint: output LST mint
/// amount: input amount in raw atomics (U64 as string)
pub async fn get_swap_quote(
    client: &reqwest::Client,
    input: &str,
    output_lst_mint: &str,
    amount: u64,
    mode: &str,
) -> Result<SwapQuoteRespV2> {
    let url = format!(
        "{}/v2/swap/quote?input={}&outputLstMint={}&amount={}&mode={}",
        ROUTER_API_BASE, input, output_lst_mint, amount, mode
    );
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Swap quote failed ({}): {}", status, body);
    }
    Ok(resp.json::<SwapQuoteRespV2>().await?)
}

/// Get liquidity add quote.
pub async fn get_liquidity_add_quote(
    client: &reqwest::Client,
    lst_mint: &str,
    amount: u64,
) -> Result<LiquidityAddQuoteResp> {
    let url = format!(
        "{}/v1/liquidity/add/quote?lstMint={}&amount={}",
        ROUTER_API_BASE, lst_mint, amount
    );
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Liquidity add quote failed ({}): {}", status, body);
    }
    Ok(resp.json::<LiquidityAddQuoteResp>().await?)
}

/// Get liquidity remove quote.
pub async fn get_liquidity_remove_quote(
    client: &reqwest::Client,
    lst_mint: &str,
    lp_amount: u64,
) -> Result<LiquidityRemoveQuoteResp> {
    let url = format!(
        "{}/v1/liquidity/remove/quote?lstMint={}&amount={}",
        ROUTER_API_BASE, lst_mint, lp_amount
    );
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Liquidity remove quote failed ({}): {}", status, body);
    }
    Ok(resp.json::<LiquidityRemoveQuoteResp>().await?)
}

/// Execute swap — returns base64 serialized transaction.
pub async fn execute_swap(
    client: &reqwest::Client,
    input: &str,
    output_lst_mint: &str,
    amount: u64,
    quoted_amount: u64,
    signer: &str,
    mode: &str,
) -> Result<String> {
    let url = format!("{}/v1/swap", ROUTER_API_BASE);
    let body = serde_json::json!({
        "input": input,
        "outputLstMint": output_lst_mint,
        "amount": amount.to_string(),
        "quotedAmount": quoted_amount.to_string(),
        "mode": mode,
        "signer": signer,
        "swapSrc": "SPool"
    });
    let resp = client.post(&url).json(&body).send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let err_body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Swap transaction failed ({}): {}", status, err_body);
    }
    let tx_resp: TxResp = resp.json().await?;
    Ok(tx_resp.tx)
}

/// Execute liquidity add — returns base64 serialized transaction.
pub async fn execute_liquidity_add(
    client: &reqwest::Client,
    lst_mint: &str,
    amount: u64,
    quoted_amount: u64,
    signer: &str,
) -> Result<String> {
    let url = format!("{}/v1/liquidity/add", ROUTER_API_BASE);
    let body = serde_json::json!({
        "lstMint": lst_mint,
        "amount": amount.to_string(),
        "quotedAmount": quoted_amount.to_string(),
        "signer": signer
    });
    let resp = client.post(&url).json(&body).send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let err_body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Liquidity add transaction failed ({}): {}", status, err_body);
    }
    let tx_resp: TxResp = resp.json().await?;
    Ok(tx_resp.tx)
}

/// Execute liquidity remove — returns base64 serialized transaction.
pub async fn execute_liquidity_remove(
    client: &reqwest::Client,
    lst_mint: &str,
    lp_amount: u64,
    quoted_amount: u64,
    signer: &str,
) -> Result<String> {
    let url = format!("{}/v1/liquidity/remove", ROUTER_API_BASE);
    let body = serde_json::json!({
        "lstMint": lst_mint,
        "amount": lp_amount.to_string(),
        "quotedAmount": quoted_amount.to_string(),
        "signer": signer
    });
    let resp = client.post(&url).json(&body).send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let err_body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Liquidity remove transaction failed ({}): {}", status, err_body);
    }
    let tx_resp: TxResp = resp.json().await?;
    Ok(tx_resp.tx)
}

/// Convert UI amount to raw atomics (9 decimals for Solana LSTs).
pub fn ui_to_atomics(amount: f64, decimals: u32) -> u64 {
    (amount * 10f64.powi(decimals as i32)).round() as u64
}

/// Convert raw atomics to UI amount.
pub fn atomics_to_ui(atomics: u64, decimals: u32) -> f64 {
    atomics as f64 / 10f64.powi(decimals as i32)
}

/// Apply slippage to get minimum acceptable out amount.
pub fn apply_slippage(amount: u64, slippage_pct: f64) -> u64 {
    let factor = 1.0 - slippage_pct / 100.0;
    (amount as f64 * factor).floor() as u64
}

/// Resolve LST mint from symbol or passthrough if already a base58 address.
pub fn resolve_lst_mint(input: &str) -> &str {
    match input.to_lowercase().as_str() {
        "sol" | "wsol" => crate::config::WSOL_MINT,
        "inf" | "infinity" => crate::config::INF_MINT,
        "jitosol" => crate::config::JITO_SOL_MINT,
        "msol" => crate::config::MSOL_MINT,
        _ => input, // assume it's already a mint address
    }
}
