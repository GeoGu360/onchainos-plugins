use anyhow::Context;
use serde::Deserialize;

use crate::config::MELLOW_API_BASE;

// ----- Response structs (based on verified API responses) -----

#[derive(Deserialize, Debug, Clone)]
pub struct VaultToken {
    pub address: String,
    pub symbol: String,
    pub decimals: u8,
    #[allow(dead_code)]
    pub wrapper: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct AprBreakdown {
    pub id: String,
    #[serde(rename = "type")]
    pub apr_type: Option<String>,
    pub apr: Option<f64>,
    pub symbol: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct VaultInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub vault_type: Option<String>,
    pub chain_id: u64,
    pub address: String,
    pub symbol: String,
    #[allow(dead_code)]
    pub decimals: u8,
    pub name: String,
    pub layer: Option<String>,
    pub base_token: Option<VaultToken>,
    pub deposit_tokens: Option<Vec<VaultToken>>,
    pub withdraw_tokens: Option<Vec<VaultToken>>,
    pub withdraw_avg_time_seconds: Option<u64>,
    pub apr: Option<f64>,
    pub price: Option<f64>,
    pub tvl_usd: Option<f64>,
    #[allow(dead_code)]
    pub limit_usd: Option<f64>,
    #[allow(dead_code)]
    pub total_supply: Option<String>,
}

/// Fetch all vaults from Mellow API, optionally filtered by chain_id.
pub async fn fetch_vaults(chain_id: Option<u64>) -> anyhow::Result<Vec<VaultInfo>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let url = format!("{}/vaults", MELLOW_API_BASE);
    let resp: Vec<VaultInfo> = client
        .get(&url)
        .send()
        .await
        .context("Failed to fetch Mellow vaults API")?
        .json()
        .await
        .context("Failed to parse Mellow vaults response")?;
    if let Some(cid) = chain_id {
        Ok(resp.into_iter().filter(|v| v.chain_id == cid).collect())
    } else {
        Ok(resp)
    }
}
