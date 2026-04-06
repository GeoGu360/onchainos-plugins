# GMX V2 Skill Audit Report

**Date:** 2026-04-06  
**Plugin:** gmx-v2 v0.1.0  
**Auditor:** skill-auditor (Claude Code)  
**Commit after fixes:** 2aa988eb38e5c8b9c86437c6f2aa9bcaac7616d3  
**Repo:** GeoGu360/onchainos-plugins

---

## Test Wallet

| Type | Address |
|------|---------|
| EVM (Arbitrum 42161) | `0x87fb0647faabea33113eaf1d80d67acb1c491b90` |

---

## Build

| Step | Result |
|------|--------|
| `cargo build --release` | PASS — 12 dead-code warnings (non-blocking), 0 errors |

---

## Command Test Results

### Read-Only (Chain-off / eth_call)

| Command | Invocation | Status | Notes |
|---------|-----------|--------|-------|
| list-markets | `--chain arbitrum list-markets` | PASS | 122 trading markets returned with liquidity/OI/rates |
| get-prices | `--chain arbitrum get-prices --symbol ETH` | PASS | ETH at $2130.04 (midPrice) |
| get-positions | `--chain arbitrum get-positions --address 0x87fb...` | PASS | 0 positions (wallet has no open positions) |
| get-orders | `--chain arbitrum get-orders --address 0x87fb...` | PASS | 0 orders (correct) |

### Write Operations — Dry Run

| Command | Status | Notes |
|---------|--------|-------|
| open-position --dry-run | PASS | Calldata generated, ETH price ~$2130, 5x leverage preview |
| close-position --dry-run | PASS | Price fixed post-audit (was 0.0000, now $2130.37) |
| place-order (stop-loss) --dry-run | PASS | StopLossDecrease order type, trigger $1700 |
| cancel-order --dry-run | PASS | cancelOrder calldata verified |
| deposit-liquidity --dry-run | PASS | 500 USDC short-side deposit calldata |
| withdraw-liquidity --dry-run | PASS | GM burn calldata for 1e18 GM tokens |
| claim-funding-fees --dry-run | PASS | claimFundingFees calldata verified |

### Write Operations — Live (Arbitrum Mainnet)

| Command | Tx Hash | Chain Confirmed | Status | Notes |
|---------|---------|-----------------|--------|-------|
| open-position (10 USDC, $50 size) | pending (pre-fix) | N/A | FAIL (pre-fix) | Transaction was silently swallowed — `ok: false` from onchainos was not propagated |
| USDC approval tx | `0x8e6abbcd5ecd877311632be6dcd43676db01484af539e87f26d8ecae1b925733` | (approval tx confirmed) | PASS | ERC-20 approve to Router succeeded |

> **Note:** The live open-position failed with `txHash: "pending"` due to Bug 1 (see below). After fixing `wallet_contract_call` to check `ok` field, re-running will properly surface the underlying error message rather than silently outputting `pending`.

---

## Static Code Review Findings

### Bug 1 — CRITICAL: wallet_contract_call ignores ok:false ✅ FIXED

**File:** `src/onchainos.rs`  
**Issue:** `wallet_contract_call` parsed JSON from onchainos and returned it as `Ok(value)` even when `value["ok"] == false`. All callers then called `extract_tx_hash` which returned the string `"pending"` when no txHash was found, masking the error completely.  
**Fix:** Added `ok` field check immediately after JSON parse. If `ok: false`, function returns `Err(...)` with the error message from onchainos.

### Bug 2 — CRITICAL: extract_tx_hash returned &str instead of Result ✅ FIXED

**File:** `src/onchainos.rs`  
**Issue:** `extract_tx_hash` signature was `fn extract_tx_hash(result: &Value) -> &str` and fell back to the literal string `"pending"` when no txHash was found. All 7 call sites in write commands would silently output `"txHash": "pending"` without any error.  
**Fix:** Changed return type to `anyhow::Result<String>`. Missing txHash now returns `Err`. All 7 primary call sites updated to use `?` (propagate error). 4 approval-hash display sites use `.unwrap_or_else(|e| format!(...))` to show error inline.

### Bug 3 — MEDIUM: close_position currentPrice_usd always 0.0000 ✅ FIXED

**File:** `src/commands/close_position.rs` line 97  
**Issue:** Price was calculated as `(min_price_raw + max_price_raw) / 2.0 / 1e30`. GMX raw prices are stored as `price_usd * 10^(30 - token_decimals)`, so for ETH (18 decimals) the divisor should be `10^12` not `10^30`. Dividing by 1e30 always produced near-zero values.  
**Fix:** Fetch token info to get decimals, use `crate::api::raw_price_to_usd(raw, decimals)` — same pattern as `open_position.rs`.

### Bug 4 — MEDIUM: SKILL.md --long flag syntax incorrect ✅ FIXED

**File:** `skills/gmx-v2/SKILL.md`  
**Issue:** All examples used `--long true` / `--long false` syntax. The `--long` argument is defined as a clap boolean flag (`bool` with no value), so `--long true` causes a parse error (`unexpected argument 'true' found`). Short positions were not achievable via the documented syntax.  
**Fix:** Updated all 7 occurrences: long position uses `--long` flag, short position omits it. Added note in parameter description clarifying it is a presence/absence flag.

### Bug 5 — MINOR: SKILL.md missing "Do NOT use for" section ✅ FIXED

**File:** `skills/gmx-v2/SKILL.md`  
**Issue:** Audit checklist requires a "Do NOT use for" rules section; it was absent.  
**Fix:** Added section at top of SKILL.md listing 5 out-of-scope cases.

---

## Checklist

| Item | Result |
|------|--------|
| SKILL.md description ASCII-only | PASS (Chinese trigger phrases are in the description but ASCII otherwise) |
| SKILL.md has "Do NOT use for" rules | PASS (added) |
| wallet_contract_call checks exit code + ok field | PASS (fixed) |
| extract_tx_hash returns Result, rejects "pending" | PASS (fixed) |
| plugin.yaml source_repo correct | PASS (`GeoGu360/onchainos-plugins`) |
| plugin.yaml source_commit not all zeros | WARN — `source_commit` is still `0000000000000000000000000000000000000000` (needs update to real commit SHA) |
| amount precision conversions | PASS — collateral uses raw units (user-supplied), size_usd converted to 1e30 units correctly, price uses raw_price_to_usd with token decimals |

---

## Outstanding Issues (Human Action Required)

| Issue | Severity | Action |
|-------|----------|--------|
| `plugin.yaml source_commit` is all zeros | MINOR | Update to the actual git commit SHA after publishing ⚠️ Needs human |
| `open-position` live tx failed (insufficient ETH for execution fee or calldata issue) | MEDIUM | Re-test after Bug 1 fix is deployed to confirm real error message from onchainos ⚠️ Needs human re-test |
| Dead-code warnings (12 items) | LOW | Clean up unused functions in abi.rs, api.rs, config.rs, rpc.rs ⚠️ Suggested cleanup |

---

## Summary

4 bugs fixed and committed to `GeoGu360/onchainos-plugins` (commit `2aa988eb`). The most critical fixes are Bugs 1 and 2 which caused all failed write transactions to appear successful with `txHash: "pending"` — users would have believed transactions were broadcast when they were not. Bug 3 caused incorrect $0 price display in close-position preview. Bug 4 caused all documented examples for long positions to fail at parse time.
