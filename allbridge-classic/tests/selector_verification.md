# Selector Verification

All EVM function selectors verified with `cast sig`.

| Function | Canonical Signature | Expected Selector | Verified |
|----------|--------------------|--------------------|---------|
| lock | `lock(uint128,address,bytes32,bytes4,uint256)` | `0x7bacc91e` | ✅ |
| lockBase | `lockBase(uint128,address,bytes32,bytes4)` | `0x5e3b0f9e` | ✅ |
| unlock | `unlock(uint128,address,uint256,bytes4,bytes4,bytes32,bytes)` | `0x14b824e0` | ✅ |
| approve | `approve(address,uint256)` | `0x095ea7b3` | ✅ |

Verification commands run:
```bash
cast sig "lock(uint128,address,bytes32,bytes4,uint256)"   # 0x7bacc91e
cast sig "lockBase(uint128,address,bytes32,bytes4)"        # 0x5e3b0f9e
cast sig "unlock(uint128,address,uint256,bytes4,bytes4,bytes32,bytes)"  # 0x14b824e0
cast sig "approve(address,uint256)"                        # 0x095ea7b3
```
