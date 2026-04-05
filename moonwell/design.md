# Moonwell Flagship Plugin Design

## Overview

Moonwell is an open lending and borrowing protocol built on Base, Optimism, Moonbeam, and Moonriver. It is a fork of Compound V2 using mTokens (instead of cTokens) and timestamps instead of blocks for interest accrual.

## Architecture

```
User
 ├── markets       → Comptroller.getAllMarkets() + mToken rates (read)
 ├── positions     → mToken.balanceOf + borrowBalanceCurrent per market (read)
 ├── supply        → ERC20.approve + mToken.mint(amount) (write)
 ├── redeem        → mToken.redeem(mTokenAmount) (write)
 ├── borrow        → mToken.borrow(amount) (dry-run only)
 ├── repay         → ERC20.approve + mToken.repayBorrow(amount) (dry-run only)
 └── claim-rewards → Comptroller.claimReward(address) (write)
```

## Key Contracts — Base (8453)

| Contract | Address |
|----------|---------|
| Comptroller | `0xfBb21d0380beE3312B33c4353c8936a0F13EF26C` |
| WELL Token | `0xA88594D404727625A9437C3f886C7643872296AE` |
| mUSDC | `0xEdc817A28E8B93B03976FBd4a3dDBc9f7D176c22` |
| mWETH | `0x628ff693426583D9a7FB391E54366292F509D457` |
| mcbETH | `0x3bf93770f2d4a794c3d9EBEfBAeBAE2a8f09A5E5` |
| mUSDbC | `0x703843C3379b52F9FF486c9f5892218d2a065cC8` |
| mDAI | `0x73b06D8d18De422E269645eaCe15400DE7462417` |
| Multi-Reward Distributor | `0xe9005b078701e2A0948D2EaC43010D35870Ad9d2` |
| stkWELL | `0xe66E3A37C3274Ac24FE8590f7D84A2427194DC17` |

## Supported Assets on Base

| Symbol | Underlying Address | Decimals | mToken Address |
|--------|-------------------|----------|----------------|
| USDC | `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913` | 6 | `0xEdc817A28E8B93B03976FBd4a3dDBc9f7D176c22` |
| WETH | `0x4200000000000000000000000000000000000006` | 18 | `0x628ff693426583D9a7FB391E54366292F509D457` |
| cbETH | `0x2Ae3F1Ec7F1F5012CFEab0185bfc7aa3cf0DEc22` | 18 | `0x3bf93770f2d4a794c3d9EBEfBAeBAE2a8f09A5E5` |
| USDbC | `0xd9aAEc86B65D86f6A7B5B1b0c42FFA531710b6CA` | 6 | `0x703843C3379b52F9FF486c9f5892218d2a065cC8` |
| DAI | `0x50c5725949A6F0c72E6C4a641F24049A917DB0Cb` | 18 | `0x73b06D8d18De422E269645eaCe15400DE7462417` |

## Key Difference from Compound V2

- Moonwell uses **timestamp-based** interest accrual (`supplyRatePerTimestamp`, `borrowRatePerTimestamp`) scaled by 1e18
- Comptroller uses `claimReward(address)` selector `0xd279c191` (not `claimComp`)
- mTokens follow exact same cToken interface for mint/redeem/borrow/repay

## Function Selectors (verified via `cast sig`)

| Function | Selector |
|----------|---------|
| `mint(uint256)` | `0xa0712d68` |
| `redeem(uint256)` | `0xdb006a75` |
| `redeemUnderlying(uint256)` | `0x852a12e3` |
| `borrow(uint256)` | `0xc5ebeaec` |
| `repayBorrow(uint256)` | `0x0e752702` |
| `claimReward(address)` | `0xd279c191` |
| `getAllMarkets()` | `0xb0772d0b` |
| `supplyRatePerTimestamp()` | `0xd3bd2c72` |
| `borrowRatePerTimestamp()` | `0xcd91801c` |
| `exchangeRateCurrent()` | `0xbd6d894d` |
| `balanceOf(address)` | `0x70a08231` |
| `borrowBalanceCurrent(address)` | `0x17bfdfbc` |
| `approve(address,uint256)` | `0x095ea7b3` |
| `underlying()` | `0x6f307dc3` |
| `decimals()` | `0x313ce567` |
| `getAccountLiquidity(address)` | `0x5ec88c79` |

## Rate Calculation (Timestamp-based)

- `supplyRatePerTimestamp()` returns rate per second (scaled by 1e18)
- Seconds per year = 31,536,000
- APY = rate_per_second * seconds_per_year / 1e18 * 100

## Operations Detail

### markets
- Query each known mToken: `supplyRatePerTimestamp()`, `borrowRatePerTimestamp()`, `exchangeRateCurrent()`
- Calculate APR, display market info

### positions
- For each market, call `balanceOf(wallet)` and `borrowBalanceCurrent(wallet)`
- Convert mToken balance to underlying via `exchangeRateCurrent()`

### supply
1. `ERC20.approve(mToken, amount)` → tx1
2. Wait 3s for nonce safety
3. `mToken.mint(amount)` → tx2
- Ask user to confirm before executing

### redeem
- `mToken.redeem(mTokenAmount)` → burns mTokens, receives underlying
- Ask user to confirm

### borrow (dry-run only)
- `mToken.borrow(amount)` — requires sufficient collateral
- Only in dry-run mode for safety

### repay (dry-run only)
- `ERC20.approve(mToken, amount)` + `mToken.repayBorrow(amount)`
- Only in dry-run mode for safety

### claim-rewards
- `Comptroller.claimReward(wallet)` — claims all accrued WELL rewards

## RPC Endpoint
- Base: `https://base.publicnode.com`
- Optimism: `https://optimism.publicnode.com`
- Moonbeam: `https://moonbeam.publicnode.com`

## Supported Chains
- Base (8453) — primary
- Optimism (10)
- Moonbeam (1284)
- Moonriver (1285)

## Constraints
- borrow and repay: dry-run only (safety)
- Max test tx: 0.01 USDC or 0.00005 ETH
- Reserve ≥ 0.001 ETH for gas
