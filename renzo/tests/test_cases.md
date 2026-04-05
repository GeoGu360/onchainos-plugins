# Test Cases — Renzo Plugin

**DApp:** Renzo Protocol  
**Chain:** Ethereum mainnet (chain 1)  
**Binary:** `renzo`  
**Date:** 2026-04-05

---

## Level 1 — Build + Lint

| # | Test | Expected |
|---|------|---------|
| L1-1 | `cargo build --release` | Exit 0, no errors |
| L1-2 | Manual lint: plugin.yaml api_calls are plain strings | E002: pass |
| L1-3 | Manual lint: SKILL.md write ops have confirmation text | E106: pass |
| L1-4 | .gitignore includes /target/ | E080/E130: pass |

---

## Level 2 — Read Operations (no gas, no wallet)

| # | Scenario | Command | Expected |
|---|----------|---------|---------|
| L2-1 | Get Renzo APR | `renzo get-apr` | JSON with `apr_percent` ~2.5% |
| L2-2 | Get Renzo TVL | `renzo get-tvl` | JSON with `total_tvl_eth` > 50000 |
| L2-3 | Check ezETH balance for test address | `renzo balance --address 0x87fb0647faabea33113eaf1d80d67acb1c491b90` | JSON with `ezETH.balance` field |
| L2-4 | Check balance for known ezETH holder | `renzo balance --address 0xd3Cc7c0E2e6F0F9aBBDcf83fC1e15c1DEE01CEB8` | Non-zero ezETH balance |

---

## Level 3 — Simulate (dry-run, verify calldata)

| # | Scenario | Command | Expected calldata prefix |
|---|----------|---------|-------------------------|
| L3-1 | Dry-run deposit ETH 0.00005 | `renzo deposit-eth --amount-eth 0.00005 --dry-run` | `0xf6326fb3` (depositETH selector) |
| L3-2 | Dry-run deposit stETH 0.00005 | `renzo deposit-steth --amount 0.00005 --dry-run` | Step1: `0x095ea7b3` (approve), Step2: `0x47e7ef24` (deposit) |
| L3-3 | Dry-run with explicit from address | `renzo deposit-eth --amount-eth 0.001 --from 0x87fb0647faabea33113eaf1d80d67acb1c491b90 --dry-run` | `0xf6326fb3`, amount_wei = 1000000000000000 |

---

## Level 4 — On-chain (needs lock, spends gas)

| # | Scenario | Command | Expected |
|---|----------|---------|---------|
| L4-1 | Deposit 0.00005 ETH into Renzo | `renzo deposit-eth --amount-eth 0.00005` | txHash on Etherscan |
| L4-2 | Check balance after deposit | `renzo balance` | Non-zero ezETH balance |
