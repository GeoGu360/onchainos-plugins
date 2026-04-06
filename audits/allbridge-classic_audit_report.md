# Allbridge Classic Plugin Audit Report

**Date:** 2026-04-06
**Auditor:** skill-auditor (Claude Sonnet 4.6)
**Plugin:** allbridge-classic v0.1.0
**Repo:** /tmp/onchainos-plugins/allbridge-classic
**Commit after fixes:** 6db11fd

---

## Summary

| Item | Result |
|------|--------|
| Build | PASS (cargo build --release, 0 errors) |
| Read tests (get-tokens, check-address) | PASS |
| Dry-run write tests (bridge --dry-run) | PASS |
| On-chain tests | SKIPPED (per test plan: bridge L4 is dry-run only due to cross-chain cost) |
| Static analysis | 3 bugs found, all fixed |
| Bugs fixed + pushed | YES (commit 6db11fd -> main) |

---

## Test Results

### L2: Read Operations

| Test | Command | Result |
|------|---------|--------|
| L2-1 get-tokens | `get-tokens` | PASS - returns chains BSC/ETH/SOL with token list |
| L2-2 check-address SOL | `check-address --chain SOL --address DTEqFXy...` | PASS - {"result":true,"status":"OK"} |
| L2-3 check-address ETH | `check-address --chain ETH --address 0x87fb...` | PASS - {"result":true,"status":"OK"} |
| L2-4 get-tx-status | `get-tx-status --lock-id 199936...` | PASS - returns clear 404 error message |

### L3: Dry-run Tests

| Test | Command | Result |
|------|---------|--------|
| L3-1 ETH->BSC 0.01 USDT | `bridge --chain 1 --token USDT --amount 0.01 --dest-chain BSC --dry-run` | PASS - calldata starts with 0x7bacc91e, amount=10000 |
| L3-2 ETH->SOL 0.01 USDT | `bridge --chain 1 --token USDT --amount 0.01 --dest-chain SOL --dry-run` | PASS - calldata correct, Solana address encoded as 32 bytes |
| L3-precision ETH->SOL 2.01 USDT | `bridge --chain 1 --token USDT --amount 2.01 --dry-run` | PASS after fix (was 2009999, now 2010000) |

---

## Bugs Found and Fixed

### BUG-1: f64 amount precision (CRITICAL)

**File:** `src/commands/bridge.rs` line 173

**Problem:** Amount was calculated as:
```rust
let amount_raw = (args.amount * 10f64.powi(decimals as i32)) as u128;
```
The `as u128` cast truncates (floor) the f64 result. Due to IEEE-754 representation, many decimal values produce off-by-one errors. For example:
- `2.01 * 1e6 = 2009999.9999...` truncates to `2009999` instead of `2010000`
- ~20% of cent-precision amounts (0.01-99.99) are affected

This causes users to bridge 1 unit less than intended (1 sub-unit = 0.000001 USDT), which may cause silent loss or bridge rejection.

**Fix:** Added `.round()` before the cast:
```rust
let amount_raw = (args.amount * 10f64.powi(decimals as i32)).round() as u128;
```

**Verified:** 2.01 USDT now encodes as 2010000 correctly.

---

### BUG-2: extract_tx_hash returns String, does not check ok field (MODERATE)

**File:** `src/onchainos.rs` line 69

**Problem:** `extract_tx_hash` returned a plain `String` with fallback `"pending"` instead of `Result`. This meant:
- If onchainos returned `{"ok":false, "error":"insufficient balance"}`, the function would return `"pending"` and bridge would report success
- Callers in bridge.rs used `== "pending"` as a heuristic which only catches missing txHash, not explicit failure

**Fix:** Changed signature to `Result<String>`. Now checks `ok` field first; returns `Err` if `ok == false`. Callers use `?` for live transactions and `unwrap_or` for dry-run.

---

### BUG-3: source_commit is all-zeros placeholder (MINOR)

**File:** `plugin.yaml` line 24

**Problem:** `source_commit: "0000000000000000000000000000000000000000"` - placeholder was never updated to a real commit hash.

**Fix:** Updated to actual HEAD commit: `c99f51fe67f39fb1655263fe4461e2b8a97b0599`

---

## Static Analysis: Other Checks

| Check | Status | Notes |
|-------|--------|-------|
| SKILL.md ASCII-only | PASS | No non-ASCII characters found |
| SKILL.md "Do NOT use for" section | N/A | Not required in this plugin's SKILL.md format |
| extract_tx_hash returns Result | FIXED | Was String, now Result<String> |
| ok-check on onchainos responses | FIXED | Now checked in extract_tx_hash |
| source_repo correct | PASS | GeoGu360/onchainos-plugins matches repo |
| source_commit correct | FIXED | Was all-zeros, updated to real commit |
| amount precision | FIXED | f64 .round() added |
| ABI encoding correctness | PASS | lock() selector 0x7bacc91e verified; params in correct order; dest bytes4 left-aligned; EVM recipient right-padded as per Allbridge protocol |
| Solana address encoding | PASS | base58 -> 32 bytes hex correct |
| ERC-20 approve calldata | PASS | 0x095ea7b3 selector, spender left-padded, amount correct |
| API error handling | PASS | 404 for unconfirmed tx properly surfaced |

---

## Architecture Notes

- Read ops (get-tokens, check-address, get-tx-status) call Allbridge REST API at allbridgeapi.net - no wallet needed
- Write op (bridge) requires EVM source chain; generates approve + lock calldatas; executes via `onchainos wallet contract-call`
- Solana as destination is supported (Solana as source is not - by design per design.md)
- Bridge contract address is identical on all EVM chains (0xBBbD1BbB4f9b936C3604906D7592A644071dE884)
- Note: Allbridge Classic is in maintenance mode, planned shutdown mid-2026

---

## Commit

All fixes committed and pushed to monorepo main:
```
commit 6db11fd
fix(allbridge-classic): fix f64 amount precision, extract_tx_hash Result, source_commit
```
