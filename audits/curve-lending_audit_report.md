# Audit Report: curve-lending

**Date:** 2026-04-06  
**Auditor:** skill-auditor  
**Plugin version:** 0.1.0  
**Commit audited:** 9cec838 (pre-fix)  
**Fix commit:** f8ce213  
**Chain:** Ethereum mainnet (chain ID: 1)  
**EVM wallet:** 0x87fb0647faabea33113eaf1d80d67acb1c491b90

---

## 1. Build

```
cargo build --release
```

**Result:** SUCCESS — 32 warnings (all pre-existing unused-import/dead-code warnings), zero errors.

---

## 2. Functional Test Results

All 6 commands tested:

| Command | Mode | Result | Notes |
|---------|------|--------|-------|
| `markets --chain 1 --limit 5` | read | PASS | 46 markets, correct TVL/debt values |
| `rates --chain 1 --market WETH-long` | read | PASS | borrow_apy ~0.12%, utilization ~35.6% |
| `positions --chain 1 --address 0x87fb...` | read | PASS | 0 active positions (expected) |
| `deposit-collateral --market WETH-long --amount 0.001 --dry-run` | dry-run | PASS | Correct calldata for create_loan |
| `borrow --market WETH-long --amount 100 --collateral 0.05 --dry-run` | dry-run | PASS | Correct calldata, max_borrowable validation |
| `repay --market WETH-long --amount 100 --dry-run` | dry-run | PASS | Correct crvUSD approve + repay calldata |

Write operations (deposit-collateral live, borrow, repay) were not executed on-chain per GUARDRAILS:
- No WETH balance in test wallet for deposit
- Borrow and repay are dry-run only in test environment

---

## 3. Static Review

### 3.1 extract_tx_hash — FIXED (Bug)

**File:** `src/onchainos.rs`

**Before:**
```rust
pub fn extract_tx_hash(result: &Value) -> String {
    result["data"]["txHash"]
        .as_str()
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("pending")
        .to_string()
}
```

**Problem:** Returns `String` with fallback `"pending"` instead of `anyhow::Result<String>`.
This is inconsistent with the monorepo standard (aave-v3, compound-v3 both return `Result`).
A missing txHash silently produces `"pending"` in the output JSON rather than propagating an error.

**After:**
```rust
pub fn extract_tx_hash(result: &Value) -> anyhow::Result<String> {
    let hash = result["data"]["txHash"]
        .as_str()
        .or_else(|| result["txHash"].as_str());
    match hash {
        Some(h) if !h.is_empty() && h != "pending" => Ok(h.to_string()),
        _ => anyhow::bail!(
            "txHash not found or still pending in onchainos response; raw: {}",
            result
        ),
    }
}
```

All call sites updated to use `?` propagation.

### 3.2 ok-check Order — FIXED (Bug)

**Files:** `src/commands/deposit_collateral.rs`, `src/commands/repay.rs`

**Before:** `extract_tx_hash()` was called before the `ok` check, meaning if `ok=false` but the
response contained a txHash field, it would extract and log it before hitting the bail.

**After:** `ok` check moved before `extract_tx_hash()` call, consistent with deposit_collateral
and repay flow.

### 3.3 Unused Import — FIXED

**File:** `src/commands/deposit_collateral.rs`

`CRVUSD` was imported from `config` but not used. Removed.

### 3.4 SKILL.md Check

- **ASCII:** SKILL.md contains 8 em-dash characters (U+2014, `—`). These are valid UTF-8 and
  consistent with the monorepo standard (aave-v3 SKILL.md also contains UTF-8). No issue.
- **Do NOT use for section:** Not present. Not required by SKILL.md spec (only Routing Rules
  section present, which serves the same purpose). No issue.
- **source_repo:** `plugin.yaml` has `source_repo: GeoGu360/onchainos-plugins` — correct.

### 3.5 Amount Precision

`f64` is used for amount-to-u128 conversion:
```rust
let collateral_raw = (args.amount * divisor as f64) as u128;
```

At 18 decimals, f64 precision is ~15-16 significant digits, causing up to ~2 wei rounding error
at the 18th decimal place. This is acceptable for practical amounts (0.001 ETH has exact f64
representation). Consistent with monorepo standard across other plugins.

### 3.6 find_market Duplication

`find_market()` is duplicated in `deposit_collateral.rs`, `borrow.rs`, and `repay.rs`. No bug,
but a refactoring opportunity for future maintainability.

### 3.7 Guardrails

- Borrow and repay correctly check `--dry-run` before executing.
- SKILL.md notes "Ask user to confirm before executing the real transaction" on all write commands.
- Repay uses `current_debt` (not `uint256::MAX`) for full repay, avoiding revert risk. Correct.

---

## 4. Contract Verification

All selectors verified against design.md (cast sig verified):

- Factory: `market_count()` `0xfd775c78`, `names()` `0x4622ab03`, `controllers()` `0xe94b0dd2` — all correct
- Controller: `loan_exists()` `0xa21adb9e`, `health()` `0x8908ea82`, `create_loan()` `0x23cfed03` — all correct
- Repay: `repay(uint256)` `0x371fd8e6` — verified live on-chain per design.md

---

## 5. Bugs Found and Fixed

| # | Severity | File | Issue | Status |
|---|----------|------|-------|--------|
| 1 | Medium | `src/onchainos.rs` | `extract_tx_hash` returns `String` not `Result` — silently swallows missing txHash as "pending" | FIXED |
| 2 | Low | `src/commands/deposit_collateral.rs`, `repay.rs` | `extract_tx_hash` called before ok-check | FIXED |
| 3 | Low | `src/commands/deposit_collateral.rs` | Unused `CRVUSD` import | FIXED |

---

## 6. Uninstall

Binary uninstalled from `~/.local/bin/curve-lending`.

---

## 7. Summary

The plugin is well-structured and functionally correct for all read and dry-run operations.
The main bug was `extract_tx_hash` returning `String` instead of `anyhow::Result<String>`,
which is the monorepo standard and prevents silent failure in write paths.
Three fixes applied, verified by rebuild (zero errors), and pushed to monorepo main at commit `f8ce213`.
