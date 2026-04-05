# Test Cases — EtherFi Borrowing (Cash)

- DApp: EtherFi Borrowing (Cash)
- Chain: Scroll (534352) — all operations
- Generated: 2026-04-05

---

## L1 — Compile + Lint

| # | Test | Command | Expected |
|---|---|---|---|
| 1 | Build release binary | `cargo build --release` | Finished without errors |
| 2 | Lint | `cargo clean && plugin-store lint .` | 0 errors, 0 warnings |

---

## L2 — Read Operations (no wallet, no gas)

| # | Scenario | Command | Expected |
|---|---|---|---|
| 1 | List borrow/collateral markets | `markets` | JSON with USDC borrow market, weETH/USDC/SCR collateral |
| 2 | Get borrow rates | `rates` | JSON with borrow_apy_pct, total_supply, total_borrow |
| 3 | Get position for zero-debt address | `position --user-safe <ADDR>` | JSON with 0 debt, not liquidatable |

---

## L3 — Dry-run Simulations (calldata verification)

| # | Scenario | Command | Expected Selector |
|---|---|---|---|
| 1 | Supply USDC dry-run | `--dry-run supply-liquidity --amount 0.01` | `0x0c0a769b` (supply) |
| 2 | Withdraw USDC dry-run | `--dry-run withdraw-liquidity --amount 0.01` | `0xa56c8ff7` (withdrawBorrowToken) |
| 3 | Repay USDC dry-run | `--dry-run repay --user-safe <ADDR> --amount 0.01` | `0x1da649cf` (repay) |

---

## L4 — On-chain Tests

| # | Scenario | Status | Notes |
|---|---|---|---|
| 1 | Supply 0.01 USDC liquidity | SKIPPED | No USDC on Scroll test wallet; would require bridging ETH+USDC to Scroll |
| 2 | Repay debt | SKIPPED | No funds on Scroll; also requires an active UserSafe with debt |

**L4 skip rationale:** EtherFi Borrowing/Cash is deployed exclusively on Scroll. The test wallet has no balance on Scroll (ETH or USDC). Bridging from Ethereum mainnet to Scroll is outside scope of current test budget. All calldata correctness verified via L3 dry-run.
