# Moonwell Selector Verification

All selectors verified via `cast sig` (foundry).

## mToken (Compound V2 fork interface)

| Function | Selector | Verified |
|----------|---------|---------|
| `mint(uint256)` | `0xa0712d68` | `cast sig "mint(uint256)"` = 0xa0712d68 |
| `redeem(uint256)` | `0xdb006a75` | `cast sig "redeem(uint256)"` = 0xdb006a75 |
| `redeemUnderlying(uint256)` | `0x852a12e3` | `cast sig "redeemUnderlying(uint256)"` = 0x852a12e3 |
| `borrow(uint256)` | `0xc5ebeaec` | `cast sig "borrow(uint256)"` = 0xc5ebeaec |
| `repayBorrow(uint256)` | `0x0e752702` | `cast sig "repayBorrow(uint256)"` = 0x0e752702 |
| `exchangeRateCurrent()` | `0xbd6d894d` | `cast sig "exchangeRateCurrent()"` = 0xbd6d894d |
| `balanceOf(address)` | `0x70a08231` | `cast sig "balanceOf(address)"` = 0x70a08231 |
| `borrowBalanceCurrent(address)` | `0x17bfdfbc` | `cast sig "borrowBalanceCurrent(address)"` = 0x17bfdfbc |
| `underlying()` | `0x6f307dc3` | `cast sig "underlying()"` = 0x6f307dc3 |
| `decimals()` | `0x313ce567` | `cast sig "decimals()"` = 0x313ce567 |

## Moonwell-Specific (Timestamp-based rates)

| Function | Selector | Verified |
|----------|---------|---------|
| `supplyRatePerTimestamp()` | `0xd3bd2c72` | `cast sig "supplyRatePerTimestamp()"` = 0xd3bd2c72 |
| `borrowRatePerTimestamp()` | `0xcd91801c` | `cast sig "borrowRatePerTimestamp()"` = 0xcd91801c |

## Comptroller

| Function | Selector | Verified |
|----------|---------|---------|
| `getAllMarkets()` | `0xb0772d0b` | `cast sig "getAllMarkets()"` = 0xb0772d0b |
| `claimReward(address)` | `0xd279c191` | `cast sig "claimReward(address)"` = 0xd279c191 |
| `getAccountLiquidity(address)` | `0x5ec88c79` | `cast sig "getAccountLiquidity(address)"` = 0x5ec88c79 |

## ERC-20

| Function | Selector | Verified |
|----------|---------|---------|
| `approve(address,uint256)` | `0x095ea7b3` | `cast sig "approve(address,uint256)"` = 0x095ea7b3 |

## Key Difference from Compound V2

- Moonwell uses `supplyRatePerTimestamp()` / `borrowRatePerTimestamp()` instead of `PerBlock` variants
- Moonwell Comptroller uses `claimReward(address)` (selector 0xd279c191) instead of `claimComp(address)`
- Both confirm the protocol is a genuine Compound V2 fork with timestamp-based accrual
