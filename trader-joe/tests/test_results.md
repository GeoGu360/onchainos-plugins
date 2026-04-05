# Test Results Report — Trader Joe DEX

- Date: 2026-04-05
- DApp supported chains: EVM only (Arbitrum 42161)
- EVM test chain: Arbitrum One (42161)
- Compiled: ✅ PASS
- Lint: SKIP (plugin-store not installed in environment)
- Overall standard: EVM DApp → EVM all pass

---

## Summary

| Total | L1 Build | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|----------|---------|-------------|-------------|--------|---------|
| 7     | 1        | 4       | 2           | 1           | 0      | 0       |

---

## Detailed Results

| # | Scenario (user perspective) | Level | Command | Result | Calldata / TxHash | Notes |
|---|-----------------------------|-------|---------|--------|-------------------|-------|
| 1 | Build plugin from source | L1 | `cargo build --release` | ✅ PASS | — | Compiled cleanly, 0 warnings |
| 2 | Get quote: 0.01 USDT → WETH | L2 | `--chain 42161 quote --from USDT --to WETH --amount 0.01 --decimals 6` | ✅ PASS | — | amountOut ~0.0000049 WETH, binStep=15, version=V2_1, pair=0xd387c40a |
| 3 | Get quote: 0.001 WETH → USDT | L2 | `--chain 42161 quote --from WETH --to USDT --amount 0.001 --decimals 18` | ✅ PASS | — | amountOut ~2.023 USDT, binStep=15, version=V2_1 |
| 4 | List WETH/USDT pools | L2 | `--chain 42161 pools --token-x WETH --token-y USDT` | ✅ PASS | — | 4 pools found: binStep 10, 100, 25, 50 |
| 5 | List WETH/USDC pools | L2 | `--chain 42161 pools --token-x WETH --token-y USDC` | ✅ PASS | — | 3 pools found: binStep 10, 100, 50 |
| 6 | Simulate swap 0.01 USDT → WETH (dry-run) | L3 | `--chain 42161 --dry-run swap --from USDT --to WETH --amount 0.01 --decimals 6` | ✅ PASS | calldata: `0x2a443fae...` (selector ✅) | amountOutMin=4863720060424, zero addr recipient |
| 7 | Simulate swap 0.001 WETH → USDT (dry-run) | L3 | `--chain 42161 --dry-run swap --from WETH --to USDT --amount 0.001 --decimals 18` | ✅ PASS | calldata: `0x2a443fae...` (selector ✅) | amountOutMin=2013165 |
| 8 | Swap 0.01 USDT → WETH on Trader Joe LB | L4 | `--chain 42161 swap --from USDT --to WETH --amount 0.01 --decimals 6 --slippage-bps 100` | ✅ PASS | approve: `0x7fb6fc10...` swap: `0x5bfecc0a...` | Arbiscan: https://arbiscan.io/tx/0x5bfecc0a40d3611a194371e96ab770542fb1cfb7696246d205f17e21cd865465 |

---

## Fix Log

| # | Issue | Root Cause | Fix | File |
|---|-------|-----------|-----|------|
| 1 | ABI decode returned garbage values | Outer struct wrapper not accounted for — response has extra 0x20 outer offset | Rewrote `parse_quote_response` to respect outer struct offset pointer | `src/commands/quote.rs` |
| 2 | `getActiveId()` wrong selector 0xd9e3fc8f | Manual computation error | Corrected to 0xdbe65edc via `cast sig "getActiveId()"` | `src/commands/pools.rs` |
