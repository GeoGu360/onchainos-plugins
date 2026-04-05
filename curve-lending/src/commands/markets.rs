use clap::Args;
use serde_json::json;
use crate::config::{LENDING_FACTORY, get_rpc};
use crate::rpc::*;

#[derive(Args)]
pub struct MarketsArgs {
    /// EVM chain ID (default: 1 = Ethereum mainnet)
    #[arg(long, default_value = "1")]
    pub chain: u64,

    /// Max number of markets to display (default: 20)
    #[arg(long, default_value = "20")]
    pub limit: u64,
}

pub async fn run(args: MarketsArgs) -> anyhow::Result<()> {
    let rpc = get_rpc(args.chain);

    // 1. Get total market count
    let count_raw = eth_call_raw(rpc, LENDING_FACTORY, "0xfd775c78")?;
    let count = decode_uint256(&count_raw) as u64;
    let limit = std::cmp::min(count, args.limit);

    let mut markets = Vec::new();

    for i in 0..limit {
        // name
        let name = eth_call_string(rpc, LENDING_FACTORY, &calldata_factory_names(i))
            .unwrap_or_else(|_| format!("market-{}", i));

        // controller address
        let controller = eth_call_address(rpc, LENDING_FACTORY, &calldata_factory_controllers(i))
            .unwrap_or_default();

        // vault address
        let vault = eth_call_address(rpc, LENDING_FACTORY, &calldata_factory_vaults(i))
            .unwrap_or_default();

        // collateral token
        let collateral_token = eth_call_address(
            rpc,
            LENDING_FACTORY,
            &calldata_factory_collateral_tokens(i),
        )
        .unwrap_or_default();

        // borrowed token (usually crvUSD)
        let borrow_token = eth_call_address(
            rpc,
            LENDING_FACTORY,
            &calldata_factory_borrowed_tokens(i),
        )
        .unwrap_or_default();

        // collateral symbol
        let collateral_symbol = if !collateral_token.is_empty() {
            eth_call_string(rpc, &collateral_token, calldata_symbol()).unwrap_or_default()
        } else {
            String::new()
        };

        // borrow symbol
        let borrow_symbol = if !borrow_token.is_empty() {
            eth_call_string(rpc, &borrow_token, calldata_symbol()).unwrap_or_else(|_| "crvUSD".to_string())
        } else {
            "crvUSD".to_string()
        };

        // vault totalAssets
        let total_assets_raw = if !vault.is_empty() {
            eth_call_uint256(rpc, &vault, calldata_total_assets()).unwrap_or(0)
        } else {
            0
        };
        let total_assets_display = total_assets_raw as f64 / 1e18;

        // n_loans from controller
        let n_loans = if !controller.is_empty() {
            eth_call_uint256(rpc, &controller, calldata_n_loans()).unwrap_or(0)
        } else {
            0
        };

        // total_debt from controller
        let total_debt_raw = if !controller.is_empty() {
            eth_call_uint256(rpc, &controller, calldata_total_debt()).unwrap_or(0)
        } else {
            0
        };
        let total_debt_display = total_debt_raw as f64 / 1e18;

        markets.push(json!({
            "index": i,
            "name": name,
            "controller": controller,
            "vault": vault,
            "collateral_token": collateral_token,
            "collateral_symbol": collateral_symbol,
            "borrow_token": borrow_token,
            "borrow_symbol": borrow_symbol,
            "total_supply_crvusd": format!("{:.2}", total_assets_display),
            "n_active_loans": n_loans,
            "total_debt_crvusd": format!("{:.2}", total_debt_display),
        }));
    }

    let output = json!({
        "ok": true,
        "chain": args.chain,
        "factory": LENDING_FACTORY,
        "total_market_count": count,
        "displayed": limit,
        "markets": markets,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
