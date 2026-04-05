# Function Selector Verification

All selectors verified via `cast sig` (Foundry).

| Function | Signature | Selector | Status |
|----------|-----------|----------|--------|
| Vault.swap | `swap((bytes32,uint8,address,address,uint256,bytes),(address,bool,address,bool),uint256,uint256)` | `0x52bbbe29` | ✅ |
| Vault.batchSwap | `batchSwap(uint8,(bytes32,uint256,uint256,uint256,bytes)[],address[],(address,bool,address,bool),int256[],uint256)` | `0x945bcec9` | ✅ |
| Vault.joinPool | `joinPool(bytes32,address,address,(address[],uint256[],bytes,bool))` | `0xb95cac28` | ✅ |
| Vault.exitPool | `exitPool(bytes32,address,address,(address[],uint256[],bytes,bool))` | `0x8bdb3913` | ✅ |
| Vault.getPool | `getPool(bytes32)` | `0xf6c00927` | ✅ |
| Vault.getPoolTokens | `getPoolTokens(bytes32)` | `0xf94d4668` | ✅ |
| BalancerQueries.querySwap | `querySwap((bytes32,uint8,address,address,uint256,bytes),(address,bool,address,bool))` | `0xe969f6b3` | ✅ |
| BalancerQueries.queryJoin | `queryJoin(bytes32,address,address,(address[],uint256[],bytes,bool))` | `0x9ebbf05d` | ✅ |
| BalancerQueries.queryExit | `queryExit(bytes32,address,address,(address[],uint256[],bytes,bool))` | `0xc7b2c52c` | ✅ |
| Pool.getNormalizedWeights | `getNormalizedWeights()` | `0xf89f27ed` | ✅ |
| Pool.getSwapFeePercentage | `getSwapFeePercentage()` | `0x55c67628` | ✅ |
| Pool.getPoolId | `getPoolId()` | `0x38fff2d0` | ✅ |
| ERC-20.totalSupply | `totalSupply()` | `0x18160ddd` | ✅ |
| ERC-20.balanceOf | `balanceOf(address)` | `0x70a08231` | ✅ |
| ERC-20.approve | `approve(address,uint256)` | `0x095ea7b3` | ✅ |
| ERC-20.allowance | `allowance(address,address)` | `0xdd62ed3e` | ✅ |
| ERC-20.decimals | `decimals()` | `0x313ce567` | ✅ |

**Tool used:** `cast sig "<function_signature>"` (Foundry v1.5.1)

**Note on ABI encoding bug found during testing:**
The `querySwap` calldata had a subtle bug in `bytes userData` offset within the `SingleSwap` tuple.
The offset within the tuple must point past all 6 head slots (including the offset slot itself):
- Incorrect: `5 * 32 = 0xa0` (counted 5 static fields, forgetting the offset slot itself)
- Correct: `6 * 32 = 0xc0` (6 head slots total: poolId + kind + assetIn + assetOut + amount + this_offset)

Verified correct encoding via `cast calldata` comparison.
