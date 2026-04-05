/// positions — query open Perps positions for a given account ID
use anyhow::Result;
use serde::Serialize;

use crate::config::{BASE_RPC_URL, PERPS_MARKET_PROXY};
use crate::rpc::{decode_uint256_array, decode_uint256_as_u128, eth_call};

// Verified selectors (cast sig):
// getAccountOpenPositions(uint128)     → 0x35254238
// getOpenPosition(uint128,uint128)     → 0x22a73967
// getAvailableMargin(uint128)          → 0x0a7dad2d
// getCollateralAmount(uint128,uint128) → selector below

#[derive(Serialize)]
pub struct OpenPosition {
    pub market_id: String,
    pub symbol: String,
    pub total_pnl: String,
    pub accrued_funding: String,
    pub position_size: String,
}

#[derive(Serialize)]
pub struct PositionsResult {
    pub ok: bool,
    pub chain: u64,
    pub account_id: String,
    pub available_margin: String,
    pub open_positions: Vec<OpenPosition>,
}

pub async fn execute(account_id: u128) -> Result<()> {
    let id_hex = format!("{:064x}", account_id);

    // getAccountOpenPositions(uint128) → uint256[]
    let calldata = format!("0x35254238{}", id_hex);
    let raw = eth_call(PERPS_MARKET_PROXY, &calldata, BASE_RPC_URL).await?;
    let open_market_ids = decode_uint256_array(&raw);

    // getAvailableMargin(uint128) → int256
    let margin_calldata = format!("0x0a7dad2d{}", id_hex);
    let margin_raw = eth_call(PERPS_MARKET_PROXY, &margin_calldata, BASE_RPC_URL).await?;
    let available_margin = decode_signed_float_from_hex(&margin_raw);

    let mut positions: Vec<OpenPosition> = Vec::new();

    for market_id in &open_market_ids {
        // getOpenPosition(uint128 accountId, uint128 marketId) → (int256 totalPnl, int256 accruedFunding, int128 positionSize, uint256 owedInterest)
        let market_hex = format!("{:064x}", market_id);
        let pos_calldata = format!("0x22a73967{}{}", id_hex, market_hex);
        let pos_raw = eth_call(PERPS_MARKET_PROXY, &pos_calldata, BASE_RPC_URL).await?;

        let clean = pos_raw.trim_start_matches("0x");
        if clean.len() < 128 {
            continue;
        }

        let pnl_raw = &clean[0..64];
        let funding_raw = &clean[64..128];
        let size_raw = if clean.len() >= 192 { &clean[128..192] } else { "0" };

        let total_pnl = decode_signed_float_from_raw(pnl_raw);
        let accrued_funding = decode_signed_float_from_raw(funding_raw);
        let position_size = decode_signed_int128_from_raw(size_raw);

        let symbol = market_id_to_symbol(*market_id);

        positions.push(OpenPosition {
            market_id: market_id.to_string(),
            symbol,
            total_pnl: format!("{:.6}", total_pnl),
            accrued_funding: format!("{:.6}", accrued_funding),
            position_size: format!("{:.6}", position_size),
        });
    }

    let result = PositionsResult {
        ok: true,
        chain: 8453,
        account_id: account_id.to_string(),
        available_margin: format!("{:.6}", available_margin),
        open_positions: positions,
    };

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn decode_signed_float_from_hex(hex: &str) -> f64 {
    let clean = hex.trim_start_matches("0x");
    if clean.len() < 2 {
        return 0.0;
    }
    decode_signed_float_from_raw(clean)
}

fn decode_signed_float_from_raw(hex: &str) -> f64 {
    if hex.len() < 2 {
        return 0.0;
    }
    let high = u8::from_str_radix(&hex[..2], 16).unwrap_or(0);
    let val = u128::from_str_radix(hex, 16).unwrap_or(0);
    if high >= 0x80 {
        let neg = (u128::MAX - val + 1) as f64;
        -neg / 1e18
    } else {
        val as f64 / 1e18
    }
}

fn decode_signed_int128_from_raw(hex: &str) -> f64 {
    // int128 is stored in lower 128 bits of the 256-bit slot
    let lower = if hex.len() >= 64 { &hex[hex.len() - 32..] } else { hex };
    if lower.len() < 2 {
        return 0.0;
    }
    let high = u8::from_str_radix(&lower[..2], 16).unwrap_or(0);
    let val = u64::from_str_radix(lower, 16).unwrap_or(0);
    if high >= 0x80 {
        let neg = (u64::MAX - val + 1) as f64;
        -neg / 1e18
    } else {
        val as f64 / 1e18
    }
}

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
        1600 => "ARB".to_string(),
        1800 => "BNB".to_string(),
        1900 => "LINK".to_string(),
        _ => format!("MARKET_{}", id),
    }
}
