# Selector Verification Checklist

All selectors verified with `cast sig` (Foundry).

| Function Signature | cast sig Result | Code Value | Status |
|-------------------|----------------|-----------|--------|
| `mint(uint256)` | `0xa0712d68` | `0xa0712d68` (alloy sol! macro) | ✅ |
| `mint()` (vBNB payable) | `0x1249c58b` | `0x1249c58b` (alloy sol! macro) | ✅ |
| `redeemUnderlying(uint256)` | `0x852a12e3` | `0x852a12e3` (alloy sol! macro) | ✅ |
| `redeem(uint256)` | `0xdb006a75` | `0xdb006a75` | ✅ |
| `borrow(uint256)` | `0xc5ebeaec` | `0xc5ebeaec` (alloy sol! macro) | ✅ |
| `repayBorrow(uint256)` | `0x0e752702` | `0x0e752702` (alloy sol! macro) | ✅ |
| `enterMarkets(address[])` | `0xc2998238` | `0xc2998238` (hardcoded, manually verified) | ✅ |
| `claimVenus(address)` | `0xadcd5fb9` | `0xadcd5fb9` (hardcoded, manually verified) | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | `0x095ea7b3` (in onchainos.rs) | ✅ |
| `getAllMarkets()` | `0xb0772d0b` | `0xb0772d0b` (in rpc.rs) | ✅ |
| `getAccountSnapshot(address)` | `0xc37f68e2` | `0xc37f68e2` (in rpc.rs) | ✅ |
| `getAccountLiquidity(address)` | `0x5ec88c79` | `0x5ec88c79` (in rpc.rs) | ✅ |
| `supplyRatePerBlock()` | `0xae9d70b0` | `0xae9d70b0` (in rpc.rs) | ✅ |
| `borrowRatePerBlock()` | `0xf8f9da28` | `0xf8f9da28` (in rpc.rs) | ✅ |
| `totalBorrows()` | `0x47bd3718` | `0x47bd3718` (in rpc.rs) | ✅ |
| `getCash()` | `0x3b1d21a2` | `0x3b1d21a2` (in rpc.rs) | ✅ |
| `exchangeRateCurrent()` | `0xbd6d894d` | `0xbd6d894d` (in rpc.rs) | ✅ |
| `underlying()` | `0x6f307dc3` | `0x6f307dc3` (in rpc.rs) | ✅ |
| `symbol()` | `0x95d89b41` | `0x95d89b41` (in rpc.rs) | ✅ |
| `decimals()` | `0x313ce567` | `0x313ce567` (in rpc.rs) | ✅ |
| `balanceOf(address)` | `0x70a08231` | `0x70a08231` (in rpc.rs) | ✅ |

All 21 selectors verified. ✅
