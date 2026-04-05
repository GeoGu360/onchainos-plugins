/// Chain and contract configuration for ether.fi Liquid vaults

pub const ETHEREUM_CHAIN_ID: u64 = 1;
pub const DEFAULT_RPC_URL: &str = "https://ethereum.publicnode.com";

// ---- ETH Yield Vault (LIQUIDETH) ----
pub const ETH_VAULT_BORING_VAULT: &str = "0xf0bb20865277abd641a307ece5ee04e79073416c";
pub const ETH_VAULT_TELLER: &str = "0x9AA79C84b79816ab920bBcE20f8f74557B514734";
pub const ETH_VAULT_ACCOUNTANT: &str = "0x0d05D94a5F1E76C18fbeB7A13d17C8a314088198";

// ---- USD Yield Vault (LIQUIDUSD) ----
pub const USD_VAULT_BORING_VAULT: &str = "0x08c6F91e2B681FaF5e17227F2a44C307b3C1364C";
pub const USD_VAULT_TELLER: &str = "0x4DE413a26fC24c3FC27Cc983be70aA9c5C299387";
pub const USD_VAULT_ACCOUNTANT: &str = "0xc315D6e14DDCDC7407784e2Caf815d131Bc1D3E7";

// ---- BTC Yield Vault (LIQUIDBTC) ----
pub const BTC_VAULT_BORING_VAULT: &str = "0x5f46d540b6eD704C3c8789105F30E075AA900726";
pub const BTC_VAULT_TELLER: &str = "0x8Ea0B382D054dbEBeB1d0aE47ee4AC433C730353";
pub const BTC_VAULT_ACCOUNTANT: &str = "0xEa23aC6D7D11f6b181d6B98174D334478ADAe6b0";

// ---- Token addresses ----
pub const WEETH_ADDR: &str = "0xCd5fE23C85820F7B72D0926FC9b05b43E359b7ee";
pub const WETH_ADDR: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
pub const EETH_ADDR: &str = "0x35fA164735182de50811E8e2E824cFb9B6118ac2";
pub const USDC_ADDR: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
pub const WBTC_ADDR: &str = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599";

// ---- DefiLlama pool IDs ----
pub const DEFILLAMA_LIQUIDETH_POOL_ID: &str = "b86d4934-2e75-415a-bdd2-e28143d72491";
pub const DEFILLAMA_LIQUIDUSD_POOL_ID: &str = "7c12f175-37bc-41db-967a-1d7f1f4a23c4";
pub const DEFILLAMA_LIQUIDBTC_POOL_ID: &str = "2f063aed-0a5a-4ba5-8b63-8404b2a99fca";
pub const DEFILLAMA_YIELDS_BASE: &str = "https://yields.llama.fi";

/// Vault info struct for listing
pub struct VaultInfo {
    pub name: &'static str,
    pub symbol: &'static str,
    pub vault: &'static str,
    pub teller: &'static str,
    pub accountant: &'static str,
    pub deposit_token: &'static str,
    pub deposit_token_symbol: &'static str,
    pub deposit_token_decimals: u8,
    pub defillama_pool_id: &'static str,
}

pub const VAULTS: &[VaultInfo] = &[
    VaultInfo {
        name: "ETH Yield Vault",
        symbol: "LIQUIDETH",
        vault: ETH_VAULT_BORING_VAULT,
        teller: ETH_VAULT_TELLER,
        accountant: ETH_VAULT_ACCOUNTANT,
        deposit_token: WEETH_ADDR,
        deposit_token_symbol: "weETH",
        deposit_token_decimals: 18,
        defillama_pool_id: DEFILLAMA_LIQUIDETH_POOL_ID,
    },
    VaultInfo {
        name: "USD Yield Vault",
        symbol: "LIQUIDUSD",
        vault: USD_VAULT_BORING_VAULT,
        teller: USD_VAULT_TELLER,
        accountant: USD_VAULT_ACCOUNTANT,
        deposit_token: USDC_ADDR,
        deposit_token_symbol: "USDC",
        deposit_token_decimals: 6,
        defillama_pool_id: DEFILLAMA_LIQUIDUSD_POOL_ID,
    },
    VaultInfo {
        name: "BTC Yield Vault",
        symbol: "LIQUIDBTC",
        vault: BTC_VAULT_BORING_VAULT,
        teller: BTC_VAULT_TELLER,
        accountant: BTC_VAULT_ACCOUNTANT,
        deposit_token: WBTC_ADDR,
        deposit_token_symbol: "WBTC",
        deposit_token_decimals: 8,
        defillama_pool_id: DEFILLAMA_LIQUIDBTC_POOL_ID,
    },
];

/// Resolve vault teller and vault address by symbol
pub fn resolve_vault(symbol: &str) -> Option<&'static VaultInfo> {
    VAULTS.iter().find(|v| v.symbol.eq_ignore_ascii_case(symbol))
}
