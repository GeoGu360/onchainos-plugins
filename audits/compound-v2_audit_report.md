# Skill Audit Report — compound-v2

**Repo**: https://github.com/GeoGu360/onchainos-plugins (dir: compound-v2)
**Audit date**: 2026-04-06
**Auditor**: skill-auditor (Claude Sonnet 4.6)
**Test wallet**: 0x87fb0647faabea33113eaf1d80d67acb1c491b90
**Test chain**: Ethereum Mainnet (chain 1)

---

## Summary

| Item | Result |
|------|--------|
| Build | PASS (0 warnings after fixes) |
| Commands tested | 7 / 7 |
| Dry-run tests | 7 / 7 PASS |
| Live write ops | 1 (claim-comp: confirmed on-chain) |
| ABI selectors verified | 13 / 13 correct |
| Bugs fixed | 4 |
| Pushed to monorepo main | DONE (b7b3b1b) |

---

## Test Plan

| # | Command | Type | Key Params |
|---|---------|------|-----------|
| 1 | `markets` | Read | -- |
| 2 | `positions` | Read | --wallet |
| 3 | `supply --dry-run` | Dry-run | --asset, --amount |
| 4 | `redeem --dry-run` | Dry-run | --asset, --ctoken-amount |
| 5 | `borrow --dry-run` | Dry-run (only) | --asset, --amount |
| 6 | `repay --dry-run` | Dry-run (only) | --asset, --amount |
| 7 | `claim-comp` | Live write | --from |

---

## Command Test Results

| # | Command | Status | Tx Hash | On-chain | Notes |
|---|---------|--------|---------|----------|-------|
| 1 | `markets` | PASS | - | - | 4 markets returned with APR/exchange rate |
| 2 | `positions --wallet 0x87fb...` | PASS | - | - | Empty positions (no active Compound V2 positions) |
| 3 | `supply USDT 0.01 --dry-run` | PASS | - | - | 2-step approve+mint calldata correct |
| 4 | `redeem DAI 0.5 --dry-run` | PASS | - | - | redeem(uint256) calldata correct |
| 5 | `borrow USDC 1.0 --dry-run` | PASS | - | - | dry-run only enforced with exit=1 (after fix) |
| 6 | `repay USDT 1.0 --dry-run` | PASS | - | - | dry-run only enforced with exit=1 (after fix) |
| 7 | `claim-comp --from 0x87fb...` | PASS | 0x2409c12c3121cf19b3a96cd78bfa9c51b680915640a4623469292092156d3689 | CONFIRMED status=1 block=24818966 | 0 COMP accrued (expected, no prior positions) |

Note: `supply` live execution skipped — Compound V2 mints are paused at contract level (protocol deprecated), would revert. `redeem` skipped — wallet holds no cTokens.

---

## Bugs Found and Fixed

### P1 — `extract_tx_hash` returned `String` not `Result<String>`

**File**: `src/onchainos.rs:79`

**Problem**: Previous signature `pub fn extract_tx_hash(result: &Value) -> String` silently returned the string `"pending"` if the wallet call failed or returned no txHash. Callers (supply, redeem, claim-comp) never detected contract-call failures — they would report success and update the UI even when the transaction was rejected.

**Fix**: Changed to `pub fn extract_tx_hash(result: &Value) -> anyhow::Result<String>`. Now:
- Checks `ok == false` first and bails with the error message
- Bails if txHash is missing, empty, or the zero-hash (dry-run sentinel)
- All callers updated to propagate with `?`

**Status**: Fixed, compiled, verified.

---

### P1 — `borrow` and `repay` returned `Ok({"ok": false})` with exit code 0

**Files**: `src/commands/borrow.rs:23`, `src/commands/repay.rs:23`

**Problem**: When called without `--dry-run`, these commands returned `Ok(json!({"ok": false, "error": "..."}))`. The `main.rs` only exits with code 1 on `Err(...)`. So callers received exit code 0, making it impossible to detect the error programmatically. The error JSON also went to stdout instead of stderr.

**Fix**: Replaced the `Ok(json!(...))` pattern with `anyhow::bail!(...)`. Now the error goes to stderr as `{"ok": false, "error": "..."}` and exit code is 1.

**Status**: Fixed, compiled, verified (exit=1 confirmed).

---

### P2 — SKILL.md `description` frontmatter contained CJK characters

**File**: `skills/compound-v2/skill.md:3`

**Problem**: The YAML frontmatter `description` field contained Chinese trigger phrases (`在Compound供应`, `Compound存款`, etc.). Frontmatter description must be ASCII-only for YAML parser compatibility across environments.

**Fix**: Replaced CJK trigger phrases with ASCII equivalents (`compound v2 supply`, `compound v2 borrow`, `compound v2 positions`, `compound v2 claim`). Also added "Do NOT use for Compound V3 / token swaps" guidance.

**Status**: Fixed.

---

### P2 — Dead code warnings: `pad_u128`, `erc20_balance_of`

**File**: `src/rpc.rs:54,98`

**Problem**: Two public functions were never called anywhere in the codebase, generating compiler warnings.

**Fix**: Removed both functions.

**Status**: Fixed, zero warnings on rebuild.

---

## Static Code Review Checklist

| Check | Result |
|-------|--------|
| ABI selectors correct | PASS — all 13 selectors verified via keccak256 |
| `extract_tx_hash` returns `Result` | FIXED (was `String`) |
| `ok`-field check after wallet_contract_call | FIXED (via extract_tx_hash Result propagation) |
| `borrow`/`repay` dry-run-only enforcement | FIXED (exit code 1 on live attempt) |
| SKILL.md description ASCII-only | FIXED |
| SKILL.md "Do NOT use for" rule | FIXED (added) |
| `source_repo` correct | PASS — `GeoGu360/onchainos-plugins` matches remote |
| `amount` precision (f64 → u128 via to_raw) | PASS — correct decimal scaling per asset |
| ERC-20 approve uses contract-call | PASS |
| Compiler warnings | FIXED — zero warnings |
| Unused imports | PASS — none |
| Panic / unwrap in hot paths | PASS — uses anyhow::Result throughout |
| Chain guard (only mainnet 1) | PASS — all commands enforce chain_id == 1 |

---

## Architecture Notes

- Read ops (`markets`, `positions`) use direct `eth_call` via `https://ethereum.publicnode.com` — no wallet needed, works correctly.
- Write ops use `onchainos wallet contract-call --force` — correct pattern.
- The protocol is officially deprecated; `supply` (mint) will revert on-chain but the plugin correctly documents this in `skill.md`.
- `borrow` and `repay` are intentionally dry-run-only and now properly enforce this with exit code 1.
- Exchange rate math is correct: `underlying = ctoken_balance * exchange_rate / 1e18`, then divide by `10^underlying_decimals`.

---

## Commit

Fix pushed to monorepo main: **b7b3b1b**
`fix(compound-v2): extract_tx_hash returns Result, ok-check, borrow/repay exit code, dead code removal`
