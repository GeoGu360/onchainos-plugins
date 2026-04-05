# Test Results Report

- Date: 2026-04-05
- DApp: Instadapp
- DApp supported chains: EVM only (Ethereum mainnet, chain ID 1)
- EVM test chain: Ethereum (1) — Instadapp Lite vaults are Ethereum-only
- Compile: ✅
- Lint: ✅ (after source_commit set in Phase 4)
- Overall pass standard: EVM DApp — EVM all pass

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|------------|---------|------------|------------|--------|---------|
| 10    | 2          | 3       | 4          | 3 (deposit+positions+withdraw) | 0 | 0 |

## Detailed Results

| # | Scenario (User View) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Build instadapp binary | L1 | `cargo build --release` | ✅ PASS | — | 4 dead-code warnings, 0 errors |
| 2 | Lint plugin structure | L1 | `cargo clean && plugin-store lint .` | ✅ PASS | — | E123 fixed after monorepo push |
| 3 | List Instadapp Lite vaults | L2 | `instadapp --chain 1 vaults` | ✅ PASS | — | 2 vaults: iETH (1.2x) and iETHv2 (1.2x) |
| 4 | Check yield rates | L2 | `instadapp --chain 1 rates` | ✅ PASS | — | iETH: 20.05% cumulative, iETHv2: 20.58% |
| 5 | Check positions (no position) | L2 | `instadapp --chain 1 positions --wallet 0x87fb...` | ✅ PASS | — | position_count: 0 |
| 6 | Simulate deposit 0.00005 ETH into iETH v1 | L3 | `instadapp --chain 1 --dry-run deposit --vault v1 --amount 0.00005` | ✅ PASS | calldata: 0x87ee9312... | selector 0x87ee9312 correct |
| 7 | Simulate deposit stETH into iETHv2 | L3 | `instadapp --chain 1 --dry-run deposit --vault v2 --amount 0.001` | ✅ PASS | approve: 0x095ea7b3, deposit: 0x6e553f65 | 2-step flow correct |
| 8 | Simulate withdraw from iETH v1 | L3 | `instadapp --chain 1 --dry-run withdraw --vault v1 --shares 0.001` | ✅ PASS | calldata: 0x00f714ce... | selector correct |
| 9 | Simulate redeem from iETHv2 | L3 | `instadapp --chain 1 --dry-run withdraw --vault v2 --shares 0.001` | ✅ PASS | calldata: 0xba087652... | selector correct |
| 10 | Deposit 0.00005 ETH into iETH v1 vault | L4 | `instadapp --chain 1 deposit --vault v1 --amount 0.00005` | ✅ PASS | [0xe972a01bf58f2c00239232b3b2d7440f886344a192f694c49ed4a14f472f577f](https://etherscan.io/tx/0xe972a01bf58f2c00239232b3b2d7440f886344a192f694c49ed4a14f472f577f) | Received 0.000042 iETH shares |
| 11 | Check iETH positions after deposit | L4 | `instadapp --chain 1 positions` | ✅ PASS | — | position_count: 1, shares: 0.000042 iETH |
| 12 | Withdraw all iETH shares from v1 | L4 | `instadapp --chain 1 withdraw --vault v1` | ✅ PASS | [0xc45a409a28bdc6011c74650acd229ecefb5c4784283752d86c793019c36c2f38](https://etherscan.io/tx/0xc45a409a28bdc6011c74650acd229ecefb5c4784283752d86c793019c36c2f38) | Redeemed all iETH shares |

## Fix Log

| # | Issue | Root Cause | Fix | File |
|---|-------|-----------|-----|------|
| 1 | `--chain` flag rejected after subcommand | Clap global flags must precede subcommand | Use `instadapp --chain 1 vaults` (flag before subcommand) | N/A (user error) |

## ETH Spend Summary

| Test | ETH Spent | Notes |
|------|-----------|-------|
| L4 deposit | 0.00005 ETH | Below GUARDRAILS limit |
| L4 withdraw | ~0.0001 ETH gas | Gas only |
| Total | ~0.00015 ETH | Within budget |
