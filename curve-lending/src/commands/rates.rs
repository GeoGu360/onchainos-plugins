use clap::Args;
use serde_json::json;
use crate::config::{LENDING_FACTORY, get_rpc};
use crate::rpc::*;

#[derive(Args)]
pub struct RatesArgs {
    /// EVM chain ID (default: 1)
    #[arg(long, default_value = "1")]
    pub chain: u64,

    /// Market name (e.g. "WETH-long") or index (e.g. "1")
    #[arg(long)]
    pub market: Option<String>,
}

/// Compute APY from per-second rate (1e18 scaled)
fn per_second_to_apy(rate_per_sec: u128) -> f64 {
    let r = rate_per_sec as f64 / 1e18;
    ((1.0 + r).powf(365.0 * 24.0 * 3600.0) - 1.0) * 100.0
}

pub async fn run(args: RatesArgs) -> anyhow::Result<()> {
    let rpc = get_rpc(args.chain);

    // Get total market count
    let count_raw = eth_call_raw(rpc, LENDING_FACTORY, "0xfd775c78")?;
    let count = decode_uint256(&count_raw) as u64;

    // Determine which markets to query
    let indices: Vec<u64> = if let Some(ref market_ref) = args.market {
        // Try parse as numeric index
        if let Ok(idx) = market_ref.parse::<u64>() {
            vec![idx]
        } else {
            // Search by name substring
            let mut found = Vec::new();
            for i in 0..count {
                let name = eth_call_string(rpc, LENDING_FACTORY, &calldata_factory_names(i))
                    .unwrap_or_default();
                if name.to_lowercase().contains(&market_ref.to_lowercase()) {
                    found.push(i);
                }
            }
            if found.is_empty() {
                anyhow::bail!("No market found matching '{}'", market_ref);
            }
            found
        }
    } else {
        // All markets (capped at 10 for rates command)
        (0..std::cmp::min(count, 10)).collect()
    };

    let mut results = Vec::new();

    for i in indices {
        let name = eth_call_string(rpc, LENDING_FACTORY, &calldata_factory_names(i))
            .unwrap_or_else(|_| format!("market-{}", i));
        let controller = eth_call_address(rpc, LENDING_FACTORY, &calldata_factory_controllers(i))
            .unwrap_or_default();
        let vault = eth_call_address(rpc, LENDING_FACTORY, &calldata_factory_vaults(i))
            .unwrap_or_default();
        let monetary_policy = eth_call_address(
            rpc,
            LENDING_FACTORY,
            &calldata_factory_monetary_policies(i),
        )
        .unwrap_or_default();

        // Vault total assets (liquidity)
        let total_assets = if !vault.is_empty() {
            eth_call_uint256(rpc, &vault, calldata_total_assets()).unwrap_or(0)
        } else {
            0
        };

        // Controller total_debt
        let total_debt = if !controller.is_empty() {
            eth_call_uint256(rpc, &controller, calldata_total_debt()).unwrap_or(0)
        } else {
            0
        };

        // Utilization
        let utilization = if total_assets > 0 {
            (total_debt as f64) / (total_assets as f64) * 100.0
        } else {
            0.0
        };

        // Borrow rate: monetary_policy.rate(controller) → per-second rate
        let borrow_rate_per_sec = if !monetary_policy.is_empty() && !controller.is_empty() {
            eth_call_uint256(rpc, &monetary_policy, &calldata_mp_rate(&controller)).unwrap_or(0)
        } else {
            0
        };
        let borrow_apy = per_second_to_apy(borrow_rate_per_sec);

        // Lend APY: vault.lend_apy() returns 0 on most Curve Lending markets.
        // Compute from borrow_apy * utilization (standard isolated vault formula).
        let lend_apy_pct = if utilization > 0.0 {
            borrow_apy * utilization / 100.0
        } else {
            0.0
        };

        // Min/max rates
        let min_rate_raw = if !monetary_policy.is_empty() {
            eth_call_uint256(rpc, &monetary_policy, "0x5d786401").unwrap_or(0) // min_rate() ✅
        } else {
            0
        };
        let max_rate_raw = if !monetary_policy.is_empty() {
            eth_call_uint256(rpc, &monetary_policy, "0x536e4ec4").unwrap_or(0) // max_rate() ✅
        } else {
            0
        };

        results.push(json!({
            "index": i,
            "name": name,
            "controller": controller,
            "vault": vault,
            "borrow_apy_pct": format!("{:.4}", borrow_apy),
            "lend_apy_pct": format!("{:.4}", lend_apy_pct),
            "borrow_rate_per_second": borrow_rate_per_sec.to_string(),
            "min_borrow_apy_pct": format!("{:.4}", per_second_to_apy(min_rate_raw)),
            "max_borrow_apy_pct": format!("{:.4}", per_second_to_apy(max_rate_raw)),
            "total_supply_crvusd": format!("{:.2}", total_assets as f64 / 1e18),
            "total_debt_crvusd": format!("{:.2}", total_debt as f64 / 1e18),
            "utilization_pct": format!("{:.2}", utilization),
        }));
    }

    let output = json!({
        "ok": true,
        "chain": args.chain,
        "rates": results,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
