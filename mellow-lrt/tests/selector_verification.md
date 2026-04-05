# Selector 核对清单

All selectors verified with `cast sig` (Foundry) on 2026-04-05.

| 函数签名 | cast sig 结果 | 代码中的值 | 状态 |
|---------|-------------|----------|------|
| `deposit(uint256,address)` | `0x6e553f65` | `0x6e553f65` | ✅ |
| `redeem(uint256,address,address)` | `0xba087652` | `0xba087652` | ✅ |
| `deposit(address,uint256,address,address,address)` | `0x0bb9f5e1` | `0x0bb9f5e1` | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | `0x095ea7b3` | ✅ |
| `balanceOf(address)` | `0x70a08231` | `0x70a08231` | ✅ |
| `convertToAssets(uint256)` | `0x07a2d13a` | `0x07a2d13a` | ✅ |
| `convertToShares(uint256)` | `0xc6e6f592` | N/A (design ref only) | ✅ |
| `totalAssets()` | `0x01e1d114` | `0x01e1d114` | ✅ |
| `maxDeposit(address)` | `0x402d267d` | `0x402d267d` | ✅ |
| `claimableAssetsOf(address)` | `0xe7beaf9d` | `0xe7beaf9d` | ✅ |
| `pendingAssetsOf(address)` | `0x63c6b4eb` | `0x63c6b4eb` | ✅ |
| `claim(address,address,uint256)` | `0x996cba68` | `0x996cba68` | ✅ |
| `asset()` | `0x38d52e0f` | `0x38d52e0f` | ✅ |
| `decimals()` | `0x313ce567` | `0x313ce567` | ✅ |
| `symbol()` | `0x95d89b41` | `0x95d89b41` | ✅ |

All 15 selectors verified. No mismatches found.
