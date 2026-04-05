# Selector Verification — Trader Joe LB

All selectors verified with `cast sig` (Foundry).

| Function Signature | cast sig Result | Code Value | Status |
|-------------------|-----------------|------------|--------|
| `swapExactTokensForTokens(uint256,uint256,(uint256[],uint8[],address[]),address,uint256)` | `0x2a443fae` | `0x2a443fae` | ✅ |
| `swapExactNATIVEForTokens(uint256,(uint256[],uint8[],address[]),address,uint256)` | `0xb066ea7c` | `0xb066ea7c` | ✅ |
| `swapExactTokensForNATIVE(uint256,uint256,(uint256[],uint8[],address[]),address payable,uint256)` | `0x9ab6156b` | `0x9ab6156b` | ✅ |
| `getAllLBPairs(address,address)` | `0x6622e0d7` | `0x6622e0d7` | ✅ |
| `getLBPairInformation(address,address,uint256)` | `0x704037bd` | `0x704037bd` | ✅ |
| `findBestPathFromAmountIn(address[],uint128)` | `0x0f902a40` | `0x0f902a40` | ✅ |
| `findBestPathFromAmountOut(address[],uint128)` | `0x59214226` | `0x59214226` | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | `0x095ea7b3` | ✅ |
| `allowance(address,address)` | `0xdd62ed3e` | `0xdd62ed3e` | ✅ |
| `getActiveId()` | `0xdbe65edc` | `0xdbe65edc` | ✅ (fixed from 0xd9e3fc8f) |
