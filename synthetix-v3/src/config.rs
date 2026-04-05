/// Synthetix V3 Base Andromeda deployment — chain 8453
/// Addresses sourced from @synthetixio/v3-contracts v8.10.0 npm package

pub const BASE_CHAIN_ID: u64 = 8453;
pub const BASE_RPC_URL: &str = "https://base-rpc.publicnode.com";

// Core contracts
pub const CORE_PROXY: &str = "0x32C222A9A159782aFD7529c87FA34b96CA72C696";
pub const ACCOUNT_PROXY: &str = "0x63f4Dd0434BEB5baeCD27F3778a909278d8cf5b8";
pub const USD_PROXY: &str = "0x09d51516F38980035153a554c26Df3C6f51a23C3";
pub const SPOT_MARKET_PROXY: &str = "0x18141523403e2595D31b22604AcB8Fc06a4CaA61";
pub const PERPS_MARKET_PROXY: &str = "0x0A2AF931eFFd34b81ebcc57E3d3c9B1E1dE1C9Ce";
pub const PERPS_ACCOUNT_PROXY: &str = "0xcb68b813210aFa0373F076239Ad4803f8809e8cf";

// Collateral tokens
pub const USDC: &str = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";
pub const SUSDC: &str = "0xC74eA762cF06c9151cE074E6a569a5945b6302E7";
pub const WETH: &str = "0x4200000000000000000000000000000000000006";

// Pool IDs
pub const SPARTAN_COUNCIL_POOL_ID: u64 = 1;
pub const PERPS_SUPER_MARKET_ID: u64 = 2;

// sUSDC decimals
pub const SUSDC_DECIMALS: u32 = 18;
pub const USDC_DECIMALS: u32 = 6;
