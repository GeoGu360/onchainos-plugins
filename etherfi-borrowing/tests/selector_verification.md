# Function Selector Verification — EtherFi Borrowing (Cash)

All selectors verified via `cast sig` (Foundry) against live Scroll mainnet contracts.

## DebtManager (0x8f9d2Cd33551CE06dD0564Ba147513F715c2F4a0)

| Function | Canonical Signature | Selector | Verified |
|---|---|---|---|
| getBorrowTokens | `getBorrowTokens()` | `0x5a52477a` | ✅ (cast + live eth_call) |
| getCollateralTokens | `getCollateralTokens()` | `0xb58eb63f` | ✅ (cast + live eth_call) |
| borrowApyPerSecond | `borrowApyPerSecond(address)` | `0x944e2f5e` | ✅ (cast + live eth_call) |
| totalSupplies | `totalSupplies(address)` | `0x9782e821` | ✅ (cast + live eth_call) |
| totalBorrowingAmount | `totalBorrowingAmount(address)` | `0xc94f8d42` | ✅ (cast) |
| collateralTokenConfig | `collateralTokenConfig(address)` | `0xf0ba097e` | ✅ (cast + live eth_call) |
| collateralOf | `collateralOf(address)` | `0x1aefb107` | ✅ (cast) |
| borrowingOf | `borrowingOf(address,address)` | `0x4142152e` | ✅ (cast) |
| remainingBorrowingCapacityInUSD | `remainingBorrowingCapacityInUSD(address)` | `0xf6513bfe` | ✅ (cast) |
| liquidatable | `liquidatable(address)` | `0xffec70af` | ✅ (cast) |
| supplierBalance | `supplierBalance(address,address)` | `0x58061652` | ✅ (cast) |
| supply | `supply(address,address,uint256)` | `0x0c0a769b` | ✅ (cast) |
| withdrawBorrowToken | `withdrawBorrowToken(address,uint256)` | `0xa56c8ff7` | ✅ (cast) |
| repay | `repay(address,address,uint256)` | `0x1da649cf` | ✅ (cast) |

## ERC-20 Standard Selectors

| Function | Selector | Verified |
|---|---|---|
| `approve(address,uint256)` | `0x095ea7b3` | ✅ standard |
| `balanceOf(address)` | `0x70a08231` | ✅ standard |
| `allowance(address,address)` | `0xdd62ed3e` | ✅ standard |

## Verification Commands Used

```bash
cast sig "getBorrowTokens()"
cast sig "getCollateralTokens()"
cast sig "borrowApyPerSecond(address)"
cast sig "totalSupplies(address)"
cast sig "totalBorrowingAmount(address)"
cast sig "collateralTokenConfig(address)"
cast sig "collateralOf(address)"
cast sig "borrowingOf(address,address)"
cast sig "remainingBorrowingCapacityInUSD(address)"
cast sig "liquidatable(address)"
cast sig "supplierBalance(address,address)"
cast sig "supply(address,address,uint256)"
cast sig "withdrawBorrowToken(address,uint256)"
cast sig "repay(address,address,uint256)"

# Live verification on Scroll:
cast call 0x8f9d2Cd33551CE06dD0564Ba147513F715c2F4a0 "getBorrowTokens()(address[])" --rpc-url https://rpc.scroll.io
cast call 0x8f9d2Cd33551CE06dD0564Ba147513F715c2F4a0 "getCollateralTokens()(address[])" --rpc-url https://rpc.scroll.io
cast call 0x8f9d2Cd33551CE06dD0564Ba147513F715c2F4a0 "borrowApyPerSecond(address)(uint64)" 0x06eFdBFf2a14a7c8E15944D1F4A48F9F95F663A4 --rpc-url https://rpc.scroll.io
cast call 0x8f9d2Cd33551CE06dD0564Ba147513F715c2F4a0 "totalSupplies(address)(uint256)" 0x06eFdBFf2a14a7c8E15944D1F4A48F9F95F663A4 --rpc-url https://rpc.scroll.io
cast call 0x8f9d2Cd33551CE06dD0564Ba147513F715c2F4a0 "collateralTokenConfig(address)((uint80,uint80,uint96))" 0x01f0a31698C4d065659b9bdC21B3610292a1c506 --rpc-url https://rpc.scroll.io
```
