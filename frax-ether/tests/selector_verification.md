# Selector Verification — frax-ether

All EVM function selectors verified with `cast sig`.

| Function Signature | cast sig Result | Code Value | Status |
|-------------------|----------------|------------|--------|
| `submit()` | `0x5bcb2fc6` | `5bcb2fc6` | ✅ |
| `deposit(uint256,address)` | `0x6e553f65` | `6e553f65` | ✅ |
| `redeem(uint256,address,address)` | `0xba087652` | `ba087652` | ✅ |
| `convertToAssets(uint256)` | `0x07a2d13a` | `07a2d13a` | ✅ |
| `convertToShares(uint256)` | `0xc6e6f592` | `c6e6f592` | ✅ |
| `balanceOf(address)` | `0x70a08231` | `70a08231` | ✅ |
| `totalAssets()` | `0x01e1d114` | `01e1d114` | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | `095ea7b3` | ✅ |
| `submitPaused()` | `0xb6d24f18` | N/A (read-only check) | ✅ |

## Notes

- `submit(address)` (selector `0xa1903eab`) reverts when called from test wallet — unclear why (possibly a referral whitelist). `submit()` (no args, `0x5bcb2fc6`) works correctly and mints frxETH.
- Both frxETHMinter functions are verified on-chain (submit() confirmed via L4 tx).
- sfrxETH ERC-4626 selectors confirmed standard (deposit/redeem verified on Etherscan).
