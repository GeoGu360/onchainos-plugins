// src/config.rs -- Benqi Lending contract addresses and market metadata
// Benqi is a Compound V2 fork on Avalanche C-Chain (chain ID 43114)
// Key difference from Compound V2: uses per-timestamp rates (not per-block)
// Reward types: 0 = QI token, 1 = AVAX

/// Avalanche C-Chain ID
pub const CHAIN_ID: u64 = 43114;

/// Benqi Comptroller
pub const COMPTROLLER: &str = "0x486Af39519B4Dc9a7fCcd318217352830E8AD9b4";

/// QI governance token
pub const QI_TOKEN: &str = "0x8729438EB15e2C8B576fCc6AeCdA6A148776C0F5";

/// Avalanche public RPC
pub const RPC_URL: &str = "https://avalanche-c-chain-rpc.publicnode.com";

/// Seconds per year (used to annualize per-timestamp rates)
pub const SECONDS_PER_YEAR: u128 = 31_536_000;

/// Known Benqi qiToken market info
#[derive(Debug, Clone)]
pub struct Market {
    pub symbol: &'static str,       // e.g. "AVAX"
    pub qi_token: &'static str,     // qiToken contract address
    pub underlying: Option<&'static str>, // None for qiAVAX (native AVAX)
    pub underlying_decimals: u8,
    pub qi_token_decimals: u8,
    pub is_native: bool,            // true for AVAX
}

pub const MARKETS: &[Market] = &[
    Market {
        symbol: "AVAX",
        qi_token: "0x5C0401e81Bc07Ca70fAD469b451682c0d747Ef1c",
        underlying: None,
        underlying_decimals: 18,
        qi_token_decimals: 8,
        is_native: true,
    },
    Market {
        symbol: "USDC",
        qi_token: "0xBEb5d47A3f720Ec0a390d04b4d41ED7d9688bC7F",
        underlying: Some("0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E"),
        underlying_decimals: 6,
        qi_token_decimals: 8,
        is_native: false,
    },
    Market {
        symbol: "USDT",
        qi_token: "0xc9e5999b8e75C3fEB117F6f73E664b9f3C8ca65C",
        underlying: Some("0x9702230A8Ea53601f5cD2dc00fDBc13d4dF4A8c7"),
        underlying_decimals: 6,
        qi_token_decimals: 8,
        is_native: false,
    },
    Market {
        symbol: "ETH",
        qi_token: "0x334AD834Cd4481BB02d09615E7c11a00579A7909",
        underlying: Some("0x49D5c2BdFfac6CE2BFdB6640F4F80f226bc10bAB"),
        underlying_decimals: 18,
        qi_token_decimals: 8,
        is_native: false,
    },
    Market {
        symbol: "BTC",
        qi_token: "0xe194c4c5aC32a3C9ffDb358d9Bfd523a0B6d1568",
        underlying: Some("0x50b7545627a5162F82A992c33b87aDc75187B218"),
        underlying_decimals: 8,
        qi_token_decimals: 8,
        is_native: false,
    },
    Market {
        symbol: "LINK",
        qi_token: "0x4e9f683A27a6BdAD3FC2764003759277e93696e6",
        underlying: Some("0x5947BB275c521040051D82396192181b413227A3"),
        underlying_decimals: 18,
        qi_token_decimals: 8,
        is_native: false,
    },
    Market {
        symbol: "DAI",
        qi_token: "0x835866d37AFB8CB8F8334dCCdaf66cf01832Ff5D",
        underlying: Some("0xd586E7F844cEa2F87f50152665BCbc2C279D8d70"),
        underlying_decimals: 18,
        qi_token_decimals: 8,
        is_native: false,
    },
    Market {
        symbol: "QI",
        qi_token: "0x35Bd6aedA81a7E5FC7A7832490e71F757b0cD9Ce",
        underlying: Some("0x8729438EB15e2C8B576fCc6AeCdA6A148776C0F5"),
        underlying_decimals: 18,
        qi_token_decimals: 8,
        is_native: false,
    },
];

pub fn find_market(symbol: &str) -> Option<&'static Market> {
    let sym = symbol.to_uppercase();
    MARKETS.iter().find(|m| m.symbol == sym.as_str())
}

/// Scale a human-readable amount to raw integer units
pub fn to_raw(amount: f64, decimals: u8) -> u128 {
    let factor = 10f64.powi(decimals as i32);
    (amount * factor).round() as u128
}
