# Curve Lending — Test Cases

## Level Summary

| Level | Count | Description |
|-------|-------|-------------|
| L1 | 2 | Compile + Lint |
| L2 | 5 | Read operations (no wallet, no gas) |
| L3 | 3 | Dry-run simulate (calldata verification) |
| L4 | 3 | Live on-chain read queries |

---

## L1 — Compile + Lint

| # | Test | Command | Expected |
|---|------|---------|---------|
| 1 | Debug compile | `cargo build` | Finished, 0 errors |
| 2 | Release + lint | `cargo build --release && cargo clean && plugin-store lint .` | Passes all checks |

---

## L2 — Read Operations

| # | Scenario | Command | Expected |
|---|---------|---------|---------|
| 1 | List all lending markets | `markets --chain 1 --limit 5` | JSON with 46 total markets, 5 shown |
| 2 | Get WETH-long rates by index | `rates --chain 1 --market 1` | borrow_apy ~1%, lend_apy ~0.37%, utilization ~35% |
| 3 | Check wallet with no loans | `positions --chain 1 --address 0x87fb... --market 1` | position_count: 0 |
| 4 | Check real loan holder | `positions --chain 1 --address 0x27b5491b... --market 1` | 5 WETH collateral, ~4504 crvUSD debt |
| 5 | Error: invalid market | `rates --chain 1 --market INVALIDXYZ` | Graceful error "Market not found" |

---

## L3 — Dry-run Simulate

| # | Scenario | Command | Expected Selector |
|---|---------|---------|---------|
| 1 | Deposit collateral dry-run | `deposit-collateral --market 1 --amount 0.001 --chain 1 --dry-run` | approve: 0x095ea7b3, create_loan: 0x23cfed03 |
| 2 | Borrow dry-run | `borrow --market 1 --amount 1.5 --collateral 0.001 --chain 1 --dry-run` | create_loan: 0x23cfed03 |
| 3 | Repay dry-run | `repay --market 1 --amount 4500 --chain 1 --dry-run` | approve: 0x095ea7b3, repay: 0x371fd8e6 |

---

## L4 — Live On-chain (Read Only)

Per GUARDRAILS: no crvUSD in test wallet → borrow/repay are dry-run only.
No WETH → deposit-collateral L4 skipped (would need ETH wrap first).
L4 covers live on-chain read queries that touch the real blockchain state.

| # | Scenario | Command | Expected |
|---|---------|---------|---------|
| 1 | Live market list | `markets --chain 1 --limit 5` | ok:true, 46 markets, real TVL values |
| 2 | Live WETH rates | `rates --chain 1 --market 1` | Real borrow APY from MonetaryPolicy |
| 3 | Live active position | `positions --chain 1 --address 0x27b5... --market 1` | Real debt 4504 crvUSD, health 4.15% |
