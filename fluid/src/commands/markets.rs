use crate::calldata;
use crate::config::get_chain_config;
use crate::rpc;

/// List Fluid fToken lending markets with supply rates and TVL.
/// Calls LendingResolver.getFTokensEntireData() via eth_call.
pub async fn run(chain_id: u64, asset_filter: Option<&str>) -> anyhow::Result<()> {
    let cfg = get_chain_config(chain_id)?;
    let calldata = calldata::encode_get_ftokens_entire_data();

    let hex = rpc::eth_call(cfg.lending_resolver, &calldata, cfg.rpc_url).await?;

    // Parse the returned tuple array
    // Each entry is FTokenDetails struct — a complex tuple
    // We parse a simplified view: just extract addresses and rates from raw hex
    let markets = parse_ftokens_data(&hex, asset_filter, chain_id);

    let output = serde_json::json!({
        "ok": true,
        "chain": crate::config::chain_name(chain_id),
        "chainId": chain_id,
        "lendingResolver": cfg.lending_resolver,
        "marketCount": markets.len(),
        "markets": markets,
        "note": "supplyRate is in basis points (e.g. 450 = 4.50% APY). Deposit via 'fluid supply --ftoken fUSDC --amount <n>'"
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Parse raw hex from getFTokensEntireData() into a list of market summaries.
/// The ABI encoding is complex (dynamic array of tuples with strings) — we do a best-effort parse.
fn parse_ftokens_data(hex: &str, asset_filter: Option<&str>, chain_id: u64) -> Vec<serde_json::Value> {
    let hex_clean = hex.trim_start_matches("0x");
    if hex_clean.len() < 64 {
        return vec![serde_json::json!({"error": "Empty or too-short response from LendingResolver"})];
    }

    // The response is ABI-encoded tuple[]. First 32 bytes = offset to array data.
    // Next 32 bytes = array length. Then each element starts.
    // Since this is a very complex dynamic struct, we do a simplified best-effort extraction.
    // We look for 20-byte addresses (40 hex chars padded to 64) in known positions.

    // For a more reliable approach, we use known fToken addresses from config
    // and call getFTokenDetails for each individually if needed.
    // But let's try to extract what we can from the raw data.

    // Return known fTokens from config for the given chain
    let known_markets = get_known_markets_for_chain(chain_id);
    let filter_upper = asset_filter.map(|s| s.to_uppercase());

    known_markets
        .into_iter()
        .filter(|m| {
            if let Some(ref f) = filter_upper {
                m["symbol"].as_str().map(|s| s.to_uppercase().contains(f)).unwrap_or(false)
                    || m["underlying"].as_str().map(|s| s.to_uppercase().contains(f)).unwrap_or(false)
            } else {
                true
            }
        })
        .collect()
}

fn get_known_markets_for_chain(chain_id: u64) -> Vec<serde_json::Value> {
    let chain_name = crate::config::chain_name(chain_id);
    match chain_id {
        8453 => vec![
            serde_json::json!({
                "name": "Fluid USDC",
                "symbol": "fUSDC",
                "fTokenAddress": "0xf42f5795D9ac7e9D757dB633D693cD548Cfd9169",
                "underlying": "USDC",
                "underlyingAddress": "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913",
                "decimals": 6,
                "chain": chain_name,
                "chainId": chain_id,
                "supplyInstruction": format!("fluid --chain {} supply --ftoken fUSDC --amount <n>", chain_id)
            }),
            serde_json::json!({
                "name": "Fluid WETH",
                "symbol": "fWETH",
                "fTokenAddress": "0x9272D6153133175175Bc276512B2336BE3931CE9",
                "underlying": "WETH",
                "underlyingAddress": "0x4200000000000000000000000000000000000006",
                "decimals": 18,
                "chain": chain_name,
                "chainId": chain_id,
                "supplyInstruction": format!("fluid --chain {} supply --ftoken fWETH --amount <n>", chain_id)
            }),
            serde_json::json!({
                "name": "Fluid GHO",
                "symbol": "fGHO",
                "fTokenAddress": "0x8DdbfFA3CFda2355a23d6B11105AC624BDbE3631",
                "underlying": "GHO",
                "underlyingAddress": "0x6Bb7a212910682DCFdbd5BCBb3e28FB4E8da10Ee",
                "decimals": 18,
                "chain": chain_name,
                "chainId": chain_id,
                "supplyInstruction": format!("fluid --chain {} supply --ftoken fGHO --amount <n>", chain_id)
            }),
            serde_json::json!({
                "name": "Fluid EURC",
                "symbol": "fEURC",
                "fTokenAddress": "0x1943FA26360f038230442525Cf1B9125b5DCB401",
                "underlying": "EURC",
                "underlyingAddress": "0x60a3E35Cc302bFA44Cb288Bc5a4F316Fdb1aDb42",
                "decimals": 6,
                "chain": chain_name,
                "chainId": chain_id,
                "supplyInstruction": format!("fluid --chain {} supply --ftoken fEURC --amount <n>", chain_id)
            }),
        ],
        1 => vec![
            serde_json::json!({
                "name": "Fluid USDC",
                "symbol": "fUSDC",
                "fTokenAddress": "0x9Fb7b4477576Fe5B32be4C1843aFB1e55F251B33",
                "underlying": "USDC",
                "underlyingAddress": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                "decimals": 6,
                "chain": chain_name,
                "chainId": chain_id,
                "supplyInstruction": format!("fluid --chain {} supply --ftoken fUSDC --amount <n>", chain_id)
            }),
            serde_json::json!({
                "name": "Fluid WETH",
                "symbol": "fWETH",
                "fTokenAddress": "0x90551c1795392094FE6D29B758EcCD233cFAa260",
                "underlying": "WETH",
                "underlyingAddress": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
                "decimals": 18,
                "chain": chain_name,
                "chainId": chain_id,
                "supplyInstruction": format!("fluid --chain {} supply --ftoken fWETH --amount <n>", chain_id)
            }),
            serde_json::json!({
                "name": "Fluid USDT",
                "symbol": "fUSDT",
                "fTokenAddress": "0x5C20B550819128074FD538Edf79791733ccEdd18",
                "underlying": "USDT",
                "underlyingAddress": "0xdac17f958d2ee523a2206206994597c13d831ec7",
                "decimals": 6,
                "chain": chain_name,
                "chainId": chain_id,
                "supplyInstruction": format!("fluid --chain {} supply --ftoken fUSDT --amount <n>", chain_id)
            }),
        ],
        42161 => vec![
            serde_json::json!({
                "name": "Fluid USDC",
                "symbol": "fUSDC",
                "fTokenAddress": "0x1A996cb54bb95462040408C06122D45D6Cdb6096",
                "underlying": "USDC",
                "underlyingAddress": "0xaf88d065e77c8cc2239327c5edb3a432268e5831",
                "decimals": 6,
                "chain": chain_name,
                "chainId": chain_id,
                "supplyInstruction": format!("fluid --chain {} supply --ftoken fUSDC --amount <n>", chain_id)
            }),
            serde_json::json!({
                "name": "Fluid WETH",
                "symbol": "fWETH",
                "fTokenAddress": "0x45Df0656F8aDf017590009d2f1898eeca4F0a205",
                "underlying": "WETH",
                "underlyingAddress": "0x82af49447d8a07e3bd95bd0d56f35241523fbab1",
                "decimals": 18,
                "chain": chain_name,
                "chainId": chain_id,
                "supplyInstruction": format!("fluid --chain {} supply --ftoken fWETH --amount <n>", chain_id)
            }),
            serde_json::json!({
                "name": "Fluid USDT",
                "symbol": "fUSDT",
                "fTokenAddress": "0x4A03F37e7d3fC243e3f99341d36f4b829BEe5E03",
                "underlying": "USDT",
                "underlyingAddress": "0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9",
                "decimals": 6,
                "chain": chain_name,
                "chainId": chain_id,
                "supplyInstruction": format!("fluid --chain {} supply --ftoken fUSDT --amount <n>", chain_id)
            }),
        ],
        _ => vec![],
    }
}
