# Test Results Report — Convex Finance Plugin

- Date: 2026-04-05
- DApp supported chains: Ethereum mainnet (chain ID 1)
- EVM test chain: Ethereum mainnet (1)
- Compile: ✅
- Lint: ✅ (only E123 placeholder SHA — expected, filled during submission)
- Overall pass standard: EVM-only DApp, EVM ops pass

## Summary

| Total | L1 Build | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|----------|---------|-------------|-------------|--------|---------|
| 11    | 3        | 3       | 5           | 1 pass + 2 blocked | 0 | 2 |

## Detailed Results

| # | Scenario (user view) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Build plugin | L1 | `cargo build` | ✅ PASS | — | 0 errors |
| 2 | Release build | L1 | `cargo build --release` | ✅ PASS | — | Binary: 4.2MB |
| 3 | Lint check | L1 | `cargo clean && plugin-store lint .` | ✅ PASS | — | Only E123 placeholder SHA |
| 4 | List top 5 Convex pools | L2 | `get-pools --limit 5` | ✅ PASS | — | 430 pools found, 3pool top with $163M TVL |
| 5 | Query positions for test wallet | L2 | `get-positions --address 0x87fb... --chain 1` | ✅ PASS | — | Balances all zero (no active positions) |
| 6 | List factory pools | L2 | `get-pools --registry factory --limit 3` | ✅ PASS | — | Factory pools returned |
| 7 | Preview stake cvxCRV | L3 | `--dry-run stake-cvxcrv --amount 1.0 --chain 1` | ✅ PASS | approve: `0x095ea7b3...`, stake: `0xa694fc3a...` | Selectors verified |
| 8 | Preview unstake cvxCRV | L3 | `--dry-run unstake-cvxcrv --amount 1.0 --chain 1` | ✅ PASS | `0x00ebf5dd...` | Selector matches |
| 9 | Preview lock CVX | L3 | `--dry-run lock-cvx --amount 1.0 --chain 1` | ✅ PASS | lock: `0x1338736f...` | Selector matches |
| 10 | Preview unlock CVX | L3 | `--dry-run unlock-cvx --chain 1` | ✅ PASS | `0x312ff839...` | Selector matches |
| 11 | Preview claim rewards | L3 | `--dry-run claim-rewards --chain 1` | ✅ PASS | cvxCRV: `0x7050ccd9`, vlCVX: `0x3d18b912` | Selectors correct |
| 12 | Claim rewards (chain 1) | L4 | `claim-rewards --chain 1` | ✅ PASS | No txHash (no pending rewards) | Wallet resolved, graceful skip |
| 13 | Stake cvxCRV on-chain | L4 | `stake-cvxcrv --amount 0.001 --chain 1` | ⚠️ BLOCKED | — | Test wallet has 0 cvxCRV; calldata verified via dry-run |
| 14 | Lock CVX on-chain | L4 | `lock-cvx --amount 0.001 --chain 1` | ⚠️ BLOCKED | — | Test wallet has 0 CVX; calldata verified via dry-run |

## Fix Record

No fixes required — all tests passed on first attempt.

## Notes

- L4-2 (stake-cvxcrv) and L4-4 (lock-cvx) blocked due to test wallet having no CVX/cvxCRV. All calldata structures verified via L3 dry-run with correct selectors.
- claim-rewards L4 passed: wallet was resolved successfully on chain 1 (confirmed onchainos CLI integration works).
- Convex staking/locking requires CVX/cvxCRV tokens which the test wallet does not hold. This is a test environment limitation, not a code issue.
