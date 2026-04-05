// src/config.rs — INIT Capital contract addresses and pool metadata (Blast chain 81457)
//
// Note: INIT Capital primary chain is Mantle (5000) but onchainos does not support Mantle.
// We use the Blast (81457) deployment which onchainos supports.

/// A lending pool entry in INIT Capital
#[derive(Debug, Clone)]
pub struct Pool {
    pub symbol: &'static str,       // user-facing token symbol, e.g. "WETH", "USDB"
    pub pool: &'static str,         // lending pool proxy address (inToken ERC-20)
    pub underlying: &'static str,   // underlying token address
    pub underlying_decimals: u8,
}

// ── Blast Mainnet (chain 81457) ─────────────────────────────────────────────

pub const INIT_CORE: &str = "0xa7d36f2106b5a5D528a7e2e7a3f436d703113A10";
pub const POS_MANAGER: &str = "0xA0e172f8BdC18854903959b8f7f73F0D332633fe";
pub const MONEY_MARKET_HOOK: &str = "0xC02819a157320Ba2859951A1dfc1a5E76c424dD4";
pub const INIT_LENS: &str = "0x56Fba2cC045C02d7adAE5A9dfDce795900b2860E";

pub const POOLS: &[Pool] = &[
    Pool {
        symbol: "WETH",
        pool: "0xD20989EB39348994AA99F686bb4554090d0C09F3",
        underlying: "0x4300000000000000000000000000000000000004",
        underlying_decimals: 18,
    },
    Pool {
        symbol: "USDB",
        pool: "0xc5EaC92633aF47c0023Afa0116500ab86FAB430F",
        underlying: "0x4300000000000000000000000000000000000003",
        underlying_decimals: 18,
    },
];

/// Blast RPC endpoint
pub const RPC_URL: &str = "https://rpc.blast.io";
/// Fallback RPC
pub const RPC_URL_FALLBACK: &str = "https://blast-rpc.publicnode.com";

/// Scale a human-readable amount to raw integer units
pub fn to_raw(amount: f64, decimals: u8) -> u128 {
    let factor = 10f64.powi(decimals as i32);
    (amount * factor).round() as u128
}

/// Find a pool by asset symbol (case-insensitive)
pub fn find_pool(symbol: &str) -> Option<&'static Pool> {
    let sym = symbol.to_uppercase();
    POOLS.iter().find(|p| p.symbol.to_uppercase() == sym)
}
