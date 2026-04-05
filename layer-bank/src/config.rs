// src/config.rs — LayerBank contract addresses and market metadata (Scroll chain 534352)

/// A LayerBank lToken market entry
#[derive(Debug, Clone)]
pub struct Market {
    pub symbol: &'static str,         // user-facing symbol, e.g. "ETH", "USDC"
    pub ltoken: &'static str,         // lToken contract address
    pub underlying: Option<&'static str>, // None = native ETH
    pub underlying_decimals: u8,
    pub is_eth: bool,
}

// ── Scroll Mainnet (chain 534352) ────────────────────────────────────────────

pub const CORE: &str = "0xEC53c830f4444a8A56455c6836b5D2aA794289Aa";
pub const PRICE_CALCULATOR: &str = "0xe3168c8D1Bcf6aaF5E090F61be619c060F3aD508";

pub const MARKETS: &[Market] = &[
    Market {
        symbol: "ETH",
        ltoken: "0x274C3795dadfEbf562932992bF241ae087e0a98C",
        underlying: None, // native ETH (address(0) in contract)
        underlying_decimals: 18,
        is_eth: true,
    },
    Market {
        symbol: "USDC",
        ltoken: "0x0D8F8e271DD3f2fC58e5716d3Ff7041dBe3F0688",
        underlying: Some("0x06efdbff2a14a7c8e15944d1f4a48f9f95f663a4"),
        underlying_decimals: 6,
        is_eth: false,
    },
    Market {
        symbol: "USDT",
        ltoken: "0xE0Cee49cC3C9d047C0B175943ab6FCC3c4F40fB0",
        underlying: Some("0xf55BEC9cafDbE8730f096Aa55dad6D22d44099Df"),
        underlying_decimals: 6,
        is_eth: false,
    },
    Market {
        symbol: "wstETH",
        ltoken: "0xB6966083c7b68175B4BF77511608AEe9A80d2Ca4",
        underlying: Some("0xf610A9dfB7C89644979b4A0f27063E9e7d7Cda32"),
        underlying_decimals: 18,
        is_eth: false,
    },
    Market {
        symbol: "WBTC",
        ltoken: "0xc40D6957B8110eC55f0F1A20d7D3430e1d8Aa4cf",
        underlying: Some("0x3C1BCa5a656e69edCD0D4E36BEbb3FcDAcA60Cf1"),
        underlying_decimals: 8,
        is_eth: false,
    },
];

/// Scroll Mainnet RPC
pub const RPC_URL: &str = "https://rpc.scroll.io";

/// Scale a human-readable amount to raw integer units
pub fn to_raw(amount: f64, decimals: u8) -> u128 {
    let factor = 10f64.powi(decimals as i32);
    (amount * factor).round() as u128
}

/// Find a market by asset symbol (case-insensitive)
pub fn find_market(symbol: &str) -> Option<&'static Market> {
    let sym = symbol.to_uppercase();
    MARKETS.iter().find(|m| m.symbol.to_uppercase() == sym)
}
