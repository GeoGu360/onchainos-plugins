# Selector Verification

All selectors verified with `cast sig` (Foundry keccak256).

| Function signature | cast sig result | Code value | Status |
|---|---|---|---|
| `depositETH()` | `0xf6326fb3` | `0xf6326fb3` | ✅ |
| `depositETH(uint256)` | `0x5358fbda` | `0x5358fbda` | ✅ |
| `deposit(address,uint256)` | `0x47e7ef24` | `0x47e7ef24` | ✅ |
| `deposit(address,uint256,uint256)` | `0x0efe6a8b` | `0x0efe6a8b` | ✅ |
| `calculateTVLs()` | `0xff9969cd` | `0xff9969cd` | ✅ |
| `getCollateralTokensLength()` | `0x75c745a6` | `0x75c745a6` | ✅ |
| `collateralTokens(uint256)` | `0x172c48c7` | `0x172c48c7` | ✅ |
| `renzoOracle()` | `0x892866a4` | `0x892866a4` | ✅ |
| `ezETH()` | `0x13a73c78` | `0x13a73c78` | ✅ |
| `paused()` | `0x5c975abb` | `0x5c975abb` | ✅ |
| `balanceOf(address)` | `0x70a08231` | `0x70a08231` | ✅ |
| `totalSupply()` | `0x18160ddd` | `0x18160ddd` | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | `0x095ea7b3` | ✅ |
| `allowance(address,address)` | `0xdd62ed3e` | `0xdd62ed3e` | ✅ |

All 14 selectors verified. No mismatches.
