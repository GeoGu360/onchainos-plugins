# Test Results Report — kamino-lend

- Date: 2026-04-05
- DApp supported chains: Solana only (501)
- Solana test chain: mainnet (501)
- Wallet: `DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE`
- Compilation: ✅
- Lint: ✅ (0 errors, 1 warning W100 base64 — expected)

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|-----------|---------|-------------|------------|--------|---------|
| 10    | 1         | 3       | 4           | 2          | 0      | 0       |

## Detailed Results

| # | Scenario (User View) | Level | Command | Result | TxHash / Notes |
|---|---------------------|-------|---------|--------|----------------|
| 1 | Build and lint | L1 | `cargo build --release && cargo clean && plugin-store lint .` | ✅ PASS | 0 errors |
| 2 | List Kamino markets with APY | L2 | `markets --name main` | ✅ PASS | USDC 9.51% supply APY, SOL 3.98% — 29 total markets |
| 3 | Check positions (empty wallet) | L2 | `positions` | ✅ PASS | Returns `has_positions: false` gracefully |
| 4 | Check positions for specific wallet | L2 | `positions --wallet DTEq...` | ✅ PASS | Same as above |
| 5 | Simulate USDC supply (dry-run) | L3 | `supply --token USDC --amount 0.01 --dry-run` | ✅ PASS | `dry_run: true`, no network call |
| 6 | Simulate USDC withdraw (dry-run) | L3 | `withdraw --token USDC --amount 0.01 --dry-run` | ✅ PASS | `dry_run: true`, no network call |
| 7 | Preview SOL borrow (dry-run) | L3 | `borrow --token SOL --amount 0.001 --dry-run` | ✅ PASS | Includes note about prior supply requirement |
| 8 | Preview SOL repay (dry-run) | L3 | `repay --token SOL --amount 0.001 --dry-run` | ✅ PASS | `dry_run: true` |
| 9 | Supply 0.001 SOL on-chain | L4 | `supply --token SOL --amount 0.001` | ✅ PASS | `ijx32Aa8dd1Z9wkXoFui2x7owtyB66R9XJMakfZpZvBbDdUrxwu7i3QHsgD2WngJoUNB9L4p2Jh6t7GxVQKzf8n` — [Solscan](https://solscan.io/tx/ijx32Aa8dd1Z9wkXoFui2x7owtyB66R9XJMakfZpZvBbDdUrxwu7i3QHsgD2WngJoUNB9L4p2Jh6t7GxVQKzf8n) slot 411164627 |
| 10 | Withdraw 0.001 SOL on-chain | L4 | `withdraw --token SOL --amount 0.001` | ✅ PASS | `65TtWHCYSyCiMc7cNKzCDXAo31bTqma2DLj1nKeXxWmj83pREgekuVcbsoPnXRcQskPwU9p6WY6dJqAaDGkbDv9Y` — [Solscan](https://solscan.io/tx/65TtWHCYSyCiMc7cNKzCDXAo31bTqma2DLj1nKeXxWmj83pREgekuVcbsoPnXRcQskPwU9p6WY6dJqAaDGkbDv9Y) slot 411164791 |

## Fixes Applied During Testing

| # | Issue | Root Cause | Fix | File |
|---|-------|-----------|-----|------|
| 1 | `markets` returned empty reserves | Metrics API requires `frequency=day` not `frequency=DAILY` | Changed to lowercase `day` | `src/api.rs` |

## Notes

- L4 supply of 0.001 SOL caused ~0.041 SOL total balance decrease due to Solana account creation rent for the Kamino obligation account (one-time cost, refundable on close)
- Borrow and repay tested via dry-run only (GUARDRAILS policy: liquidation risk)
- All txHashes confirmed on Solana mainnet via `getTransaction` RPC check
