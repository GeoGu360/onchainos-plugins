# Skill Audit Report — dinero-pxeth

**Repo**: https://github.com/GeoGu360/onchainos-plugins (dir: dinero-pxeth/)
**Audit Date**: 2026-04-06
**Auditor Wallet**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
**Chain**: Ethereum Mainnet (chain ID 1)
**Source at audit start**: `GeoGu360/onchainos-plugins` @ `e6b05ca23667c2409cf1e598a4121ec8a4e71c00`
**Fix commit**: `79b4b04`

---

## Summary

| Item | Result |
|------|--------|
| Compile | ✅ (0 errors, 0 warnings — before and after fixes) |
| Commands tested | 6 / 6 |
| Read ops passing | 2 / 2 |
| Write ops passing (dry-run) | 3 / 3 |
| Write op (deposit live — paused guard) | ✅ ok=false returned correctly |
| ABI selectors verified | 9 / 9 ✅ |
| Issues found | 2 (P0 + P1) |
| Issues fixed | All ✅ |

---

## Command Test Results

| # | Command | Type | Status | Notes |
|---|---------|------|--------|-------|
| 1 | `rates` | Read | ✅ | apxeth_per_pxeth=1.11605975, paused=true, TVL=2598.1964 pxETH |
| 2 | `positions --address 0x87fb...` | Read | ✅ | pxETH=0, apxETH=0 (no holdings) |
| 3 | `positions` (wallet auto-resolve) | Read | ✅ | Correctly resolved to `0x87fb0647faabea33113eaf1d80d67acb1c491b90` |
| 4 | `deposit --amount 0.001 --dry-run` | Write/Dry | ✅ | Correct calldata: `0xadc9740c` + zero addr + compound=false |
| 5 | `deposit --amount 0.001 --compound --dry-run` | Write/Dry | ✅ | compound=true encoded correctly |
| 6 | `stake --amount 0.001 --dry-run` | Write/Dry | ✅ | approve_calldata + deposit_calldata both correct |
| 7 | `redeem --amount 0.001 --dry-run` | Write/Dry | ✅ | SEL_REDEEM + shares + receiver + owner encoded |
| 8 | `deposit --amount 0.001` (live) | Write | ✅ | PirexEth paused guard fires: ok=false with suggestion |

**Note**: live `stake` and `redeem` not executed as wallet holds no pxETH/apxETH. Protocol deposit is paused; pxETH cannot be acquired without OTC/swap.

---

## ABI Selector Verification (9/9 ✅)

All selectors verified with `cast sig`:

| Function | Selector in Code | cast sig Output | Match |
|----------|-----------------|-----------------|-------|
| `deposit(address,bool)` | `adc9740c` | `0xadc9740c` | ✅ |
| `deposit(uint256,address)` | `6e553f65` | `0x6e553f65` | ✅ |
| `redeem(uint256,address,address)` | `ba087652` | `0xba087652` | ✅ |
| `convertToAssets(uint256)` | `07a2d13a` | `0x07a2d13a` | ✅ |
| `totalAssets()` | `01e1d114` | `0x01e1d114` | ✅ |
| `totalSupply()` | `18160ddd` | `0x18160ddd` | ✅ |
| `balanceOf(address)` | `70a08231` | `0x70a08231` | ✅ |
| `approve(address,uint256)` | `095ea7b3` | `0x095ea7b3` | ✅ |
| `paused()` | `5c975abb` | `0x5c975abb` | ✅ |

---

## Issues Found and Fixed

### P0 — `extract_tx_hash` returns `String`, no ok-check (FIXED)

**File**: `src/onchainos.rs`

**Before**: `extract_tx_hash` returned `String` with `"pending"` as silent fallback. The callers in `deposit.rs`, `stake.rs`, and `redeem.rs` never checked `result["ok"]`, so if `onchainos wallet contract-call` returned `ok=false`, the code would silently print a success JSON with `txHash: "pending"`.

**After**: `extract_tx_hash` now returns `anyhow::Result<String>`:
- Checks `result["ok"]`; bails with the error message on `ok=false`
- Rejects empty or `"pending"` hash with a descriptive error
- All callers updated to propagate with `?`

**Impact**: In `stake.rs`, the approve step would proceed to the deposit even if the approval transaction failed. This is now caught and the command exits with an error.

### P1 — `plugin.yaml` source_commit stale (FIXED)

**File**: `plugin.yaml`

**Before**: `source_commit: "142184f229b8b54ac75b7c2735ccfbe6913e35c9"` (the initial add commit)

**After**: `source_commit: "e6b05ca23667c2409cf1e598a4121ec8a4e71c00"` (HEAD at audit start)

---

## Non-Issues (Checked but OK)

- **SKILL.md non-ASCII**: Chinese characters in the YAML frontmatter `description` field (trigger phrases) — accepted pattern, consistent with `stader` and other reviewed plugins. Emoji symbols (⚠️, ✅, →, —) appear in the body, not the frontmatter key/value pairs used for parsing.
- **amount precision**: ETH amounts use `(args.amount * 1e18) as u128` — correct; matches the standard used across all other plugins.
- **source_repo**: correctly set to `GeoGu360/onchainos-plugins`
- **"Do NOT use for" disclaimer**: not required; not present in comparable plugins (lido, stader, etc.)
- **Protocol paused guard**: `deposit` command correctly checks `PirexEth.paused()` on-chain before submitting and returns `ok=false` with a helpful message

---

## Contract Address Verification

| Contract | Address | On-chain Status (verified) |
|----------|---------|---------------------------|
| PirexEth | `0xD664b74274DfEB538d9baC494F3a4760828B02b0` | paused=true ✅ |
| pxETH token | `0x04C154b66CB340F3Ae24111CC767e0184Ed00Cc6` | totalSupply=2981.38 ✅ |
| apxETH vault | `0x9Ba021B0a9b958B5E75cE9f6dff97C7eE52cb3E6` | totalAssets=2598.19 pxETH ✅ |

---

## Fix Commit

`79b4b04` pushed to `GeoGu360/onchainos-plugins` main
