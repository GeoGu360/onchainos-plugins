use anyhow::Result;
use crate::api;

pub async fn run() -> Result<()> {
    let tokens = api::get_token_info().await
        .map_err(|e| anyhow::anyhow!("Failed to fetch token info from Allbridge API: {}", e))?;

    // Only show EVM + Solana chains (supported by onchainos)
    let supported_chains = ["ETH", "BSC", "POL", "AVA", "SOL", "FTM", "CELO"];

    let mut output = serde_json::json!({
        "ok": true,
        "data": {
            "chains": {}
        }
    });

    let chains_obj = output["data"]["chains"].as_object_mut().unwrap();

    for chain_id in &supported_chains {
        if let Some(token_list) = tokens.get(*chain_id) {
            let tokens_arr: Vec<serde_json::Value> = token_list
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "symbol": t.symbol,
                        "tokenAddress": t.address,
                        "precision": t.precision,
                        "minFee": t.min_fee,
                        "isBase": t.is_base,
                        "isWrapped": t.is_wrapped,
                        "tokenSource": t.token_source
                    })
                })
                .collect();
            chains_obj.insert(chain_id.to_string(), serde_json::json!(tokens_arr));
        }
    }

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
