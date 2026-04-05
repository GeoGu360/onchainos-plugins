# Function Selector Verification

All selectors verified via `python3 keccak` (Keccak-256 of function signature).

| Function | Signature | Expected Selector | Verified |
|----------|-----------|------------------|---------|
| ERC-4626 deposit | `deposit(uint256,address)` | `0x6e553f65` | ✅ |
| ERC-4626 redeem | `redeem(uint256,address,address)` | `0xba087652` | ✅ |
| ERC-4626 withdraw | `withdraw(uint256,address,address)` | `0xb460af94` | ✅ |
| ERC-20 approve | `approve(address,uint256)` | `0x095ea7b3` | ✅ |
| ERC-20 balanceOf | `balanceOf(address)` | `0x70a08231` | ✅ |
| ERC-20 decimals | `decimals()` | `0x313ce567` | ✅ |
| ERC-20 symbol | `symbol()` | `0x95d89b41` | ✅ |
| ERC-4626 convertToAssets | `convertToAssets(uint256)` | `0x07a2d13a` | ✅ |
| fToken asset() | `asset()` | `0x38d52e0f` | ✅ |
| LendingResolver getAllFTokens | `getFTokensEntireData()` | `0xe26533a3` | ✅ |
| LendingResolver getUserPositions | `getUserPositions(address)` | `0x2a6bc2dd` | ✅ |
| Dex swapIn | `swapIn(bool,uint256,uint256,address)` | `0x2668dfaa` | ✅ |
| Dex swapOut | `swapOut(bool,uint256,uint256,address)` | `0x286f0e61` | ✅ |

## Verification Method

```python
from Crypto.Hash import keccak
def sel(sig):
    k = keccak.new(digest_bits=256)
    k.update(sig.encode())
    return '0x' + k.hexdigest()[:8]
```

All selectors confirmed match the deployed contract ABIs from
`github.com/Instadapp/fluid-contracts-public/deployments/base/`.
