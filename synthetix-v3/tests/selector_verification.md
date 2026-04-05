# Selector Verification — Synthetix V3

All selectors verified with `cast sig` (Foundry).

| 函数签名 | cast sig 结果 | 代码中的值 | 状态 |
|---------|-------------|----------|------|
| `deposit(uint128,address,uint256)` | `0x83802968` | `0x83802968` | ✅ |
| `withdraw(uint128,address,uint256)` | `0x95997c51` | `0x95997c51` | ✅ |
| `createAccount()` | `0x9dca362f` | design.md only | ✅ |
| `createAccount(uint128)` | `0xcadb09a5` | design.md only | ✅ |
| `getAccountCollateral(uint128,address)` | `0xef45148e` | `0xef45148e` | ✅ |
| `getAccountAvailableCollateral(uint128,address)` | `0x927482ff` | `0x927482ff` | ✅ |
| `getCollateralConfigurations(bool)` | `0x75bf2444` | design.md only | ✅ |
| `getCollateralPrice(address)` | `0x51a40994` | design.md only | ✅ |
| `getPreferredPool()` | `0x3b390b57` | design.md only | ✅ |
| `getPoolName(uint128)` | `0xf86e6f91` | design.md only | ✅ |
| `getMarkets()` | `0xec2c9016` | `0xec2c9016` | ✅ |
| `getMarketSummary(uint128)` | `0x41c2e8bd` | `0x41c2e8bd` | ✅ |
| `getAvailableMargin(uint128)` | `0x0a7dad2d` | `0x0a7dad2d` | ✅ |
| `getOpenPosition(uint128,uint128)` | `0x22a73967` | `0x22a73967` | ✅ |
| `currentFundingRate(uint128)` | `0xd435b2a2` | `0xd435b2a2` | ✅ |
| `getAccountOpenPositions(uint128)` | `0x35254238` | `0x35254238` | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | `0x095ea7b3` | ✅ |
| `balanceOf(address)` | `0x70a08231` | `0x70a08231` | ✅ |
