# Function Selector Verification

All selectors verified with `cast sig` (Foundry).

| Function Signature | Selector | Status | Usage |
|-------------------|----------|--------|-------|
| `supplyEth(address)` | `0x87ee9312` | ✅ | iETH v1 — deposit ETH |
| `supply(address,uint256,address)` | `0x8b2a4df5` | ✅ | iETH v1 — deposit WETH/stETH |
| `withdraw(uint256,address)` | `0x00f714ce` | ✅ | iETH v1 — withdraw iETH shares |
| `getCurrentExchangePrice()` | `0xcc4a0158` | ✅ | iETH v1 — exchange price |
| `netAssets()` | `0x0782d421` | ✅ | iETH v1 — net assets/TVL |
| `approve(address,uint256)` | `0x095ea7b3` | ✅ | ERC-20 approve (for iETHv2) |
| `deposit(uint256,address)` | `0x6e553f65` | ✅ | iETHv2 — ERC-4626 deposit |
| `redeem(uint256,address,address)` | `0xba087652` | ✅ | iETHv2 — ERC-4626 redeem |
| `exchangePrice()` | `0x9e65741e` | ✅ | iETHv2 — exchange price |
| `getNetAssets()` | `0x08bb5fb0` | ✅ | iETHv2 — net assets |
| `totalAssets()` | `0x01e1d114` | ✅ | iETHv2 — total assets |
| `totalSupply()` | `0x18160ddd` | ✅ | ERC-20 totalSupply |
| `balanceOf(address)` | `0x70a08231` | ✅ | ERC-20 balanceOf |
| `asset()` | `0x38d52e0f` | ✅ | iETHv2 — underlying token |

## Verification Commands

```bash
cast sig "supplyEth(address)"              # 0x87ee9312
cast sig "supply(address,uint256,address)" # 0x8b2a4df5
cast sig "withdraw(uint256,address)"       # 0x00f714ce
cast sig "getCurrentExchangePrice()"       # 0xcc4a0158
cast sig "netAssets()"                     # 0x0782d421
cast sig "approve(address,uint256)"        # 0x095ea7b3
cast sig "deposit(uint256,address)"        # 0x6e553f65
cast sig "redeem(uint256,address,address)" # 0xba087652
cast sig "exchangePrice()"                 # 0x9e65741e
cast sig "getNetAssets()"                  # 0x08bb5fb0
cast sig "totalAssets()"                   # 0x01e1d114
cast sig "totalSupply()"                   # 0x18160ddd
cast sig "balanceOf(address)"              # 0x70a08231
cast sig "asset()"                         # 0x38d52e0f
```
