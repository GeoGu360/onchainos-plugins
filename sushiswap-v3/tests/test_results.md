# Test Results Report — SushiSwap V3

- Date: 2026-04-05
- Test Chain: Base (8453)
- Compilation: PASS
- Lint: SKIP (plugin-store binary not available in environment)

---

## Summary

| Total | L1 Build | L2 Read | L3 Dry-run | L4 On-chain | Failed | Blocked |
|-------|----------|---------|------------|-------------|--------|---------|
| 11    | 1        | 4       | 4          | 2           | 0      | 0       |

---

## Detailed Results

| # | Scenario (user view) | Level | Command | Result | TxHash / Calldata | Notes |
|---|----------------------|-------|---------|--------|-------------------|-------|
| 1 | Build release binary | L1 | `cargo build --release` | PASS | — | Compiled in ~19s |
| 2 | Query WETH/USDC pools on Base (all fee tiers) | L2 | `get-pools --token0 WETH --token1 USDC --chain 8453` | PASS | — | 4 pools deployed: fee=100 (0x482fe...), 500 (0x57713...), 3000 (0x41595...), 10000 (0x6fa08...) |
| 3 | Quote 0.001 ETH WETH→USDC (auto fee) | L2 | `quote --token-in WETH --token-out USDC --amount-in 1000000000000000 --chain 8453` | PASS | — | bestFee=500, amountOut=2051116 (~$2.05 USDC) |
| 4 | Quote WETH→USDC fee=500 explicit | L2 | `quote --token-in WETH --token-out USDC --amount-in 1000000000000000 --fee 500 --chain 8453` | PASS | — | amountOut=2051116, fee=500 |
| 5 | Get LP positions for test wallet (empty) | L2 | `get-positions --owner 0x87fb... --chain 8453` | PASS | — | 0 positions, ok=true |
| 6 | Dry-run swap WETH→USDC calldata | L3 | `swap --token-in WETH --token-out USDC --amount-in 50000000000000 --dry-run --chain 8453` | PASS | calldata starts 0x414bf389 (exactInputSingle) | Selector 0x414bf389 verified |
| 7 | Dry-run add-liquidity with negative ticks | L3 | `add-liquidity ... --tick-lower -887270 --tick-upper 887270 --dry-run` | PASS | calldata starts 0x88316456 (mint) | Negative ticks accepted via allow_hyphen_values=true |
| 8 | Dry-run collect-fees (no fees owed) | L3 | `collect-fees --token-id 99999 --dry-run --chain 8453` | PASS | — | Returns early with "no fees owed" message |
| 9 | Dry-run remove-liquidity (zero liquidity pos) | L3 | `remove-liquidity --token-id 99999 --dry-run --chain 8453` | PASS | txHash=0x000...000 | decreaseLiquidity+collect both dry-run |
| 10 | On-chain ETH→USDC swap via Sushi API | L4 | Direct RouteProcessor4 call, 0.00005 ETH | PASS | `0x0d15bb9297a724b3fd6d8f5bba7feaab532352261940b091ec4562f82bea7a34` | 0.00005 ETH (~$0.10) swapped to ~102703 USDC units (~$0.10 USDC) |
| 11 | On-chain plugin swap WETH→USDC via SwapRouter | L4 | `swap --token-in WETH --token-out USDC --amount-in 50000000000000 --chain 8453` | PASS | approve: `0x07764cdd...`, swap: `0xea20495b...` | Auto-approved WETH for SwapRouter; fee=100 (best); amountOutMin=101584 |

---

## Bugs Found and Fixed

None. Plugin compiled and ran correctly on first attempt.

---

## Test Wallet Context

- Wallet: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
- Starting ETH: 0.003 ETH (6.16 USD)
- ETH spent in L4: ~0.00015 ETH (wrap + 2 swaps + gas)
- Remaining ETH: ~0.00285 ETH (well above 0.001 reserve)

---

## Notes

- `plugin-store lint` skipped — binary not in PATH in this environment
- All on-chain operations confirmed via txHash lookup
- The plugin correctly uses `--force` flag for EVM DEX approve and swap operations
- SushiSwap V3 fee tiers differ from PancakeSwap V3: uses 3000 (0.3%) instead of 2500 (0.25%)
- Contract addresses confirmed same across all chains (deterministic CREATE2 deployment)
