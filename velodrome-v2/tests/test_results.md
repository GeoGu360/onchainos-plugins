# Test Results Report

- Date: 2026-04-05
- DApp Supported Chains: EVM only (Optimism, chain 10)
- EVM Test Chain: Optimism (10) — no funds for on-chain write ops
- Fallback: L4 write ops SKIPPED (zero ETH/USDC on Optimism test wallet)
- Compilation: pass
- Lint: pass
- Overall Pass Criteria: EVM DApp -> EVM ops all pass (L2/L3 full coverage; L4 SKIPPED due to funding)

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|-----------|---------|------------|------------|--------|---------|
| 13    | 2         | 5       | 5          | 1 (SKIP)   | 0      | 1       |

## Detailed Results

| # | Scenario (User View) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Compile velodrome-v2 binary | L1 | `cargo build --release` | PASS | - | Compiled in ~1m25s |
| 2 | Run plugin-store lint | L1 | `cargo clean && plugin-store lint .` | PASS | - | 0 errors |
| 3 | Query WETH/USDC volatile pool on Velodrome | L2 | `velodrome-v2 pools --token-a WETH --token-b USDC --stable false` | PASS | - | Pool 0xf4f2657...ab returned with reserves |
| 4 | Get swap quote: 0.00005 WETH -> USDC | L2 | `velodrome-v2 quote --token-in WETH --token-out USDC --amount-in 50000000000000` | PASS | - | amountOut=101902 USDC units (both volatile and stable pools found) |
| 5 | Query pool by address | L2 | `velodrome-v2 pools --pool 0xf4f2657ae744354baca871e56775e5083f7276ab` | PASS | - | token0=USDC, token1=WETH, reserves correct |
| 6 | Check LP positions for wallet | L2 | `velodrome-v2 positions` | PASS | - | Wallet resolved: 0x87fb..., no positions (empty wallet) |
| 7 | Claim-rewards dry-run (gauge lookup) | L2 | `velodrome-v2 claim-rewards --token-a WETH --token-b USDC --dry-run` | PASS | - | Gauge 0xbde5e... found via Voter |
| 8 | Simulate swap WETH->USDC | L3 | `velodrome-v2 swap --token-in WETH --token-out USDC --amount-in 50000000000000 --dry-run` | PASS | Selector 0xcac88ea9 (swapExactTokensForTokens) | dry_run=true, zero txHash |
| 9 | Simulate add-liquidity WETH/USDC | L3 | `velodrome-v2 add-liquidity --token-a WETH --token-b USDC --amount-a-desired 50000000000000 --dry-run` | PASS | Selector 0x5a47ddc3 (addLiquidity), amountB auto-quoted=102211 | dry_run=true |
| 10 | Simulate remove-liquidity WETH/USDC | L3 | `velodrome-v2 remove-liquidity --token-a WETH --token-b USDC --dry-run` | PASS | Selector 0x0dede6c4 (removeLiquidity) | dry_run=true, mock LP=1e18 |
| 11 | Simulate claim-rewards from gauge | L3 | `velodrome-v2 claim-rewards --token-a WETH --token-b USDC --dry-run` | PASS | Selector 0xc00007b0 (getReward) | dry_run=true |
| 12 | Simulate swap USDC->WETH | L3 | `velodrome-v2 swap --token-in USDC --token-out WETH --amount-in 10000 --dry-run` | PASS | dry_run=true | - |
| 13 | Real swap on Optimism (0.01 USDC->WETH) | L4 | `velodrome-v2 swap --token-in USDC --token-out WETH --amount-in 10000` | SKIP | - | Test wallet has 0 ETH + 0 USDC on Optimism (chain 10). Lark notif sent. |

## Fix Record

No bugs found. Clean pass on L1-L3. L4 skipped due to zero funding on Optimism.

## Selector Verification

| Operation | Function Signature | Selector | Verified |
|-----------|-------------------|----------|---------|
| swap | `swapExactTokensForTokens(uint256,uint256,(address,address,bool,address)[],address,uint256)` | `0xcac88ea9` | cast sig verified |
| add-liquidity | `addLiquidity(address,address,bool,uint256,uint256,uint256,uint256,address,uint256)` | `0x5a47ddc3` | cast sig verified |
| remove-liquidity | `removeLiquidity(address,address,bool,uint256,uint256,uint256,address,uint256)` | `0x0dede6c4` | cast sig verified |
| claim-rewards | `getReward(address)` | `0xc00007b0` | cast sig verified |
| approve | `approve(address,uint256)` | `0x095ea7b3` | standard ERC-20 |
| getPool | `getPool(address,address,bool)` | `0x79bc57d5` | cast sig verified |
| getAmountsOut | `getAmountsOut(uint256,(address,address,bool,address)[])` | `0x5509a1ac` | cast sig verified |
