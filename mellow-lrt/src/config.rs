/// Chain and contract configuration for Mellow LRT plugin.

pub struct ChainConfig {
    #[allow(dead_code)]
    pub chain_id: u64,
    pub name: &'static str,
    pub rpc_url: &'static str,
    pub eth_wrapper: &'static str,
    #[allow(dead_code)]
    pub wsteth: &'static str,
}

pub const CHAIN_ETHEREUM: ChainConfig = ChainConfig {
    chain_id: 1,
    name: "Ethereum",
    rpc_url: "https://ethereum.publicnode.com",
    // EthWrapper: wraps ETH/WETH/stETH -> wstETH, then deposits into vault
    eth_wrapper: "0x83F6c979ce7a52C7027F08119222825E5bd50351",
    wsteth: "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0",
};

pub fn get_chain_config(chain_id: u64) -> anyhow::Result<&'static ChainConfig> {
    match chain_id {
        1 => Ok(&CHAIN_ETHEREUM),
        _ => anyhow::bail!(
            "Unsupported chain ID: {}. Mellow LRT is deployed on Ethereum mainnet (chain 1).",
            chain_id
        ),
    }
}

/// Known Mellow LRT vault addresses on Ethereum mainnet.
/// Returns (vault_address, base_token_address, base_token_decimals, symbol)
/// Sorted by TVL (descending) as of 2026-04.
pub const KNOWN_VAULTS: &[(&str, &str, &str, u8)] = &[
    // (symbol, vault_address, base_token_address, base_token_decimals)
    ("steakLRT",  "0xBEEF69Ac7870777598A04B2bd4771c71212E6aBc", "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0", 18),
    ("Re7LRT",    "0x84631c0d0081FDe56DeB72F6DE77abBbF6A9f93a", "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0", 18),
    ("amphrETH",  "0x5fD13359Ba15A84B76f7F87568309040176167cd", "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0", 18),
    ("rstETH",    "0x7a4EffD87C2f3C55CA251080b1343b605f327E3a", "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0", 18),
    ("pzETH",     "0x8c9532a60E0E7C6BbD2B2c1303F63aCE1c3E9811", "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0", 18),
    ("DVstETH",   "0x5E362eb2c0706Bd1d134689eC75176018385430B", "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0", 18),
    ("cp0xLRT",   "0xB908c9FE885369643adB5FBA4407d52bD726c72d", "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0", 18),
    ("roETH",     "0x7b31F008c48EFb65da78eA0f255EE424af855249", "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0", 18),
    ("urLRT",     "0x4f3Cc6359364004b245ad5bE36E6ad4e805dC961", "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0", 18),
    ("ifsETH",    "0x49cd586dd9BA227Be9654C735A659a1dB08232a9", "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0", 18),
];

/// Resolve a vault by symbol (case-insensitive) or address.
/// Returns (vault_address, base_token_address, base_token_decimals).
pub fn resolve_vault_static(input: &str) -> Option<(String, String, u8)> {
    let input_lower = input.to_lowercase();
    for (symbol, vault_addr, base_addr, decimals) in KNOWN_VAULTS {
        if symbol.to_lowercase() == input_lower || vault_addr.to_lowercase() == input_lower {
            return Some((vault_addr.to_string(), base_addr.to_string(), *decimals));
        }
    }
    // Accept raw vault address
    if input.starts_with("0x") && input.len() == 42 {
        return Some((input.to_string(), String::new(), 18));
    }
    None
}

/// Mellow Protocol REST API base URL.
pub const MELLOW_API_BASE: &str = "https://api.mellow.finance/v1";

/// ETH pseudo-address used by EthWrapper for native ETH deposits.
/// This is the standard ETH placeholder address used in EthWrapper contract.
pub const ETH_ADDR: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
