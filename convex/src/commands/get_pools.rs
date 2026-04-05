use anyhow::Result;
use clap::Args;
use crate::api;

#[derive(Args, Debug)]
pub struct GetPoolsArgs {
    /// Maximum number of pools to return (default: 10)
    #[arg(long, default_value = "10")]
    pub limit: usize,

    /// Registry to query: main, factory, or all (default: all)
    #[arg(long, default_value = "all")]
    pub registry: String,
}

pub async fn run(args: GetPoolsArgs) -> Result<()> {
    let registries: Vec<&str> = match args.registry.as_str() {
        "main" => vec!["main"],
        "factory" => vec!["factory"],
        _ => vec!["main", "factory"],
    };

    let mut all_pools: Vec<api::PoolSummary> = Vec::new();

    for reg in &registries {
        match api::fetch_curve_pools(reg).await {
            Ok(pools) => {
                for pool in &pools {
                    all_pools.push(api::to_summary(pool, reg));
                }
            }
            Err(e) => {
                eprintln!("Warning: failed to fetch {} pools: {}", reg, e);
            }
        }
    }

    // Sort by TVL (parse $ prefix)
    all_pools.sort_by(|a, b| {
        let a_val: f64 = a.tvl_usd.trim_start_matches('$').replace(',', "").parse().unwrap_or(0.0);
        let b_val: f64 = b.tvl_usd.trim_start_matches('$').replace(',', "").parse().unwrap_or(0.0);
        b_val.partial_cmp(&a_val).unwrap_or(std::cmp::Ordering::Equal)
    });

    let pools_to_show: Vec<&api::PoolSummary> = all_pools.iter().take(args.limit).collect();

    let output = serde_json::json!({
        "ok": true,
        "data": {
            "total_found": all_pools.len(),
            "shown": pools_to_show.len(),
            "pools": pools_to_show
        }
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
