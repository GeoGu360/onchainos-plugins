// pools — query Sanctum Infinity pool stats
use anyhow::Result;
use serde_json::json;

use crate::api;
use crate::config::LST_DECIMALS;

pub async fn execute(client: &reqwest::Client) -> Result<()> {
    // Fetch SOL value, APY, TVL and allocation in parallel
    let (sol_val_res, apy_res, tvl_res, alloc_res) = tokio::join!(
        api::get_sol_value(client, "INF"),
        api::get_apy(client, &["INF"]),
        api::get_tvl(client, "INF"),
        api::get_infinity_allocation(client),
    );

    let sol_value_str = sol_val_res?;
    let apy_resp = apy_res?;
    let tvl_str = tvl_res.unwrap_or_else(|_| "0".to_string());

    // INF NAV in lamports per 1 INF (atomic)
    let nav_lamports: u64 = sol_value_str.parse().unwrap_or(0);
    let nav_sol = api::atomics_to_ui(nav_lamports, LST_DECIMALS);
    let inf_apy = apy_resp.apys.get("INF").copied().unwrap_or(0.0);

    // TVL in lamports
    let tvl_lamports: u64 = tvl_str.parse().unwrap_or(0);
    let tvl_sol = api::atomics_to_ui(tvl_lamports, LST_DECIMALS);

    // Pool allocation (may be unavailable — NO_DATA_AVAILABLE)
    let (lst_allocations, alloc_note) = match alloc_res {
        Ok(alloc) if !alloc.infinity.is_empty() => {
            let mut allocations: Vec<serde_json::Value> = alloc
                .infinity
                .iter()
                .map(|(mint, info)| {
                    let amt_atomics: u64 = info.amt.parse().unwrap_or(0);
                    let sol_val_atomics: u64 = info.sol_value.parse().unwrap_or(0);
                    json!({
                        "mint": mint,
                        "amount_ui": api::atomics_to_ui(amt_atomics, LST_DECIMALS),
                        "sol_value_ui": api::atomics_to_ui(sol_val_atomics, LST_DECIMALS),
                        "pool_share_pct": (info.share * 100.0)
                    })
                })
                .collect();
            allocations.sort_by(|a, b| {
                let sa = a["pool_share_pct"].as_f64().unwrap_or(0.0);
                let sb = b["pool_share_pct"].as_f64().unwrap_or(0.0);
                sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
            });
            (allocations, None)
        }
        _ => (vec![], Some("Allocation data temporarily unavailable")),
    };

    let mut data = json!({
        "pool": "Sanctum Infinity (INF)",
        "inf_mint": crate::config::INF_MINT,
        "inf_program_id": crate::config::INF_PROGRAM_ID,
        "nav_sol_per_inf": nav_sol,
        "apy_pct": inf_apy * 100.0,
        "total_tvl_sol": tvl_sol,
    });

    if lst_allocations.is_empty() {
        data["allocation_note"] = json!(alloc_note.unwrap_or("No allocations"));
    } else {
        data["lst_count"] = json!(lst_allocations.len());
        data["top_allocations"] = json!(&lst_allocations[..lst_allocations.len().min(10)]);
    }

    let output = json!({ "ok": true, "data": data });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
