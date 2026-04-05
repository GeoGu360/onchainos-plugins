use anyhow::Context;
use serde_json::{json, Value};

use crate::config::get_chain_config;
use crate::rpc;

/// List Aave V2 reserve data.
///
/// Calls LendingPool.getReservesList() to get asset addresses, then
/// LendingPool.getReserveData(address) per reserve.
///
/// Aave V2 DataTypes.ReserveData struct slot layout:
///   Slot 0: configuration (packed bitmask)
///   Slot 1: liquidityIndex (ray = 1e27)
///   Slot 2: variableBorrowIndex (ray)
///   Slot 3: currentLiquidityRate    ← supply APY (ray = 1e27)
///   Slot 4: currentVariableBorrowRate ← variable borrow APY (ray = 1e27)
///   Slot 5: currentStableBorrowRate  ← stable borrow APY (ray = 1e27)
///   Slot 6: lastUpdateTimestamp (uint40)
///   Slot 7: aTokenAddress
///   Slot 8: stableDebtTokenAddress
///   Slot 9: variableDebtTokenAddress
///   Slot 10: interestRateStrategyAddress
///   Slot 11: id (uint8)
///
/// Note: V2 slot layout differs from V3 — rates start at slot 3 (not 2).
pub async fn run(chain_id: u64, asset_filter: Option<&str>) -> anyhow::Result<Value> {
    let cfg = get_chain_config(chain_id)?;

    // Resolve LendingPool address at runtime
    let pool_addr = rpc::get_lending_pool(cfg.lending_pool_addresses_provider, cfg.rpc_url)
        .await
        .unwrap_or_else(|_| cfg.lending_pool_proxy.to_string());

    // Get list of reserves: LendingPool.getReservesList() → selector 0xd1946dbc
    let reserves_list_hex = rpc::eth_call(cfg.rpc_url, &pool_addr, "0xd1946dbc")
        .await
        .context("Failed to call LendingPool.getReservesList()")?;

    let reserve_addresses = decode_address_array(&reserves_list_hex)?;

    if reserve_addresses.is_empty() {
        return Ok(json!({
            "ok": true,
            "chain": cfg.name,
            "chainId": chain_id,
            "reserves": [],
            "message": "No reserves found"
        }));
    }

    let mut reserves: Vec<Value> = Vec::new();

    for addr in &reserve_addresses {
        if let Some(filter) = asset_filter {
            if filter.starts_with("0x") && !addr.eq_ignore_ascii_case(filter) {
                continue;
            }
        }

        match get_reserve_data_v2(&pool_addr, addr, cfg.rpc_url).await {
            Ok(reserve_data) => {
                reserves.push(reserve_data);
            }
            Err(e) => {
                eprintln!("Warning: failed to fetch data for reserve {}: {}", addr, e);
            }
        }
    }

    Ok(json!({
        "ok": true,
        "chain": cfg.name,
        "chainId": chain_id,
        "lendingPool": pool_addr,
        "reserveCount": reserves.len(),
        "reserves": reserves
    }))
}

/// Fetch V2 reserve data from LendingPool.getReserveData(address).
/// Selector: 0x35ea6a75
///
/// V2 ReserveData slot layout (different from V3):
///   Slot 3: currentLiquidityRate (supply APY)
///   Slot 4: currentVariableBorrowRate
///   Slot 5: currentStableBorrowRate
///   Slot 7: aTokenAddress (last 20 bytes)
async fn get_reserve_data_v2(
    pool_addr: &str,
    asset_addr: &str,
    rpc_url: &str,
) -> anyhow::Result<Value> {
    // getReserveData(address asset) → selector 0x35ea6a75
    let addr_bytes = hex::decode(asset_addr.trim_start_matches("0x"))?;
    let mut data = hex::decode("35ea6a75")?;
    data.extend_from_slice(&[0u8; 12]);
    data.extend_from_slice(&addr_bytes);
    let data_hex = format!("0x{}", hex::encode(&data));

    let result = rpc::eth_call(rpc_url, pool_addr, &data_hex).await?;
    let raw = result.trim_start_matches("0x");

    if raw.len() < 64 * 6 {
        anyhow::bail!("LendingPool.getReserveData: short response ({} chars)", raw.len());
    }

    // V2: supply APY at slot 3, variable borrow at slot 4, stable at slot 5
    let supply_apy = decode_ray_to_apy_pct(raw, 3)?;
    let variable_borrow_apy = decode_ray_to_apy_pct(raw, 4)?;
    let stable_borrow_apy = decode_ray_to_apy_pct(raw, 5)?;

    // Extract aToken address from slot 7 (last 40 hex chars of the slot)
    let atoken_addr = if raw.len() >= 64 * 8 {
        let slot7 = &raw[7 * 64..8 * 64];
        format!("0x{}", &slot7[24..64])
    } else {
        String::new()
    };

    Ok(json!({
        "underlyingAsset": asset_addr,
        "aTokenAddress": atoken_addr,
        "supplyApy": format!("{:.4}%", supply_apy),
        "variableBorrowApy": format!("{:.4}%", variable_borrow_apy),
        "stableBorrowApy": format!("{:.4}%", stable_borrow_apy)
    }))
}

fn decode_ray_to_apy_pct(raw: &str, slot: usize) -> anyhow::Result<f64> {
    let start = slot * 64;
    let end = start + 64;
    if raw.len() < end {
        return Ok(0.0);
    }
    let slot_hex = &raw[start..end];
    let low = &slot_hex[32..64];
    let val = u128::from_str_radix(low, 16).unwrap_or(0);
    let pct = val as f64 / 1e27 * 100.0;
    Ok(pct)
}

/// Decode an ABI-encoded dynamic array of addresses.
fn decode_address_array(hex_result: &str) -> anyhow::Result<Vec<String>> {
    let raw = hex_result.trim_start_matches("0x");
    if raw.len() < 128 {
        return Ok(vec![]);
    }
    let len_hex = &raw[64..128];
    let len = usize::from_str_radix(len_hex.trim_start_matches('0'), 16).unwrap_or(0);
    if len == 0 {
        return Ok(vec![]);
    }

    let mut addresses = Vec::with_capacity(len);
    let data_start = 128;
    for i in 0..len {
        let slot_start = data_start + i * 64;
        let slot_end = slot_start + 64;
        if raw.len() < slot_end {
            break;
        }
        let addr_hex = &raw[slot_end - 40..slot_end];
        addresses.push(format!("0x{}", addr_hex));
    }
    Ok(addresses)
}
