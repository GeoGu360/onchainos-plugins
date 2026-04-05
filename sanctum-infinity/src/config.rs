// Sanctum Infinity — configuration constants

/// Solana mainnet chain ID
pub const SOLANA_CHAIN_ID: u64 = 501;

/// Sanctum Infinity token mint
pub const INF_MINT: &str = "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm";

/// Sanctum Infinity pool program ID (SPool)
pub const INF_PROGRAM_ID: &str = "5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx";

/// wSOL mint (native SOL)
pub const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";

/// JitoSOL mint
pub const JITO_SOL_MINT: &str = "J1toso1uCk3RLmjorhTtrVBVzHQDSsvVQ6n8CGBbBTkp";

/// mSOL mint
pub const MSOL_MINT: &str = "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So";

/// Sanctum Extra API base URL — pool stats, APY, sol-value
pub const EXTRA_API_BASE: &str = "https://extra-api.sanctum.so";

/// Sanctum S Router API base URL — swap quotes, swap/liquidity transactions
pub const ROUTER_API_BASE: &str = "https://sanctum-s-api.fly.dev";

/// Solana token decimals (all Sanctum-supported LSTs use 9 decimals)
pub const LST_DECIMALS: u32 = 9;
