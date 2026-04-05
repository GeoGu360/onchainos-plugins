// swap — LST → LST swap via Sanctum Infinity pool
use anyhow::Result;
use serde_json::json;

use crate::api;
use crate::config::{INF_PROGRAM_ID, LST_DECIMALS};
use crate::onchainos;

pub async fn execute(
    client: &reqwest::Client,
    from: &str,    // symbol or mint
    to: &str,      // symbol or mint
    amount: f64,   // UI units
    slippage: f64, // percentage (e.g. 0.5 = 0.5%)
    dry_run: bool,
) -> Result<()> {
    // dry_run guard — resolve wallet AFTER this
    if dry_run {
        let from_mint = api::resolve_lst_mint(from);
        let to_mint = api::resolve_lst_mint(to);
        let amount_atomics = api::ui_to_atomics(amount, LST_DECIMALS);
        // Attempt to get quote for preview (non-fatal if router is down)
        let quote_result = api::get_swap_quote(client, from_mint, to_mint, amount_atomics, "ExactIn").await;
        let preview = match &quote_result {
            Ok(q) => {
                let out: u64 = q.out_amount.parse().unwrap_or(0);
                json!({ "out_amount_ui": api::atomics_to_ui(out, LST_DECIMALS), "swap_src": q.swap_src })
            }
            Err(_) => json!({ "note": "quote unavailable in dry-run" })
        };
        let output = json!({
            "ok": true,
            "dry_run": true,
            "data": {
                "txHash": "",
                "from_mint": from_mint,
                "to_mint": to_mint,
                "amount_ui": amount,
                "slippage_pct": slippage,
                "preview": preview
            }
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    let from_mint = api::resolve_lst_mint(from);
    let to_mint = api::resolve_lst_mint(to);
    let amount_atomics = api::ui_to_atomics(amount, LST_DECIMALS);

    // Resolve wallet (after dry_run guard)
    let wallet = onchainos::resolve_wallet_solana()?;

    // Get quote first to calculate min out amount
    let quote = api::get_swap_quote(client, from_mint, to_mint, amount_atomics, "ExactIn").await?;
    let out_amount: u64 = quote.out_amount.parse().unwrap_or(0);
    let quoted_amount = api::apply_slippage(out_amount, slippage);

    // Get serialized transaction
    // ⚠️ Must be called immediately after getting tx — Solana blockhash expires in ~60s
    let tx_b64 = api::execute_swap(
        client,
        from_mint,
        to_mint,
        amount_atomics,
        quoted_amount,
        &wallet,
        "ExactIn",
    )
    .await?;

    // Submit via onchainos (base64 → base58 conversion happens inside)
    let result = onchainos::wallet_contract_call_solana(INF_PROGRAM_ID, &tx_b64, false).await?;
    let tx_hash = onchainos::extract_tx_hash(&result);

    let output = json!({
        "ok": true,
        "data": {
            "txHash": tx_hash,
            "from_mint": from_mint,
            "to_mint": to_mint,
            "amount_ui": amount,
            "out_amount_ui": api::atomics_to_ui(out_amount, LST_DECIMALS),
            "slippage_pct": slippage,
            "swap_src": quote.swap_src
        }
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
