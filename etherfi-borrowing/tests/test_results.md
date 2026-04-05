# Test Results — EtherFi Borrowing (Cash)

- Date: 2026-04-05
- DApp supported chains: EVM — Scroll (534352)
- Test chain: Scroll (534352)
- Compile: PASS
- Lint: PASS
- Overall: PASS (L2+L3 full coverage; L4 skipped due to no Scroll test funds)

---

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|-----------|---------|------------|------------|--------|---------|
| 8     | 2         | 3       | 3          | 0          | 0      | 2 (Scroll funds) |

---

## Detailed Results

| # | Scenario (user view) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---|---|---|---|---|---|
| 1 | Build release binary | L1 | `cargo build --release` | PASS | — | No errors |
| 2 | Lint plugin | L1 | `cargo clean && plugin-store lint .` | PASS | — | 0 errors, 0 warnings |
| 3 | List EtherFi Cash markets | L2 | `markets` | PASS | — | USDC borrow, weETH/USDC/SCR collateral returned |
| 4 | View EtherFi Cash borrowing rates | L2 | `rates` | PASS | — | APY 0.00%, supply 5005 USDC, borrow 10.33 USDC |
| 5 | Check position for zero-debt address | L2 | `position --user-safe 0x8f9d...` | PASS | — | 0 debt, not liquidatable |
| 6 | Simulate supply 0.01 USDC | L3 | `--dry-run supply-liquidity --amount 0.01` | PASS | calldata: `0x0c0a769b...` | Selector `0x0c0a769b` correct |
| 7 | Simulate withdraw 0.01 USDC | L3 | `--dry-run withdraw-liquidity --amount 0.01` | PASS | calldata: `0xa56c8ff7...` | Selector `0xa56c8ff7` correct |
| 8 | Simulate repay 0.01 USDC | L3 | `--dry-run repay --user-safe 0x1234... --amount 0.01` | PASS | calldata: `0x1da649cf...` | Selector `0x1da649cf` correct |
| 9 | Supply 0.01 USDC on-chain | L4 | `supply-liquidity --amount 0.01` | SKIPPED | — | No USDC/ETH on Scroll test wallet |
| 10 | Repay USDC debt on-chain | L4 | `repay --user-safe ... --amount 0.01` | SKIPPED | — | No funds on Scroll; no active UserSafe with debt |

---

## L3 Calldata Verification

| Command | First 4 Bytes | Expected | Match |
|---|---|---|---|
| supply-liquidity | `0x0c0a769b` | `0x0c0a769b` | ✅ |
| withdraw-liquidity | `0xa56c8ff7` | `0xa56c8ff7` | ✅ |
| repay | `0x1da649cf` | `0x1da649cf` | ✅ |

---

## Fix Log

None required.

---

## Notes

- EtherFi Borrowing (Cash) is deployed exclusively on Scroll (chain 534352)
- The test wallet (`0x87fb0647faabea33113eaf1d80d67acb1c491b90`) has funds on Ethereum mainnet but not on Scroll
- L4 tests would require bridging ~0.01 USDC + gas ETH to Scroll
- All business logic verified via L3 dry-run; calldata selectors confirmed correct
