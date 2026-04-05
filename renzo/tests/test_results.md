# Test Results Report — Renzo Plugin

- **Date:** 2026-04-05
- **DApp supported chains:** EVM only (Ethereum mainnet, chain 1)
- **EVM test chain:** Ethereum mainnet (chain 1)
- **Binary:** `renzo v0.1.0`
- **Compile:** ✅ PASS
- **Lint:** ✅ PASS (manual — plugin-store binary not installed)
- **Overall pass standard:** EVM DApp → EVM all pass

---

## Summary

| Total | L1 Build | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|----------|---------|-------------|-------------|--------|---------|
| 11    | 4        | 4       | 3           | 1 (deposit-eth + balance check) | 0      | 0       |

---

## Detailed Results

| # | Scenario (User View) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Build release binary | L1 | `cargo build --release` | ✅ PASS | — | 0 errors, 0 warnings after dead_code suppression |
| 2 | Manual lint: api_calls strings | L1 | Check plugin.yaml | ✅ PASS | — | E002: api_calls are plain strings |
| 3 | Manual lint: write ops confirmation | L1 | Check SKILL.md | ✅ PASS | — | E106: all 4 wallet contract-call references have "confirm" nearby |
| 4 | .gitignore targets | L1 | Check .gitignore | ✅ PASS | — | /target/ present |
| 5 | Get Renzo restaking APR | L2 | `renzo get-apr` | ✅ PASS | — | Returns APR 2.5208%, correct JSON structure |
| 6 | Get Renzo TVL | L2 | `renzo get-tvl` | ✅ PASS | — | Returns totalTVL=55377 ETH, ezETH supply=185667 |
| 7 | Check ezETH balance for test address | L2 | `renzo balance --address 0x87fb...` | ✅ PASS | — | Returns balance_wei correctly, 0 before deposit |
| 8 | Check balance on address with stETH | L2 | `renzo balance --address 0x40ec5B...` | ✅ PASS | — | Shows stETH balance ~5.88 ETH, 0 ezETH |
| 9 | Dry-run deposit ETH | L3 | `renzo deposit-eth --amount-eth 0.00005 --dry-run` | ✅ PASS | `0xf6326fb3` | depositETH() selector correct; amount_wei=50000000000000 |
| 10 | Dry-run deposit stETH (two-step) | L3 | `renzo deposit-steth --amount 0.00005 --dry-run` | ✅ PASS | Step1: `0x095ea7b3` (approve), Step2: `0x47e7ef24` (deposit) | Both selectors correct; RestakeManager padded correctly |
| 11 | Dry-run with explicit from address | L3 | `renzo deposit-eth --amount-eth 0.001 --from 0x87fb... --dry-run` | ✅ PASS | `0xf6326fb3` | amount_wei=1000000000000000 |
| 12 | Deposit 0.00005 ETH into Renzo on Ethereum | L4 | `renzo deposit-eth --amount-eth 0.00005` | ✅ PASS | `0xe30d768e857581659a319901ea511421424279f8cafc922874c6bc71fcb26e06` | Block 24812276, received 46486502653881 wei ezETH (~0.0000465 ezETH) |
| 13 | Check ezETH balance after deposit | L4 | `renzo balance` | ✅ PASS | — | ezETH balance: 0.000046486502653881 (confirmed on-chain) |

---

## L4 Transaction

**Etherscan:** https://etherscan.io/tx/0xe30d768e857581659a319901ea511421424279f8cafc922874c6bc71fcb26e06

- **Block:** 24812276
- **From:** 0x87fb0647faabea33113eaf1d80d67acb1c491b90
- **To:** RestakeManager `0x74a09653A083691711cF8215a6ab074BB4e99ef5`
- **Value:** 0.00005 ETH (50000000000000 wei)
- **Calldata:** `0xf6326fb3` (depositETH())
- **Status:** ✅ SUCCESS
- **ezETH minted:** 46,486,502,653,881 wei (~0.0000465 ezETH) to `0x87fb...`

---

## Fix Log

| # | Problem | Root Cause | Fix | File |
|---|---------|-----------|-----|------|
| 1 | `renzo get-apr` DNS error | `api.renzoprotocol.com` not resolvable from sandbox | Changed API_BASE_URL to `https://app.renzoprotocol.com/api` (verified reachable) | `src/config.rs` |
| 2 | `renzo balance` showed 0.0 for tiny ezETH | Initial response for pre-deposit state was correct 0; post-deposit shows correct value | No fix needed — balance correctly showed 0.000046 ezETH after block confirmed | — |
