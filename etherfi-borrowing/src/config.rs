/// Chain and contract configuration for EtherFi Borrowing (Cash) on Scroll

pub const SCROLL_CHAIN_ID: u64 = 534352;
pub const DEFAULT_RPC_URL: &str = "https://rpc.scroll.io";

// --- Core contract addresses (Scroll mainnet) ---
pub const DEBT_MANAGER: &str = "0x8f9d2Cd33551CE06dD0564Ba147513F715c2F4a0";
pub const CASH_DATA_PROVIDER: &str = "0xb1F5bBc3e4DE0c767ace41EAb8A28b837fBA966F";
pub const USER_SAFE_LENS: &str = "0x333321a783f765bFd4c22FBBC5B2D02b97efB44c";
pub const USER_SAFE_FACTORY: &str = "0x18Fa07dF94b4E9F09844e1128483801B24Fe8a27";

// --- Token addresses (Scroll mainnet) ---
pub const USDC_ADDR: &str = "0x06eFdBFf2a14a7c8E15944D1F4A48F9F95F663A4";
pub const WEETH_ADDR: &str = "0x01f0a31698C4d065659b9bdC21B3610292a1c506";
pub const SCR_ADDR: &str = "0xd29687c813D741E2F938F4aC377128810E217b1b";

/// Token info for display
pub struct TokenInfo {
    pub symbol: &'static str,
    pub address: &'static str,
    pub decimals: u8,
}

pub const BORROW_TOKENS: &[TokenInfo] = &[
    TokenInfo { symbol: "USDC", address: USDC_ADDR, decimals: 6 },
];

pub const COLLATERAL_TOKENS: &[TokenInfo] = &[
    TokenInfo { symbol: "weETH", address: WEETH_ADDR, decimals: 18 },
    TokenInfo { symbol: "USDC",  address: USDC_ADDR,  decimals: 6  },
    TokenInfo { symbol: "SCR",   address: SCR_ADDR,   decimals: 18 },
];

/// Resolve token address from symbol (case-insensitive)
pub fn resolve_token_address(symbol: &str) -> Option<(&'static str, u8)> {
    match symbol.to_uppercase().as_str() {
        "USDC" => Some((USDC_ADDR, 6)),
        "WEETH" => Some((WEETH_ADDR, 18)),
        "SCR" => Some((SCR_ADDR, 18)),
        _ => None,
    }
}

/// Resolve token symbol from address
pub fn resolve_token_symbol(addr: &str) -> &'static str {
    let a = addr.to_lowercase();
    if a == USDC_ADDR.to_lowercase() { return "USDC"; }
    if a == WEETH_ADDR.to_lowercase() { return "weETH"; }
    if a == SCR_ADDR.to_lowercase() { return "SCR"; }
    "UNKNOWN"
}
