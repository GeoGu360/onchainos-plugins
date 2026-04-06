# Skill Audit Report — Symbiotic

**Repo**: https://github.com/GeoGu360/onchainos-plugins (dir: symbiotic/)
**Audit Date**: 2026-04-06
**Auditor Wallet**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
**Chain**: Ethereum Mainnet (chain ID 1)
**Source at audit start**: `GeoGu360/onchainos-plugins` @ `6b02dabc` (before fixes)
**Source after fixes**: `GeoGu360/onchainos-plugins` @ `fec6b09`

---

## Summary

| Item | Result |
|------|--------|
| Compile | ✅ (0 errors, 15 dead-code warnings) |
| Commands tested | 7 / 7 |
| Read ops passing | 4 / 4 |
| Write ops (deposit) | ✅ confirmed on-chain (via manual retry after nonce conflict) |
| Write ops (withdraw dry-run) | ✅ calldata correct after fix |
| ABI selectors verified | 7 / 7 ✅ |
| Issues found | 0 P0, 3 P1, 2 P2 |
| Issues fixed | All 3 P1 ✅ |

---

## Command Test Results

| # | Command | Type | Status | Tx Hash | On-chain Confirmation | Notes |
|---|---------|------|--------|---------|----------------------|-------|
| 1 | `symbiotic vaults --limit 5` | Read | ✅ | - | - | 5 vaults returned, TVL/APR data live |
| 2 | `symbiotic vaults --token wstETH --limit 3` | Read | ✅ | - | - | 3 wstETH vaults returned correctly |
| 3 | `symbiotic rates --limit 5` | Read | ✅ | - | - | Sorted by APR desc, highest 7.09% (stHYPER) |
| 4 | `symbiotic positions --address 0x87fb...` | Read | ✅ | - | - | 0 positions before deposit; 1 position (0.00004 wstETH) after |
| 5 | `symbiotic deposit --token wstETH --amount 0.00004 --dry-run` | Read | ✅ (after fix) | - | - | Pre-fix: onBehalfOf=0x000...000 in calldata. Post-fix: correct address |
| 6 | `symbiotic deposit --token wstETH --amount 0.00004` | Write | ⚠️ | Approve: `0xceccebada1831a1624d55ab6ae2655999d7edf2685107048e5c8af7cb0fc2d9f` / Deposit: `0x6989196025a0de8b92342e810b169da2954253255ba56b5bdff57c22d3c95f54` | ✅ Both confirmed | Approve confirmed block ~24821783; deposit returned `"pending"` (nonce conflict during busy period), manually retried → confirmed ✅. Fixed ok-checks prevent silent failures going forward. |
| 7 | `symbiotic withdraw --token wstETH --amount 0.00004 --dry-run` | Read | ✅ (after fix) | - | - | Pre-fix: onBehalfOf=0x000...000. Post-fix: correct address. Epoch=25, ~7 days |

---

## On-chain State Changes

| Operation | Before | After | Confirmed |
|-----------|--------|-------|-----------|
| approve wstETH → Stakestone vault | allowance=0 | allowance=40000000000000 wei | ✅ `0xceccebad...` block ~24821783 |
| deposit 0.00004 wstETH | positions=0 | 0.00004 wstETH restaked @ 2.74% APR | ✅ `0x69891960...` block ~24821800 |

---

## Issues Found and Fixed

### P1 — `deposit`/`withdraw` dry-run uses zero address in calldata

**File**: `src/commands/deposit.rs:34`, `src/commands/withdraw.rs:34`

**Before**: When `--dry-run` is used without `--from`, the wallet defaulted to `0x0000000000000000000000000000000000000000` and the preview calldata encoded this zero address as `onBehalfOf`/`claimer`. This gave users a misleading transaction preview.

**After**: Wallet is always resolved first (for both dry-run and live), so calldata always encodes the real wallet address.

---

### P1 — No `ok`-check on approve before submitting deposit step

**File**: `src/commands/deposit.rs:118` (original)

**Before**: After calling `erc20_approve`, the code proceeded directly to the vault deposit regardless of whether the approve succeeded. If approve returned `ok: false`, the deposit would still be attempted (and fail expensively, or silently).

**After**: Added `if approve_result["ok"].as_bool() != Some(true)` guard. If approve fails, the command bails with a clear error message before the deposit step.

---

### P1 — No `ok`-check on deposit/withdraw result (always returns `ok: true`)

**File**: `src/commands/deposit.rs:141`, `src/commands/withdraw.rs:119` (original)

**Before**: Both deposit and withdraw always returned `"ok": true` in the final JSON response, even if the onchainos contract-call responded with `ok: false` or returned without a `txHash`. The `extract_tx_hash` helper returned `"pending"` silently on failure, masking the real error.

**After**: Added `ok`-check on both deposit and withdraw results. Failures now surface as proper error responses with the onchainos error message.

---

## P2 Issues (Not Fixed — Informational)

### P2 — Dead code warnings: unused constants and helper functions

**File**: `src/config.rs` (5 unused constants), `src/rpc.rs` (`decode_address`, `vault_collateral`, `total_stake`, `token_symbol`, `token_decimals`)

These constants and functions were defined for potential future use but generate 15 compiler warnings. They do not affect correctness. Recommend either removing unused items or using `#[allow(dead_code)]` if intentionally kept for future use.

---

### P2 — 15-second inter-step delay may be insufficient under load

**File**: `src/commands/deposit.rs:122`

The code waits 15 seconds between the approve and deposit transactions. During high wallet activity (e.g. concurrent audits sharing the same nonce sequence), this delay is insufficient and the deposit call may return without a `txHash` due to nonce sequencing issues in the onchainos backend. The added `ok`-check in P1 fix will now surface this as an error rather than silent failure, but increasing the delay to 20–30 seconds or implementing retry logic would be more robust.

---

## SKILL.md Quality

| Check | Result |
|-------|--------|
| `description` ASCII-only (no CJK) | ✅ |
| Trigger phrases cover common use cases | ✅ English phrases for all 5 commands |
| "Do NOT use for..." rule | ❌ Missing — could add to prevent mis-triggering on non-Symbiotic restaking queries |
| Parameters have examples | ✅ All commands have usage examples |
| Chinese trigger phrases | ⚠️ Missing — other plugins include Chinese triggers; adding would improve usability for Chinese-speaking users |

**Recommendation**: Add a "Do NOT use for" section and Chinese trigger phrases. Example:

```markdown
**Do NOT use for:**
- EigenLayer, Lido, or other restaking protocols
- Symbiotic governance or tokenomics questions
```

---

## Code Quality Checklist

| Check | Result |
|-------|--------|
| Contract addresses hardcoded | ⚠️ Fallback addresses in `config.rs` (unused); primary resolution via API ✅ |
| Amount precision correct | ✅ `parse_amount()` handles `uint8` decimals, e.g. 18 for wstETH |
| `onchainos` contract-call usage correct | ✅ Uses `contract-call` for vault deposit/withdraw; `erc20_approve` correctly uses separate approve step |
| Error messages user-friendly | ✅ After P1 fixes; errors are descriptive strings, no panics |
| `ok`-check on write ops | ✅ After P1 fixes |
| ABI selectors verified with `cast sig` | ✅ All 7 selectors match |

---

## ABI Selector Verification

| Function | Expected | `cast sig` | Match |
|----------|---------|-----------|-------|
| `deposit(address,uint256)` | `0x47e7ef24` | `0x47e7ef24` | ✅ |
| `withdraw(address,uint256)` | `0xf3fef3a3` | `0xf3fef3a3` | ✅ |
| `activeBalanceOf(address)` | `0x59f769a9` | `0x59f769a9` | ✅ |
| `collateral()` | `0xd8dfeb45` | `0xd8dfeb45` | ✅ |
| `currentEpoch()` | `0x76671808` | `0x76671808` | ✅ |
| `epochDuration()` | `0x4ff0876a` | `0x4ff0876a` | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | `0x095ea7b3` | ✅ |

---

## Overall Assessment

The Symbiotic plugin is **functionally correct** after the P1 fixes. Read operations (vaults, rates, positions) work reliably and return live on-chain data. The deposit flow succeeds end-to-end, verified by a live 0.00004 wstETH deposit into the Stakestone-Stakestone-wstETH vault. The primary issues were silent failure masking in the two-step deposit flow (fixed), and incorrect zero-address calldata in dry-run previews (fixed).

**Rating (post-fix)**: 4 / 5 — Solid implementation with good API integration and proper ABI encoding. Minor improvements needed for SKILL.md trigger coverage and noise reduction from dead code warnings.
