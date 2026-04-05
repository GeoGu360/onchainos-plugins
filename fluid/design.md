# Fluid Protocol Plugin — Design Document

## Protocol Overview

Fluid is a combined DEX + Lending protocol by Instadapp, deployed on Ethereum, Base, and Arbitrum. It consists of:

- **Fluid Lending** — ERC-4626 fToken contracts (fUSDC, fWETH, etc.) where users deposit assets and earn yield
- **Fluid DEX** — Novel concentrated AMM with swapIn/swapOut interface, deployed as individual pool contracts per pair
- **Fluid Vault** — Collateral-based borrowing system (vault protocol)

All funds flow through a central **Liquidity** contract; individual protocols build on top of it.

---

## Supported Chains

| Chain | Chain ID |
|-------|----------|
| Base (primary test) | 8453 |
| Ethereum Mainnet | 1 |
| Arbitrum | 42161 |

---

## Contract Addresses

### Base (8453) — Primary Test Chain

| Contract | Address |
|----------|---------|
| LendingResolver | `0x48D32f49aFeAEC7AE66ad7B9264f446fc11a1569` |
| VaultResolver | `0xA5C3E16523eeeDDcC34706b0E6bE88b4c6EA95cC` |
| DexResolver | `0x11D80CfF056Cef4F9E6d23da8672fE9873e5cC07` |
| LiquidityResolver | `0xca13A15de31235A37134B4717021C35A3CF25C60` |
| fToken_fUSDC | `0xf42f5795D9ac7e9D757dB633D693cD548Cfd9169` |
| fToken_fWETH | `0x9272D6153133175175Bc276512B2336BE3931CE9` |
| fToken_fGHO | `0x8DdbfFA3CFda2355a23d6B11105AC624BDbE3631` |
| fToken_fEURC | `0x1943FA26360f038230442525Cf1B9125b5DCB401` |
| fToken_fsUSDS | `0x...` (from fToken_fsUSDS.json) |
| fToken_fwstETH | `0x...` (from fToken_fwstETH.json) |
| Dex_EURC_USDC | `0x2886a01a0645390872a9eb99dAe1283664b0c524` |
| Dex_USDe_USDC | `0x836951EB21F3Df98273517B7249dCEFF270d34bf` |
| Dex_wstETH_ETH | `0x667701e51B4D1Ca244F17C78F7aB8744B4C99F9B` |
| Dex_weETH_ETH | `0x3C0441B42195F4aD6aa9a0978E06096ea616CDa7` |
| Dex_FLUID_ETH | `0xdE632C3a214D5f14C1d8ddF0b92F8BCd188fee45` |

### Ethereum Mainnet (1)

| Contract | Address |
|----------|---------|
| LendingResolver | `0x48D32f49aFeAEC7AE66ad7B9264f446fc11a1569` |
| VaultResolver | `0xA5C3E16523eeeDDcC34706b0E6bE88b4c6EA95cC` |
| DexResolver | `0x11D80CfF056Cef4F9E6d23da8672fE9873e5cC07` |
| LiquidityResolver | `0xca13A15de31235A37134B4717021C35A3CF25C60` |
| fToken_fUSDC | `0x9Fb7b4477576Fe5B32be4C1843aFB1e55F251B33` |
| fToken_fWETH | `0x90551c1795392094FE6D29B758EcCD233cFAa260` |
| fToken_fUSDT | `0x5C20B550819128074FD538Edf79791733ccEdd18` |

### Arbitrum (42161)

| Contract | Address |
|----------|---------|
| LendingResolver | `0x48D32f49aFeAEC7AE66ad7B9264f446fc11a1569` |
| VaultResolver | `0xA5C3E16523eeeDDcC34706b0E6bE88b4c6EA95cC` |
| DexResolver | `0x11D80CfF056Cef4F9E6d23da8672fE9873e5cC07` |
| LiquidityResolver | `0xca13A15de31235A37134B4717021C35A3CF25C60` |
| fToken_fUSDC | `0x1A996cb54bb95462040408C06122D45D6Cdb6096` |
| fToken_fWETH | `0x45Df0656F8aDf017590009d2f1898eeca4F0a205` |
| fToken_fUSDT | `0x4A03F37e7d3fC243e3f99341d36f4b829BEe5E03` |

### Token Addresses

#### Base (8453)
| Symbol | Address |
|--------|---------|
| USDC | `0x833589fcd6edb6e08f4c7c32d4f71b54bda02913` |
| WETH | `0x4200000000000000000000000000000000000006` |
| wstETH | `0xc1cba3fcea344f92d9239c08c0568f6f2f0ee452` |
| EURC | `0x60a3e35cc302bfa44cb288bc5a4f316fdb1adb42` |

#### Ethereum (1)
| Symbol | Address |
|--------|---------|
| USDC | `0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48` |
| WETH | `0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2` |
| USDT | `0xdac17f958d2ee523a2206206994597c13d831ec7` |
| wstETH | `0x7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0` |

#### Arbitrum (42161)
| Symbol | Address |
|--------|---------|
| USDC | `0xaf88d065e77c8cc2239327c5edb3a432268e5831` |
| WETH | `0x82af49447d8a07e3bd95bd0d56f35241523fbab1` |
| USDT | `0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9` |

---

## Operations

### Read Operations (no wallet required)

#### `markets` — List fToken lending markets
- Calls `getFTokensEntireData()` on LendingResolver → `0xe26533a3`
- Returns: fToken address, name, symbol, underlying asset, totalAssets, supplyRate, APY
- No auth needed

#### `positions` — User lending positions
- Calls `getUserPositions(address)` on LendingResolver → `0x2a6bc2dd`
- Returns: per-fToken: fTokenShares, underlyingAssets, underlyingBalance, allowance
- Requires wallet address

#### `quote` — DEX swap quote (read)
- Calls `swapIn` or `swapOut` with `estimate_` flag equivalent (read-only simulation via eth_call)
- Actual approach: use eth_call on the dex pool's `swapIn(swap0to1, amountIn, 0, address(0))` to simulate

### Write Operations (wallet required, ask user to confirm)

#### `supply` — Supply to fToken (ERC-4626 deposit)
- Step 1: `approve(fToken, amount)` on underlying token → selector `0x095ea7b3`
- Step 2: `deposit(uint256 assets, address receiver)` on fToken → selector `0x6e553f65`
- Requires: fToken address (or symbol like fUSDC), amount

#### `withdraw` — Withdraw from fToken
- Partial: `withdraw(uint256 assets, address receiver, address owner)` → selector `0xb460af94`
- Full (`--all`): `redeem(uint256 shares, address receiver, address owner)` → selector `0xba087652`
- Requires: fToken address (or symbol), amount or `--all`

#### `borrow` — Borrow from Fluid Vault (DRY-RUN ONLY — liquidation risk)
- Would call vault borrow function
- Dry-run only to avoid liquidation risk

#### `repay` — Repay Fluid Vault debt (DRY-RUN ONLY — liquidation risk)
- Would call vault repay function
- Dry-run only

#### `swap` — Swap via Fluid DEX
- Calls `swapIn(bool swap0to1, uint256 amountIn, uint256 amountOutMin, address to)` → selector `0x2668dfaa`
- Or `swapOut(bool swap0to1, uint256 amountOut, uint256 amountInMax, address to)` → selector `0x286f0e61`
- Requires: DEX pool address (or token pair), token direction, amount
- If token is ERC-20: Step 1 approve the pool, Step 2 swapIn/swapOut
- If ETH involved: send msg.value via `--amt`

---

## Function Selectors

| Function | Signature | Selector |
|----------|-----------|----------|
| ERC-4626 deposit | `deposit(uint256,address)` | `0x6e553f65` |
| ERC-4626 redeem | `redeem(uint256,address,address)` | `0xba087652` |
| ERC-4626 withdraw | `withdraw(uint256,address,address)` | `0xb460af94` |
| ERC-20 approve | `approve(address,uint256)` | `0x095ea7b3` |
| ERC-20 balanceOf | `balanceOf(address)` | `0x70a08231` |
| ERC-4626 convertToAssets | `convertToAssets(uint256)` | `0x07a2d13a` |
| LendingResolver getAllFTokens | `getFTokensEntireData()` | `0xe26533a3` |
| LendingResolver getUserPositions | `getUserPositions(address)` | `0x2a6bc2dd` |
| Dex swapIn | `swapIn(bool,uint256,uint256,address)` | `0x2668dfaa` |
| Dex swapOut | `swapOut(bool,uint256,uint256,address)` | `0x286f0e61` |

---

## Architecture Notes

### Fluid Lending (fTokens)
- Standard ERC-4626 interface
- The fToken IS the vault — users deposit directly to fToken address
- No separate vault factory needed for supply/withdraw
- `LendingResolver.getUserPositions(user)` returns all positions across all fTokens
- `LendingResolver.getFTokensEntireData()` returns all fTokens with rates

### Fluid DEX
- Each pair is a separate pool contract
- Pool has `swapIn(swap0to1, amountIn, amountOutMin, to)` and `swapOut(swap0to1, amountOut, amountInMax, to)`
- `swap0to1 = true` means selling token0 to get token1
- ETH pools are payable — ETH sent via msg.value
- ERC-20 pools require prior approval to the pool address

### DEX Pool Token Ordering
- token0 and token1 are determined at pool creation
- Must determine token0/token1 order from pool's `constantsView()` function
- Or deduce from the pair name (e.g. Dex_EURC_USDC → token0=EURC, token1=USDC)

---

## RPC Endpoints

| Chain | RPC URL |
|-------|---------|
| Base (8453) | `https://base-rpc.publicnode.com` |
| Ethereum (1) | `https://eth.llamarpc.com` |
| Arbitrum (42161) | `https://arbitrum-one-rpc.publicnode.com` |

---

## Plugin Structure

```
fluid/
├── plugin.yaml
├── skills/fluid/SKILL.md
├── src/
│   ├── main.rs          — CLI entry point (clap)
│   ├── config.rs        — Chain configs, contract addresses, token maps
│   ├── onchainos.rs     — wallet_contract_call, resolve_wallet, erc20_approve
│   ├── rpc.rs           — eth_call helpers (balanceOf, decimals, symbol)
│   ├── calldata.rs      — ABI encoding for all operations
│   └── commands/
│       ├── mod.rs
│       ├── markets.rs   — getFTokensEntireData read
│       ├── positions.rs — getUserPositions read
│       ├── supply.rs    — ERC-4626 deposit (approve + deposit)
│       ├── withdraw.rs  — ERC-4626 withdraw/redeem
│       ├── borrow.rs    — dry-run only
│       ├── repay.rs     — dry-run only
│       ├── swap.rs      — DEX swapIn/swapOut
│       └── quote.rs     — DEX quote (eth_call simulation)
├── Cargo.toml
├── .gitignore
├── LICENSE
└── README.md
```

---

## Key Design Decisions

1. **No external API** — All data read from on-chain resolver contracts via eth_call
2. **fToken-centric supply** — User specifies fToken symbol (fUSDC, fWETH) or address; plugin looks up underlying asset from config
3. **DEX pool registry** — Known pool addresses hardcoded per chain; users specify token pair
4. **Borrow/repay dry-run only** — Vault operations have liquidation risk; full execution disabled
5. **resolve_wallet** — Uses `onchainos wallet addresses` with correct chainIndex lookup per spec
