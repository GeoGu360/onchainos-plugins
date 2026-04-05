// Curve Lending — static configuration
// All addresses are read-only constants; never hardcoded in business logic.

/// OneWayLendingFactory on Ethereum mainnet
pub const LENDING_FACTORY: &str = "0xeA6876DDE9e3467564acBeE1Ed5bac88783205E0";

/// crvUSD — the borrowable stablecoin in all "long" Curve Lending markets
pub const CRVUSD: &str = "0xf939E0A03FB07F59A73314E73794Be0E57ac1b4e";

/// ERC-20 approve selector
pub const SEL_APPROVE: &str = "095ea7b3";

// Factory read selectors (all verified via `cast sig`)
pub const SEL_MARKET_COUNT: &str = "fd775c78";
pub const SEL_NAMES: &str = "4622ab03";
pub const SEL_CONTROLLERS: &str = "e94b0dd2";
pub const SEL_VAULTS: &str = "8c64ea4a";
pub const SEL_COLLATERAL_TOKENS: &str = "49b89984";
pub const SEL_BORROWED_TOKENS: &str = "6fe4501f";
pub const SEL_MONETARY_POLICIES: &str = "762e7b92";

// Controller read selectors
pub const SEL_N_LOANS: &str = "6cce39be";
pub const SEL_TOTAL_DEBT: &str = "31dc3ca8";
pub const SEL_LOAN_EXISTS: &str = "a21adb9e";
pub const SEL_DEBT: &str = "9b6c56ec";
pub const SEL_USER_STATE: &str = "ec74d0a8";
pub const SEL_USER_PRICES: &str = "2c5089c3";
pub const SEL_HEALTH: &str = "8908ea82";
pub const SEL_MAX_BORROWABLE: &str = "9a497196";

// Controller write selectors
pub const SEL_CREATE_LOAN: &str = "23cfed03";
pub const SEL_ADD_COLLATERAL: &str = "24049e57";
pub const SEL_BORROW_MORE: &str = "dd171e7c";
pub const SEL_REPAY: &str = "371fd8e6"; // repay(uint256)

// Vault selectors
pub const SEL_TOTAL_ASSETS: &str = "01e1d114";
pub const SEL_LEND_APY: &str = "1eb25c42";
pub const SEL_BORROW_APY: &str = "3ca97d20";
pub const SEL_TOTAL_SUPPLY: &str = "18160ddd";

// Monetary policy selectors
pub const SEL_MP_RATE: &str = "0ba9d8ca"; // rate(address)
pub const SEL_MP_MIN_RATE: &str = "5d786401"; // cast sig "min_rate()" ✅
pub const SEL_MP_MAX_RATE: &str = "536e4ec4"; // cast sig "max_rate()" ✅

// ERC-20 selectors
pub const SEL_BALANCE_OF: &str = "70a08231";
pub const SEL_DECIMALS: &str = "313ce567";
pub const SEL_SYMBOL: &str = "95d89b41";

/// RPC for Ethereum mainnet — avoid llamarpc/cloudflare (rate-limited or blocked)
pub fn get_rpc(chain_id: u64) -> &'static str {
    match chain_id {
        1 => "https://ethereum.publicnode.com",
        _ => "https://ethereum.publicnode.com",
    }
}
