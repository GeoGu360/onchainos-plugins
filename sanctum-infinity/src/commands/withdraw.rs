// withdraw — remove liquidity from Sanctum Infinity pool
use anyhow::Result;
use serde_json::json;

use crate::api;
use crate::config::{INF_PROGRAM_ID, LST_DECIMALS};
use crate::onchainos;

pub async fn execute(
    client: &reqwest::Client,
    lst: &str,      // LST to receive (symbol or mint)
    amount: f64,    // INF/LP tokens to burn (UI units)
    slippage: f64,  // percentage
    dry_run: bool,
) -> Result<()> {
    // dry_run guard — resolve wallet AFTER this
    if dry_run {
        let lst_mint = api::resolve_lst_mint(lst);
        let lp_amount_atomics = api::ui_to_atomics(amount, LST_DECIMALS);
        let quote_result = api::get_liquidity_remove_quote(client, lst_mint, lp_amount_atomics).await;
        let preview = match &quote_result {
            Ok(q) => {
                let out: u64 = q.lst_amount.parse().unwrap_or(0);
                json!({ "expected_lst_ui": api::atomics_to_ui(out, LST_DECIMALS) })
            }
            Err(_) => json!({ "note": "quote unavailable in dry-run" })
        };
        let output = json!({
            "ok": true,
            "dry_run": true,
            "data": {
                "txHash": "",
                "lst_mint": lst_mint,
                "lp_amount_ui": amount,
                "slippage_pct": slippage,
                "preview": preview
            }
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    let lst_mint = api::resolve_lst_mint(lst);
    let lp_amount_atomics = api::ui_to_atomics(amount, LST_DECIMALS);

    // Resolve wallet (after dry_run guard)
    let wallet = onchainos::resolve_wallet_solana()?;

    // Get liquidity remove quote
    let quote = api::get_liquidity_remove_quote(client, lst_mint, lp_amount_atomics).await?;
    let lst_amount: u64 = quote.lst_amount.parse().unwrap_or(0);
    let quoted_amount = api::apply_slippage(lst_amount, slippage);

    // Get serialized transaction
    // ⚠️ Solana blockhash expires in ~60s — call onchainos immediately
    let tx_b64 = api::execute_liquidity_remove(
        client,
        lst_mint,
        lp_amount_atomics,
        quoted_amount,
        &wallet,
    )
    .await?;

    // Submit via onchainos
    let result = onchainos::wallet_contract_call_solana(INF_PROGRAM_ID, &tx_b64, false).await?;
    let tx_hash = onchainos::extract_tx_hash(&result);

    let output = json!({
        "ok": true,
        "data": {
            "txHash": tx_hash,
            "lst_mint": lst_mint,
            "lp_burned_ui": amount,
            "lst_received_ui": api::atomics_to_ui(lst_amount, LST_DECIMALS),
            "slippage_pct": slippage
        }
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
