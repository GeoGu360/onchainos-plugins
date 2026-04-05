# Selector Verification — swell-restaking

All selectors verified via `cast sig` (Foundry).

| Function Signature | Expected Selector | Verified |
|-------------------|------------------|---------|
| `deposit()` | `0xd0e30db0` | ✅ |
| `rswETHToETHRate()` | `0xa7b9544e` | ✅ |
| `ethToRswETHRate()` | `0x780a47e0` | ✅ |
| `getRate()` | `0x679aefce` | ✅ |
| `totalETHDeposited()` | `0x7b2c9070` | ✅ |
| `balanceOf(address)` | `0x70a08231` | ✅ |
| `totalSupply()` | `0x18160ddd` | ✅ |

## Verification Commands

```bash
cast sig "deposit()"           # 0xd0e30db0
cast sig "rswETHToETHRate()"   # 0xa7b9544e
cast sig "ethToRswETHRate()"   # 0x780a47e0
cast sig "getRate()"           # 0x679aefce
cast sig "totalETHDeposited()" # 0x7b2c9070
cast sig "balanceOf(address)"  # 0x70a08231
cast sig "totalSupply()"       # 0x18160ddd
```
