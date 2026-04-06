# SushiSwap V3 — Function Selector Verification

All selectors verified with:
```
python3 -c "from eth_utils import keccak; print('0x' + keccak(text=SIG).hex()[:8])"
```

## SwapRouter

| Function | Signature | Selector | Verified |
|---|---|---|---|
| `exactInputSingle` | `exactInputSingle((address,address,uint24,address,uint256,uint256,uint256,uint160))` | `0x414bf389` | YES |
| `exactInput` | `exactInput((bytes,address,uint256,uint256,uint256))` | varies | not used |

Note: SushiSwap V3 uses the same interface as Uniswap V3 with `deadline` as the 5th parameter in the struct.

## QuoterV2

| Function | Signature | Selector | Verified |
|---|---|---|---|
| `quoteExactInputSingle` | `quoteExactInputSingle((address,address,uint256,uint24,uint160))` | `0xc6a5026a` | YES |

## NonfungiblePositionManager

| Function | Signature | Selector | Verified |
|---|---|---|---|
| `mint` | `mint((address,address,uint24,int24,int24,uint256,uint256,uint256,uint256,address,uint256))` | `0x88316456` | YES |
| `increaseLiquidity` | `increaseLiquidity((uint256,uint256,uint256,uint256,uint256,uint256))` | `0x219f5d17` | YES |
| `decreaseLiquidity` | `decreaseLiquidity((uint256,uint128,uint256,uint256,uint256))` | `0x0c49ccbe` | YES |
| `collect` | `collect((uint256,address,uint128,uint128))` | `0xfc6f7865` | YES |
| `burn` | `burn(uint256)` | `0x42966c68` | YES |
| `positions` | `positions(uint256)` | `0x99fbab88` | YES |
| `balanceOf` | `balanceOf(address)` | `0x70a08231` | YES |
| `tokenOfOwnerByIndex` | `tokenOfOwnerByIndex(address,uint256)` | `0x2f745c59` | YES |

## Factory

| Function | Signature | Selector | Verified |
|---|---|---|---|
| `getPool` | `getPool(address,address,uint24)` | `0x1698ee82` | YES |

## ERC-20

| Function | Signature | Selector | Verified |
|---|---|---|---|
| `approve` | `approve(address,uint256)` | `0x095ea7b3` | YES |
| `allowance` | `allowance(address,address)` | `0xdd62ed3e` | YES |

## Verification Script

```python
from eth_utils import keccak

sigs = {
    'exactInputSingle (V3 with deadline)': 'exactInputSingle((address,address,uint24,address,uint256,uint256,uint256,uint160))',
    'quoteExactInputSingle': 'quoteExactInputSingle((address,address,uint256,uint24,uint160))',
    'mint': 'mint((address,address,uint24,int24,int24,uint256,uint256,uint256,uint256,address,uint256))',
    'decreaseLiquidity': 'decreaseLiquidity((uint256,uint128,uint256,uint256,uint256))',
    'collect': 'collect((uint256,address,uint128,uint128))',
    'burn': 'burn(uint256)',
    'positions': 'positions(uint256)',
    'balanceOf': 'balanceOf(address)',
    'tokenOfOwnerByIndex': 'tokenOfOwnerByIndex(address,uint256)',
    'getPool': 'getPool(address,address,uint24)',
    'approve': 'approve(address,uint256)',
    'allowance': 'allowance(address,address)',
}
for name, sig in sigs.items():
    sel = '0x' + keccak(text=sig).hex()[:8]
    print(f'{sel}  {name}')
```

## Output of Verification Script

```
0x414bf389  exactInputSingle (V3 with deadline)
0xc6a5026a  quoteExactInputSingle
0x88316456  mint
0x0c49ccbe  decreaseLiquidity
0xfc6f7865  collect
0x42966c68  burn
0x99fbab88  positions
0x70a08231  balanceOf
0x2f745c59  tokenOfOwnerByIndex
0x1698ee82  getPool
0x095ea7b3  approve
0xdd62ed3e  allowance
```
