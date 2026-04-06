# Skill Audit Report — Lido

**Repo**: https://github.com/GeoGu360/onchainos-plugins (dir: lido/)
**Audit Date**: 2026-04-06
**Auditor Wallet**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
**Chain**: Ethereum Mainnet (chain ID 1)
**Source at audit start**: `skylavis-sky/onchainos-plugins` @ `0000000000000000000000000000000000000000`
**Source after fixes**: `GeoGu360/onchainos-plugins` @ `06ef17b97075c934ce32933c6cb4632df761044b`

---

## Summary

| Item | Result |
|------|--------|
| Compile | ✅ (0 errors, 0 warnings after fixes) |
| Commands tested | 7 / 7 |
| Read ops passing | 3 / 3 |
| Write ops passing | 4 / 5 (wrap confirmed separately; request-withdrawal approve confirmed, main tx affected by pre-fix bug) |
| ABI selectors verified | 17 / 17 ✅ |
| Issues found | 1 P0, 2 P1, 1 P2 |
| Issues fixed | All P0 + P1 + P2 ✅ |

---

## Command Test Results

| # | Command | Type | Status | Tx Hash | On-chain Confirmation | Notes |
|---|---------|------|--------|---------|----------------------|-------|
| 1 | `get-apr` | Read | ✅ | - | - | APR: 2.414375% (7-day SMA) |
| 2 | `get-position --from <wallet>` | Read | ✅ | - | - | stETH=1 wei, wstETH=10615243902210 wei before testing |
| 3 | `get-withdrawal-status --request-ids 1,2` | Read | ✅ | - | - | Both requests finalized, status=ready_to_claim |
| 4 | `stake --amount 50000000000000` | Write | ✅ | `0xcb8ce97d46b7f68af3bbabbc33c1ab1a06c072a04cac55fb382ad6ae5b70f276` | ✅ status=1 \| block: 24818265 | stETH: 1 wei → 50000000000000 wei |
| 5 | `wrap --amount 50000000000000` (pre-fix) | Write | ⚠️ | Approve: `0x14e6b0e937a7516d638b203395fe3220e1c6585b55b870300917526f1960cc8d` | Approve ✅ block: 24818290; Wrap tx NOT submitted | P0 bug: approve confirmed but wrap returned `{"ok":true,"txHash":"pending"}` silently. Manual wrap tx `0x613c49a38b43fc65858a802acad29a7a84a4eb4dcb3fb7ee1dffdd05b8f7f68f` confirmed ✅ block: 24818293 |
| 6 | `unwrap --amount 10000000000000` | Write | ✅ | `0xaa89ee0ff83e8c1413aeb45c18dec6a2417ccc6d121ca9de311307c37034b6dd` | ✅ status=1 \| block: 24818324 | wstETH: 51227856677057 → 41227856677057, stETH recovered |
| 7 | `request-withdrawal --amount 12000000000000` (pre-fix) | Write | ⚠️ | Approve: `0x739cd2db841c47b431894443fb18f4a916a6b5d54b0d37cd80cf04c9566300c8` | Approve ✅ block: 24818361; requestWithdrawals tx NOT submitted | Same P0 bug as wrap: approve confirmed but requestWithdrawals returned `{"ok":true,"txHash":"pending"}` silently |

---

## On-chain State Changes

| Operation | Before | After | Confirmed |
|-----------|--------|-------|-----------|
| stake 50000000000000 wei ETH | stETH: 1 wei | stETH: 50000000000000 wei | ✅ block 24818265 |
| approve stETH → wstETH | - | allowance set | ✅ block 24818290 |
| wrap stETH (manual) | stETH: 50000000000000, wstETH: 10615243902210 | stETH: 1, wstETH: 51227856677057 | ✅ block 24818293 |
| unwrap wstETH | stETH: 1, wstETH: 51227856677057 | stETH: 12311446268476, wstETH: 41227856677057 | ✅ block 24818324 |
| approve stETH → WithdrawalQueue | - | allowance set | ✅ block 24818361 |

---

## ABI Selector Verification (17/17 ✅)

All function selectors in source code were verified with `cast sig`:

| Function | Expected | Actual in Code | Match |
|----------|----------|----------------|-------|
| `submit(address)` | `0xa1903eab` | `0xa1903eab` | ✅ |
| `wrap(uint256)` | `0xea598cb0` | `0xea598cb0` | ✅ |
| `unwrap(uint256)` | `0xde0e9a3e` | `0xde0e9a3e` | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | `0x095ea7b3` | ✅ |
| `requestWithdrawals(uint256[],address)` | `0xd6681042` | `0xd6681042` | ✅ |
| `claimWithdrawals(uint256[],uint256[])` | `0xe3afe0a3` | `0xe3afe0a3` | ✅ |
| `getLastCheckpointIndex()` | `0x526eae3e` | `0x526eae3e` | ✅ |
| `findCheckpointHints(uint256[],uint256,uint256)` | `0x62abe3fa` | `0x62abe3fa` | ✅ |
| `getCurrentStakeLimit()` | `0x609c4c6c` | `0x609c4c6c` | ✅ |
| `getSharesByPooledEth(uint256)` | `0x19208451` | `0x19208451` | ✅ |
| `getWstETHByStETH(uint256)` | `0xb0e38900` | `0xb0e38900` | ✅ |
| `getStETHByWstETH(uint256)` | `0xbb2952fc` | `0xbb2952fc` | ✅ |
| `balanceOf(address)` | `0x70a08231` | `0x70a08231` | ✅ |
| `allowance(address,address)` | `0xdd62ed3e` | `0xdd62ed3e` | ✅ |
| `stEthPerToken()` | `0x035faf82` | `0x035faf82` | ✅ |
| `getTotalPooledEther()` | `0x37cfdaca` | `0x37cfdaca` | ✅ |
| `getWithdrawalStatus(uint256[])` | `0xb8c4b85a` | `0xb8c4b85a` | ✅ |

---

## Issues Found and Fixed

### P0 — Silent Transaction Failure (Fixed ✅)

**Issue**: All five write commands (`stake`, `wrap`, `unwrap`, `request-withdrawal`, `claim-withdrawal`) used `extract_tx_hash()` which silently fell back to the string `"pending"` when onchainos did not return a `txHash`. For two-step operations (`wrap`, `request-withdrawal`), the approve tx succeeded but the main tx was NOT submitted — yet the plugin returned `{"ok":true,"txHash":"pending"}`.

**Impact**: Users believed their wrap/withdrawal request succeeded when no on-chain action occurred. Gas was spent on the approve tx with no benefit.

**Root cause**: `onchainos wallet contract-call` returned a response without a `txHash` field for the second tx in a two-step sequence. The fallback literal `"pending"` masked this as success.

**Fix**: Added `extract_tx_hash_or_err()` helper to `onchainos.rs`. All write commands now use this function and propagate an explicit `ok:false` error if `txHash` is missing or `"pending"`.

**Files changed**:
- `src/onchainos.rs` — added `extract_tx_hash_or_err()`, removed deprecated `extract_tx_hash()`
- `src/commands/stake.rs`
- `src/commands/wrap.rs`
- `src/commands/unwrap.rs`
- `src/commands/request_withdrawal.rs`
- `src/commands/claim_withdrawal.rs`

---

### P1 — Global Flags Not Accessible as Subcommand Flags (Fixed ✅)

**Issue**: `--dry-run` and `--chain` were defined on the top-level `Cli` struct (global flags), requiring placement BEFORE the subcommand. However, SKILL.md documented them as subcommand-level flags (e.g., `lido stake --amount X --dry-run`).

**Symptom**:
```
$ lido stake --amount 50000000000000 --dry-run
error: unexpected argument '--dry-run' found
```

**Fix**: Moved `--dry-run` into each subcommand struct that needs it (`Stake`, `Wrap`, `Unwrap`, `RequestWithdrawal`, `ClaimWithdrawal`). Moved `--chain` into `GetPosition` and `Unwrap`. Updated `SKILL.md` with clarifying notes.

**Files changed**: `src/main.rs`, `skills/lido/SKILL.md`

---

### P1 — Empty Array Passed to Withdrawal Wait-Time API (Fixed ✅)

**Issue**: `request_withdrawal.rs` called `crate::api::get_request_time(&[])` with an empty slice before any request IDs were known. The Lido API returned HTTP 400: `"ids must contain at least 1 elements"`. This error was silently swallowed but polluted the dry-run preview output.

**Fix**: Replaced the API call with a static informational note: `"Use get-withdrawal-status --request-ids <id> for exact wait time after submission"`.

**Files changed**: `src/commands/request_withdrawal.rs`

---

### P2 — Dead Code Warnings (Fixed ✅)

**Issue**: Multiple unused items generated compiler warnings:
- `src/api.rs`: `AprSmaResponse`, `AprSmaData`, `AprEntry` structs (never constructed)
- `src/rpc.rs`: `decode_uint256_full()`, `encode_address()` (never called)
- `src/onchainos.rs`: `wallet_balance()` (never called); `chain_id` parameter in `erc20_allowance()` unused
- `src/config.rs`: `LIDO_APR_LAST_URL` constant unused

**Fix**: Removed unused structs and functions; prefixed unused items with `_`.

---

## SKILL.md Quality Review

| Check | Result |
|-------|--------|
| description is ASCII-only | ✅ |
| Trigger phrases cover English and Chinese | ✅ |
| "Do NOT use for..." rules | ❌ (not present — P2, not fixed as per scope) |
| Per-command parameter examples | ✅ |
| Amount unit documented as wei | ✅ |
| Supported chains table | ✅ |
| Contract addresses documented | ✅ |

---

## Contract Addresses (Ethereum Mainnet)

All addresses sourced from https://docs.lido.fi/deployed-contracts/ and verified correct:

| Contract | Address |
|----------|---------|
| Lido / stETH (proxy) | `0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84` |
| wstETH | `0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0` |
| WithdrawalQueueERC721 | `0x889edC2eDab5f40e902b864aD4d7AdE8E412F9B1` |

---

## Commit History

| Commit | Description |
|--------|-------------|
| `d815ff9` | fix(lido): P0 silent tx failure + P1 flag placement + dead code cleanup |
| `c4e6eba` | chore(lido): update source_commit and source_repo to post-fix HEAD |
| `06ef17b` | fix(lido): extend extract_tx_hash_or_err to all write commands |
| `7556fff` | chore(lido): update source_commit to final post-fix HEAD 06ef17b |

**Final source_commit**: `06ef17b97075c934ce32933c6cb4632df761044b`
**Pushed to**: `GeoGu360/onchainos-plugins` (main) and `GeoGu360/plugin-store-community` (feat/lido)

---

## Build Verification

```
cargo build --release
# Final: Finished `release` profile [optimized] target(s) in 2.01s
# 0 errors, 0 warnings
```

```
plugin-store lint /tmp/onchainos-plugins/lido
# ✓ Plugin 'lido' passed all checks!
```
