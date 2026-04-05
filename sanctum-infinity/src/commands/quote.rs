// quote — get swap quote for LST → LST via Sanctum Infinity
use anyhow::Result;
use serde_json::json;

use crate::api;
use crate::config::LST_DECIMALS;

pub async fn execute(
    client: &reqwest::Client,
    from: &str,  // symbol or mint address
    to: &str,    // symbol or mint address
    amount: f64, // UI units
    slippage: f64,
) -> Result<()> {
    let from_mint = api::resolve_lst_mint(from);
    let to_mint = api::resolve_lst_mint(to);
    let amount_atomics = api::ui_to_atomics(amount, LST_DECIMALS);

    let quote = api::get_swap_quote(client, from_mint, to_mint, amount_atomics, "ExactIn").await?;

    let in_amount: u64 = quote.in_amount.parse().unwrap_or(0);
    let out_amount: u64 = quote.out_amount.parse().unwrap_or(0);
    let min_out = api::apply_slippage(out_amount, slippage);

    let output = json!({
        "ok": true,
        "data": {
            "from_mint": from_mint,
            "to_mint": to_mint,
            "in_amount_ui": api::atomics_to_ui(in_amount, LST_DECIMALS),
            "out_amount_ui": api::atomics_to_ui(out_amount, LST_DECIMALS),
            "min_out_ui": api::atomics_to_ui(min_out, LST_DECIMALS),
            "slippage_pct": slippage,
            "swap_src": quote.swap_src,
            "fees": quote.fees,
            "rate": if in_amount > 0 { out_amount as f64 / in_amount as f64 } else { 0.0 }
        }
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
