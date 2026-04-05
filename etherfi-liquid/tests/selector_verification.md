# Selector Verification Checklist

All selectors verified with `cast sig` (Foundry) on 2026-04-05.

| Function Signature | cast sig Result | Code Value | Status |
|---|---|---|---|
| `deposit(address,uint256,uint256,address)` | `0x8b6099db` | `0x8b6099db` | ✅ |
| `bulkWithdraw(address,uint256[],uint256[],address[])` | `0x8432f02b` | `0x8432f02b` | ✅ |
| `assetData(address)` | `0x41fee44a` | `0x41fee44a` | ✅ |
| `vault()` | `0xfbfa77cf` | `0xfbfa77cf` | ✅ |
| `getRateInQuote(address)` | `0x1dcbb110` | `0x1dcbb110` | ✅ |
| `getRateInQuoteSafe(address)` | `0x820973da` | `0x820973da` | ✅ |
| `totalSupply()` | `0x18160ddd` | `0x18160ddd` | ✅ |
| `balanceOf(address)` | `0x70a08231` | `0x70a08231` | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | `0x095ea7b3` | ✅ |
| `allowance(address,address)` | `0xdd62ed3e` | `0xdd62ed3e` | ✅ |

All 10 selectors verified. ✅
