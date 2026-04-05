/// markets — list Synthetix V3 Perps markets with funding rates and sizes
use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

use crate::config::{BASE_RPC_URL, PERPS_MARKET_PROXY};
use crate::rpc::{decode_int256, decode_uint256_array, decode_uint256_as_u128, eth_call};

// Verified selectors (cast sig):
// getMarkets()                  → 0xec2c9016
// getMarketSummary(uint128)     → 0x41c2e8bd
// currentFundingRate(uint128)   → 0xd435b2a2

#[derive(Serialize)]
pub struct Market {
    pub market_id: String,
    pub symbol: String,
    pub skew: String,
    pub size: String,
    pub max_open_interest: String,
    pub current_funding_rate: String,
    pub current_funding_velocity: String,
}

pub async fn execute(market_id: Option<u128>) -> Result<()> {
    let market_ids: Vec<u128> = if let Some(id) = market_id {
        vec![id]
    } else {
        // getMarkets() -> uint256[]
        let result = eth_call(PERPS_MARKET_PROXY, "0xec2c9016", BASE_RPC_URL).await?;
        decode_uint256_array(&result)
    };

    let mut markets: Vec<Market> = Vec::new();

    // Limit to avoid too many RPC calls — show first 20 if all markets requested
    let ids_to_query: Vec<u128> = if market_id.is_none() && market_ids.len() > 20 {
        market_ids[..20].to_vec()
    } else {
        market_ids.clone()
    };

    let total_markets = market_ids.len();

    for id in &ids_to_query {
        match fetch_market_summary(*id).await {
            Ok(m) => markets.push(m),
            Err(_) => {
                // Many markets require Pyth price feed updates (ERC-7412) — skip silently
                // when listing all markets; only surface if a specific market was requested
                if market_id.is_some() {
                    markets.push(Market {
                        market_id: id.to_string(),
                        symbol: format!("MARKET_{}", id),
                        skew: "N/A (price feed update required)".to_string(),
                        size: "0".to_string(),
                        max_open_interest: "0".to_string(),
                        current_funding_rate: "0".to_string(),
                        current_funding_velocity: "0".to_string(),
                    });
                }
            }
        }
    }

    let output = serde_json::json!({
        "ok": true,
        "chain": 8453,
        "protocol": "Synthetix V3 Perps",
        "total_markets": total_markets,
        "showing": ids_to_query.len(),
        "markets": markets
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Fetch summary for a single market
async fn fetch_market_summary(market_id: u128) -> Result<Market> {
    // Encode marketId as uint128 padded to 32 bytes
    let id_hex = format!("{:064x}", market_id);

    // getMarketSummary(uint128) -> tuple(int256 skew, uint256 maxOpenInterest, uint256 size, int256 currentFundingRate, int256 currentFundingVelocity, bytes32 feedId)
    let calldata_summary = format!("0x41c2e8bd{}", id_hex);
    let summary_raw = eth_call(PERPS_MARKET_PROXY, &calldata_summary, BASE_RPC_URL).await?;

    // ABI decode tuple: 6 x 32 bytes = 192 bytes = 384 hex chars (after 0x)
    let clean = summary_raw.trim_start_matches("0x");
    let (skew_raw, max_oi_raw, size_raw, funding_rate_raw, funding_vel_raw) =
        if clean.len() >= 320 {
            (
                &clean[0..64],
                &clean[64..128],
                &clean[128..192],
                &clean[192..256],
                &clean[256..320],
            )
        } else {
            return Ok(Market {
                market_id: market_id.to_string(),
                symbol: format!("MARKET_{}", market_id),
                skew: "0".to_string(),
                size: "0".to_string(),
                max_open_interest: "0".to_string(),
                current_funding_rate: "0".to_string(),
                current_funding_velocity: "0".to_string(),
            });
        };

    // Decode values (18 decimals for sizes)
    let skew = decode_signed_int256_str(skew_raw);
    let max_oi = decode_uint256_as_u128(&format!("0x{}", max_oi_raw));
    let size = decode_uint256_as_u128(&format!("0x{}", size_raw));
    let funding_rate = decode_signed_int256_str(funding_rate_raw);
    let funding_vel = decode_signed_int256_str(funding_vel_raw);

    // Format: divide by 1e18 for human-readable values
    let skew_f = format!("{:.4}", decode_signed_float(skew_raw));
    let size_f = format!("{:.4}", max_oi as f64 / 1e18);
    let max_oi_f = format!("{:.4}", max_oi as f64 / 1e18);
    let funding_f = format!("{:.8}", decode_signed_float(funding_rate_raw));
    let funding_vel_f = format!("{:.8}", decode_signed_float(funding_vel_raw));

    // Get symbol from known market IDs
    let symbol = market_id_to_symbol(market_id);

    Ok(Market {
        market_id: market_id.to_string(),
        symbol,
        skew: skew_f,
        size: size_f,
        max_open_interest: max_oi_f,
        current_funding_rate: funding_f,
        current_funding_velocity: funding_vel_f,
    })
}

/// Decode signed int256 (hex without 0x prefix, 64 chars) to a formatted float
fn decode_signed_float(hex: &str) -> f64 {
    let bytes = u128::from_str_radix(hex, 16).unwrap_or(0);
    let high = u8::from_str_radix(&hex[..2], 16).unwrap_or(0);
    if high >= 0x80 {
        // Negative
        let val = (u128::MAX - bytes + 1) as f64;
        -val / 1e18
    } else {
        bytes as f64 / 1e18
    }
}

fn decode_signed_int256_str(hex: &str) -> String {
    format!("{:.8}", decode_signed_float(hex))
}

/// Map well-known market IDs to symbols
fn market_id_to_symbol(id: u128) -> String {
    match id {
        100 => "ETH".to_string(),
        200 => "BTC".to_string(),
        300 => "SNX".to_string(),
        400 => "SOL".to_string(),
        500 => "WIF".to_string(),
        600 => "W".to_string(),
        700 => "ENA".to_string(),
        800 => "DOGE".to_string(),
        900 => "AVAX".to_string(),
        1000 => "OP".to_string(),
        1100 => "ORDI".to_string(),
        1200 => "PEPE".to_string(),
        1300 => "RUNE".to_string(),
        1400 => "BONK".to_string(),
        1500 => "FTM".to_string(),
        1600 => "ARB".to_string(),
        1700 => "MATIC".to_string(),
        1800 => "BNB".to_string(),
        1900 => "LINK".to_string(),
        2000 => "PENDLE".to_string(),
        _ => format!("MARKET_{}", id),
    }
}
