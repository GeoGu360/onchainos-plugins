use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const ALLBRIDGE_API: &str = "https://allbridgeapi.net";

fn build_client() -> Result<Client> {
    let mut builder = Client::builder();
    // Respect system proxy settings
    if let Ok(proxy_url) = std::env::var("HTTPS_PROXY").or_else(|_| std::env::var("https_proxy")) {
        builder = builder.proxy(reqwest::Proxy::https(&proxy_url)?);
    }
    if let Ok(proxy_url) = std::env::var("HTTP_PROXY").or_else(|_| std::env::var("http_proxy")) {
        builder = builder.proxy(reqwest::Proxy::http(&proxy_url)?);
    }
    Ok(builder.build()?)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenInfo {
    /// Token contract address on this chain
    pub address: String,
    pub symbol: String,
    pub precision: Option<u8>,
    #[serde(rename = "minFee")]
    pub min_fee: Option<String>,
    #[serde(rename = "tokenSource")]
    pub token_source: Option<String>,
    #[serde(rename = "isBase")]
    pub is_base: Option<bool>,
    #[serde(rename = "isWrapped")]
    pub is_wrapped: Option<bool>,
}

#[derive(Deserialize)]
struct ChainData {
    #[serde(rename = "confirmations")]
    _confirmations: Option<u32>,
    pub tokens: Vec<TokenInfo>,
}

/// GET /token-info — returns map of chainId -> [TokenInfo]
/// Actual response: { "BSC": { "confirmations": 15, "tokens": [...] }, ... }
pub async fn get_token_info() -> Result<HashMap<String, Vec<TokenInfo>>> {
    let client = build_client()?;
    let url = format!("{}/token-info", ALLBRIDGE_API);
    let resp = client
        .get(&url)
        .header("User-Agent", "allbridge-classic-plugin/0.1.0")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow!("API error {}: {}", resp.status(), resp.text().await?));
    }

    let raw_text = resp.text().await?;

    // Parse as generic Value first to handle variable structure
    let data: HashMap<String, serde_json::Value> = serde_json::from_str(&raw_text)
        .map_err(|e| anyhow!("Failed to parse token-info response: {}", e))?;

    let mut result: HashMap<String, Vec<TokenInfo>> = HashMap::new();

    for (chain, chain_val) in data {
        // Try nested structure: { "confirmations": N, "tokens": [...] }
        if let Ok(chain_data) = serde_json::from_value::<ChainData>(chain_val.clone()) {
            result.insert(chain, chain_data.tokens);
        } else if let Ok(tokens) = serde_json::from_value::<Vec<TokenInfo>>(chain_val) {
            // Fallback: direct array
            result.insert(chain, tokens);
        }
    }

    Ok(result)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SignResponse {
    #[serde(rename = "lockId")]
    pub lock_id: Option<String>,
    pub block: Option<String>,
    pub source: Option<String>,
    pub amount: Option<String>,
    pub destination: Option<String>,
    pub recipient: Option<String>,
    #[serde(rename = "tokenSource")]
    pub token_source: Option<String>,
    #[serde(rename = "tokenSourceAddress")]
    pub token_source_address: Option<String>,
    pub signature: Option<String>,
}

/// GET /sign/{transactionId} — get bridge confirmation and signature
pub async fn get_sign(transaction_id: &str) -> Result<SignResponse> {
    let client = build_client()?;
    let url = format!("{}/sign/{}", ALLBRIDGE_API, transaction_id);
    let resp = client
        .get(&url)
        .header("User-Agent", "allbridge-classic-plugin/0.1.0")
        .send()
        .await?;

    let status = resp.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(anyhow!("Transaction not yet confirmed or not found. Please wait for the lock transaction to be confirmed on the source chain."));
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("API error {}: {}", status, body));
    }

    let data: SignResponse = resp.json().await?;
    Ok(data)
}

/// GET /check/{blockchainId}/address/{address}
pub async fn check_address(blockchain_id: &str, address: &str) -> Result<serde_json::Value> {
    let client = build_client()?;
    let url = format!("{}/check/{}/address/{}", ALLBRIDGE_API, blockchain_id, address);
    let resp = client
        .get(&url)
        .header("User-Agent", "allbridge-classic-plugin/0.1.0")
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("API error {}: {}", status, body));
    }

    let data: serde_json::Value = resp.json().await?;
    Ok(data)
}
