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

---

## Re-audit — 2026-04-06

**Re-audit trigger:** Wallet now has ETH on Scroll (0.002409 ETH / ~$5.19). Re-auditing to attempt live write operations.

**Wallet:** `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
**Chain:** Scroll (chain ID 534352)
**ETH balance:** 0.002409 ETH
**USDC balance:** 0.000000 USDC

### Build

`cargo build --release` — PASS (8 pre-existing unused-variable warnings, no errors). Build time: 0.11s (cached).

### Read-Only Commands (Live)

| Command | Result | Key Output |
|---|---|---|
| `markets` | PASS | USDC borrow market: total_supply=5005.0008 USDC, total_borrow=10.3332 USDC, utilization=0.21%, borrow_apy=0.0000% |
| `rates` | PASS | borrow_token=USDC, borrow_apy_pct=0.000000, utilization=0.21% |
| `position --user-safe 0x87fb...` | PASS | No position (0 USDC debt, 0 supplier balance, not liquidatable) as expected for a plain EOA |

### Write Operations Assessment

**The plugin does NOT support ETH deposit or ETH-backed borrowing.** All write operations (`supply-liquidity`, `withdraw-liquidity`, `repay`) are USDC-only. The wallet's ETH balance on Scroll cannot be used as input for any of the plugin's write operations.

The protocol accepts collateral deposits (weETH/SCR) only through the EtherFi Cash app UserSafe smart wallet — not via this plugin. This plugin's `supply-liquidity` command supplies USDC to a lending pool; it is not an ETH deposit function.

### Dry-run Write Operations (Re-verified)

| Command | Result | Notes |
|---|---|---|
| `--dry-run supply-liquidity --amount 0.01` | PASS | Correct calldata for `supply(address,address,uint256)` selector `0x0c0a769b`, amount_raw=10000 (6 decimals) |
| `--dry-run withdraw-liquidity --amount 0.01` | PASS | Correct calldata for withdraw selector `0xa56c8ff7`, amount_raw=10000 |

### Error Handling Re-verified

`supply-liquidity --amount 999999999` → `Error: Insufficient USDC balance on Scroll. Have: 0.000000 USDC, need: 999999999.000000 USDC` — PASS, user-friendly message, exits with code 1.

### Live Write Operations

**Not executed.** Reason: The plugin requires USDC on Scroll for all write operations. The wallet holds only ETH on Scroll (0.002409 ETH). Converting ETH to USDC is outside the scope of this plugin and would require a DEX swap, which is not part of this audit scope. No new bugs were found that would block a live test once USDC is available.

### Re-audit Verdict

**PASS (read operations) / N/A (write operations — USDC required, wallet has none)**

All read commands work correctly on live Scroll RPC. The plugin correctly blocks write operations with a friendly error when USDC balance is insufficient. The previously applied fixes (ok-check, ASCII SKILL.md, dry-run flag placement) remain intact. No new issues found.
