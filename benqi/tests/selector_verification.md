# Selector Verification — Benqi Lending

All selectors verified with `cast sig` (Foundry).

| Operation | Function Signature | Selector | Status |
|-----------|-------------------|---------|--------|
| supply ERC20 approve | `approve(address,uint256)` | `0x095ea7b3` | ✅ |
| supply ERC20 mint | `mint(uint256)` | `0xa0712d68` | ✅ |
| supply AVAX mint | `mint()` | `0x1249c58b` | ✅ |
| redeem underlying | `redeemUnderlying(uint256)` | `0x852a12e3` | ✅ |
| borrow | `borrow(uint256)` | `0xc5ebeaec` | ✅ |
| repay ERC20 | `repayBorrow(uint256)` | `0x0e752702` | ✅ |
| repay AVAX | `repayBorrow()` | `0x4e4d9fea` | ✅ |
| claim rewards | `claimReward(uint8,address)` | `0x0952c563` | ✅ |
| getAllMarkets | `getAllMarkets()` | `0xb0772d0b` | ✅ (eth_call verified) |
| getAccountLiquidity | `getAccountLiquidity(address)` | `0x5ec88c79` | ✅ |
| supplyRatePerTimestamp | `supplyRatePerTimestamp()` | `0xd3bd2c72` | ✅ (eth_call verified) |
| borrowRatePerTimestamp | `borrowRatePerTimestamp()` | `0xcd91801c` | ✅ |
| exchangeRateCurrent | `exchangeRateCurrent()` | `0xbd6d894d` | ✅ (eth_call verified) |
| balanceOf | `balanceOf(address)` | `0x70a08231` | ✅ |
| borrowBalanceCurrent | `borrowBalanceCurrent(address)` | `0x17bfdfbc` | ✅ |
