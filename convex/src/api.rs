use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::config::CURVE_API_BASE;

fn build_client() -> reqwest::Client {
    let mut builder = reqwest::Client::builder();
    if let Ok(proxy_url) = std::env::var("HTTPS_PROXY")
        .or_else(|_| std::env::var("https_proxy"))
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .or_else(|_| std::env::var("http_proxy"))
    {
        if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
            builder = builder.proxy(proxy);
        }
    }
    builder.build().unwrap_or_default()
}

/// A Curve/Convex pool entry from the Curve API
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurvePool {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub coins: Vec<PoolCoin>,
    #[serde(default)]
    pub total_supply: Option<Value>,
    #[serde(default)]
    pub usd_total: Option<f64>,
    #[serde(default)]
    pub apy: Option<f64>,
    #[serde(default)]
    pub virtual_price: Option<Value>,
    #[serde(default)]
    pub gauge_address: Option<String>,
    #[serde(default)]
    pub gauge_rewards_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PoolCoin {
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub decimals: Value,
    #[serde(default)]
    pub usd_price: Option<f64>,
}

/// Fetch pools from a Curve registry
pub async fn fetch_curve_pools(registry: &str) -> Result<Vec<CurvePool>> {
    let url = format!("{}/getPools/ethereum/{}", CURVE_API_BASE, registry);
    let client = build_client();
    let resp: Value = client.get(&url)
        .header("User-Agent", "convex-plugin/0.1")
        .send()
        .await?
        .json()
        .await?;

    let pools = resp["data"]["poolData"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let mut result = Vec::new();
    for pool_val in pools {
        if let Ok(pool) = serde_json::from_value::<CurvePool>(pool_val) {
            result.push(pool);
        }
    }
    Ok(result)
}

/// Pool summary for display
#[derive(Debug, Serialize)]
pub struct PoolSummary {
    pub name: String,
    pub address: String,
    pub coins: Vec<String>,
    pub tvl_usd: String,
    pub apy_pct: String,
    pub registry: String,
}

/// Convert raw CurvePool to display summary
pub fn to_summary(pool: &CurvePool, registry: &str) -> PoolSummary {
    let coins: Vec<String> = pool.coins.iter().map(|c| c.symbol.clone()).collect();
    let tvl = pool.usd_total.map(|v| format!("${:.0}", v)).unwrap_or_else(|| "N/A".to_string());
    let apy = pool.apy.map(|v| format!("{:.2}%", v)).unwrap_or_else(|| "N/A".to_string());
    PoolSummary {
        name: pool.name.clone(),
        address: pool.address.clone(),
        coins,
        tvl_usd: tvl,
        apy_pct: apy,
        registry: registry.to_string(),
    }
}
