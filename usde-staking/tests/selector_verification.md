# Selector Verification

All selectors verified with `cast sig` before use in code.

| Function Signature | Selector | Status |
|-------------------|---------|--------|
| `approve(address,uint256)` | `0x095ea7b3` | ✅ |
| `deposit(uint256,address)` | `0x6e553f65` | ✅ |
| `redeem(uint256,address,address)` | `0xba087652` | ✅ |
| `cooldownShares(uint256)` | `0x9343d9e1` | ✅ |
| `cooldownAssets(uint256)` | `0xcdac52ed` | ✅ |
| `unstake(address)` | `0xf2888dbb` | ✅ |
| `balanceOf(address)` | `0x70a08231` | ✅ |
| `convertToAssets(uint256)` | `0x07a2d13a` | ✅ |
| `convertToShares(uint256)` | `0xc6e6f592` | ✅ |
| `previewDeposit(uint256)` | `0xef8b30f7` | ✅ |
| `previewRedeem(uint256)` | `0x4cdad506` | ✅ |
| `cooldowns(address)` | `0x01320fe2` | ✅ (also verified on-chain) |
| `cooldownDuration()` | `0x35269315` | ✅ |
| `totalAssets()` | `0x01e1d114` | ✅ |
