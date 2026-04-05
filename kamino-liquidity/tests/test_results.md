# Test Results Report — kamino-liquidity

- Date: 2026-04-05
- DApp supported chains: Solana only
- Solana test chain: mainnet (501)
- Compile: ✅
- Lint: ✅
- **Overall pass standard**: Solana DApp → Solana all pass

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Fail | Blocked |
|-------|------------|---------|------------|------------|------|---------|
| 9     | 3          | 4       | 2          | 3          | 0    | 0       |

## Detailed Results

| # | Scenario (user view) | Level | Command | Result | TxHash / Notes |
|---|---------------------|-------|---------|--------|---------------|
| 1 | Build debug binary | L1 | `cargo build` | ✅ PASS | 1 warning (unused SOLANA_CHAIN_ID const) |
| 2 | Build release binary | L1 | `cargo build --release` | ✅ PASS | — |
| 3 | Plugin lint check | L1 | `cargo clean && plugin-store lint .` | ✅ PASS | 0 errors, 0 warnings |
| 4 | List all Kamino vaults | L2 | `vaults --chain 501 --limit 5` | ✅ PASS | 115 vaults returned |
| 5 | Filter vaults by token | L2 | `vaults --chain 501 --token SOL --limit 5` | ✅ PASS | SOL vaults returned |
| 6 | View user positions (empty) | L2 | `positions --chain 501 --wallet DTEqFX...` | ✅ PASS | `[]` empty array |
| 7 | View user positions (after deposit) | L2 | `positions --chain 501 --wallet DTEqFX...` | ✅ PASS | Shows 0.954958 shares in vault GEodMs... |
| 8 | Preview deposit (dry-run) | L3 | `deposit --vault GEodMs... --amount 0.001 --dry-run` | ✅ PASS | `dry_run:true`, serialized_tx non-empty |
| 9 | Preview withdraw (dry-run) | L3 | `withdraw --vault GEodMs... --amount 1 --dry-run` | ✅ PASS | `dry_run:true`, serialized_tx non-empty |
| 10 | Deposit 0.001 SOL into SOL vault | L4 | `deposit --vault GEodMs... --amount 0.001 --chain 501` | ✅ PASS | `2U23FHhnskQwgSXTznUoNYGdxQ9butCYpw4oze2mvoKwuCTy6CvuPg9x1SmaUPU2GCFngNVtmzhiu4sbwA8YwFEZ` |
| 11 | Verify position after deposit | L4 | `positions --chain 501 --wallet DTEqFX...` | ✅ PASS | 0.954958 shares in GEodMs... |
| 12 | Withdraw 0.5 shares from SOL vault | L4 | `withdraw --vault GEodMs... --amount 0.5 --chain 501` | ✅ PASS | `SiyBvzURYrwHXvD8pendyM8BX6c2LenvQp5f6ffYDb7LKhVFbqtHERR3rvsYNNBVeWdX8wk3mVb1YJvtrZopLZj` |

## Fix Log

| # | Problem | Root Cause | Fix | File |
|---|---------|-----------|-----|------|
| 1 | positions returned empty fields | API uses `vaultAddress`/`stakedShares`/`unstakedShares`/`totalShares` — not `vault`/`sharesAmount`/`tokenAmount` | Updated field name mapping in positions.rs | `src/commands/positions.rs` |

## Block Explorer Links

- L4 Deposit: https://solscan.io/tx/2U23FHhnskQwgSXTznUoNYGdxQ9butCYpw4oze2mvoKwuCTy6CvuPg9x1SmaUPU2GCFngNVtmzhiu4sbwA8YwFEZ
- L4 Withdraw: https://solscan.io/tx/SiyBvzURYrwHXvD8pendyM8BX6c2LenvQp5f6ffYDb7LKhVFbqtHERR3rvsYNNBVeWdX8wk3mVb1YJvtrZopLZj
