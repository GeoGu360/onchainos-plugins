# Audit Report: etherfi-borrowing

**Auditor:** skill-auditor (Claude Sonnet 4.6)
**Date:** 2026-04-06
**Plugin path:** /tmp/onchainos-plugins/etherfi-borrowing
**Commit after fixes:** cb05a66

---

## Summary

The `etherfi-borrowing` plugin is a Rust CLI skill for the EtherFi Cash borrowing protocol on Scroll (chain ID 534352). It provides 6 commands: `markets`, `rates`, `position`, `supply-liquidity`, `withdraw-liquidity`, and `repay`.

**Overall verdict: PASS (after fixes)**

---

## Build

- `cargo build --release` succeeded with 8 warnings (unused variables/constants — non-critical, pre-existing dead code).
- Binary: `target/release/etherfi-borrowing`

---

## Command Test Results

| Command | Result | Notes |
|---|---|---|
| `markets` | PASS | Live RPC call to Scroll; returned borrow + collateral markets |
| `rates` | PASS | Live RPC call; APY/utilization data returned correctly |
| `position --user-safe <EOA>` | PASS | Returns zeroed position for non-UserSafe address as expected |
| `--dry-run supply-liquidity --amount 0.01` | PASS | Dry-run works; calldata and wallet=0x00 as expected |
| `--dry-run withdraw-liquidity --amount 0.01` | PASS | Dry-run works correctly |
| `--dry-run repay --user-safe <addr> --amount 0.01` | PASS | Dry-run works correctly |
| `supply-liquidity --dry-run ...` (flag after subcommand) | FAIL (expected) | clap correctly rejects; flag must be global |

---

## Bugs Found and Fixed

### Bug 1: Missing ok-check on wallet_contract_call result (CRITICAL)
**Files:** `src/commands/supply.rs`, `src/commands/withdraw.rs`, `src/commands/repay.rs`

**Problem:** After calling `wallet_contract_call()` and `erc20_approve()`, the code did not check whether the returned JSON had `ok=false`. A failed on-chain transaction would still result in the plugin printing `"ok": true` with a null/dummy tx_hash.

**Fix:** Added guard block after each `wallet_contract_call` and `erc20_approve` invocation:
```rust
if result["ok"].as_bool() == Some(false) {
    let err = result["error"].as_str().unwrap_or("unknown error");
    anyhow::bail!("... transaction failed: {}", err);
}
```

### Bug 2: Non-ASCII em-dashes in SKILL.md (MEDIUM)
**File:** `skills/etherfi-borrowing/SKILL.md`

**Problem:** Parameter descriptions used Unicode em-dashes (U+2014, `—`) in 8 places. SKILL.md must be pure ASCII.

**Fix:** Replaced all `—` with ASCII `--`.

### Bug 3: Missing "Do NOT use for" section (MEDIUM)
**File:** `skills/etherfi-borrowing/SKILL.md`

**Problem:** SKILL.md had no "Do NOT use for" section to guide the AI agent on out-of-scope actions.

**Fix:** Added section listing 4 explicit exclusions: direct borrowing, collateral deposit via EOA, non-Scroll chains, and ETH staking.

### Bug 4: --dry-run documented as per-subcommand flag (LOW)
**File:** `skills/etherfi-borrowing/SKILL.md`

**Problem:** SKILL.md showed `--dry-run` after the subcommand name (e.g. `supply-liquidity --amount 0.01 --dry-run`), but the flag is defined as a global flag in `main.rs` and must appear before the subcommand.

**Fix:** Updated all command usage examples in SKILL.md to show `etherfi-borrowing [--dry-run] <subcommand> ...` and added a note clarifying placement.

---

## Static Analysis Findings (No Fix Required)

| Item | Status | Notes |
|---|---|---|
| `extract_tx_hash` returns `String` not `Result` | OK | Returns "pending" fallback; ok-check guards prevent silent failures |
| `source_repo` in plugin.yaml | OK | `GeoGu360/onchainos-plugins` matches the actual remote |
| Amount precision (USDC 6 decimals) | OK | `(amount * 1_000_000.0).round() as u128` is correct |
| Calldata encoding | OK | ABI encoding verified for approve, supply, withdraw, repay selectors |
| RPC fallback | OK | Uses `unwrap_or_default()` / `unwrap_or(0)` for read calls; does not abort on individual market fetch failure |
| Unused constants (CASH_DATA_PROVIDER, USER_SAFE_LENS, etc.) | INFO | Dead code warnings; not a functional issue |
| Unused variable `weeth_collateral` and `get_col_data` in position.rs | INFO | Dead code from incomplete collateral breakdown; not a functional issue |

---

## Files Changed

- `etherfi-borrowing/.gitignore` -- added (new) to exclude `target/`
- `etherfi-borrowing/skills/etherfi-borrowing/SKILL.md` -- ASCII em-dashes, Do NOT use for section, dry-run placement
- `etherfi-borrowing/src/commands/supply.rs` -- ok-checks on approve + supply tx
- `etherfi-borrowing/src/commands/withdraw.rs` -- ok-check on withdraw tx
- `etherfi-borrowing/src/commands/repay.rs` -- ok-checks on approve + repay tx

---

## Commit

```
cb05a66  fix(etherfi-borrowing): ok-check on tx results, ASCII SKILL.md, Do NOT use for section
```

Pushed to: `origin/main` (GeoGu360/onchainos-plugins)
