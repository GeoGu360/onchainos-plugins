# Function Selector Verification

All selectors verified with `cast sig` and `eth_utils.keccak` (Python).

| Function | Signature | Selector | Status |
|---|---|---|---|
| `healthFactor(address,address)` | `healthFactor(address,address)` | `0x576f5c40` | ✅ |
| `balanceOfCollateral(address,address)` | `balanceOfCollateral(address,address)` | `0xda3a855f` | ✅ |
| `enterPosition(address,address,uint256,uint256,bytes)` | `enterPosition(address,address,uint256,uint256,bytes)` | `0xde13c617` | ✅ |
| `exitPosition(address,address,uint256,uint16,bytes)` | `exitPosition(address,address,uint256,uint16,bytes)` | `0x8a363181` | ✅ |
| `initiateWithdraw(address,address,uint256)` | `initiateWithdraw(address,address,uint256)` | `0x37753799` | ✅ |
| `claimRewards(address,address)` | `claimRewards(address,address)` | `0xf1e42ccd` | ✅ |
| ERC-20 `approve(address,uint256)` | `approve(address,uint256)` | `0x095ea7b3` | ✅ |

## Verification commands used

```bash
cast sig "healthFactor(address,address)"
# 0x576f5c40

cast sig "balanceOfCollateral(address,address)"
# 0xda3a855f

cast sig "enterPosition(address,address,uint256,uint256,bytes)"
# 0xde13c617

cast sig "exitPosition(address,address,uint256,uint16,bytes)"
# 0x8a363181

cast sig "initiateWithdraw(address,address,uint256)"
# 0x37753799

cast sig "claimRewards(address,address)"
# 0xf1e42ccd

cast sig "approve(address,uint256)"
# 0x095ea7b3
```

## Notes
- ABI encoding is done with `alloy-sol-types` `sol!{}` macro (not manual hex concatenation)
- Keccak-256 used (NOT SHA3-256) — per KNOWLEDGE_HUB.md guardrail
