/// Per-chain configuration for Aave V2.
///
/// Aave V2 is deployed on Ethereum mainnet only.
/// LendingPool address is resolved at runtime via LendingPoolAddressesProvider.getLendingPool().
///
/// Addresses verified against Aave V2 documentation and Etherscan:
///   - LendingPoolAddressesProvider: 0xB53C1a33016B2DC2fF3653530bfF1848a515c8c5
///   - LendingPool (proxy): 0x7d2768dE32b0b80b7a3454c06BdAc94A69DDc7A9
///   - ProtocolDataProvider: 0x057835Ad21a177dbdd3090bB1CAE03EaCF78Fc6d
#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub lending_pool_addresses_provider: &'static str,
    /// Static LendingPool proxy address — used as fallback if dynamic resolution fails.
    pub lending_pool_proxy: &'static str,
    pub rpc_url: &'static str,
    pub name: &'static str,
}

pub static CHAINS: &[ChainConfig] = &[
    ChainConfig {
        chain_id: 1,
        lending_pool_addresses_provider: "0xB53C1a33016B2DC2fF3653530bfF1848a515c8c5",
        lending_pool_proxy: "0x7d2768dE32b0b80b7a3454c06BdAc94A69DDc7A9",
        rpc_url: "https://ethereum.publicnode.com",
        name: "Ethereum Mainnet",
    },
];

pub fn get_chain_config(chain_id: u64) -> anyhow::Result<&'static ChainConfig> {
    CHAINS
        .iter()
        .find(|c| c.chain_id == chain_id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Unsupported chain ID: {}. Aave V2 is only deployed on: {}",
                chain_id,
                CHAINS
                    .iter()
                    .map(|c| format!("{} ({})", c.name, c.chain_id))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
}

/// Interest rate mode constants for Aave V2
/// V2 supports both stable and variable rate modes
#[allow(dead_code)]
pub const INTEREST_RATE_MODE_STABLE: u128 = 1;
pub const INTEREST_RATE_MODE_VARIABLE: u128 = 2;

/// Aave referral code (0 = no referral)
pub const REFERRAL_CODE: u16 = 0;

/// Health factor thresholds (scaled 1e18 on-chain, these are human-readable)
pub const HF_WARN_THRESHOLD: f64 = 1.1;
#[allow(dead_code)]
pub const HF_DANGER_THRESHOLD: f64 = 1.05;
