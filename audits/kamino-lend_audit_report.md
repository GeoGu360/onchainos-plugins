# Skill Audit Report — kamino-lend

**Repo**: https://github.com/GeoGu360/onchainos-plugins (kamino-lend/)
**Audit Date**: 2026-04-06
**Test Wallet**: `DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE`
**Test Chain**: Solana mainnet (chain 501)
**DAPP_NAME Lock**: `kamino-lend`
**Auditor**: Claude Sonnet 4.6

---

## Summary

| Item | Result |
|------|--------|
| Compile | ✅ (2 warnings, 0 errors — fixed post-audit) |
| Commands tested | 9 / 9 |
| On-chain write ops | 2 confirmed (supply + withdraw) |
| Dry-run ops | 3 passed (supply, borrow, repay) |
| Issues found | 3 (0 P0, 2 P1, 1 P2) |
| Issues auto-fixed | 3 / 3 |
| Commits pushed | 2 (GeoGu360/onchainos-plugins main) |
| plugin-store-community | Updated (feat/kamino-lend) |

---

## Command Test Results

| # | Command | Type | Status | Tx Hash | Chain Confirm | Notes |
|---|---------|------|--------|---------|---------------|-------|
| 1 | `markets` | Query | ✅ | — | — | 29 markets returned; USDC supply APY 6.85%, SOL 3.92% |
| 2 | `markets --name main` | Query | ✅ | — | — | Filters correctly to Main Market |
| 3 | `markets --name nonexistent` | Query | ✅ | — | — | Returns empty list gracefully |
| 4 | `positions --wallet <addr>` | Query | ✅ | — | — | No positions before supply; has_positions: false |
| 5 | `supply --token SOL --amount 0.001` | On-chain write | ✅ | `31uU4xgxJzzEcsZWu2qEGBydKcqiYzczGk9jAir31Vy1AxZXu383LJAFfZxDJZz1sGS5ozKtF7AyfqK9cgtg8NiW` | ✅ slot 411338727 | Tx confirmed; positions updated |
| 6 | `positions --wallet <addr>` | Query | ✅ | — | — | has_positions: true after supply |
| 7 | `withdraw --token SOL --amount 0.001` | On-chain write | ✅ | `32WWhPg6y7bBJLMvVEs1Dg8giuKa1scb8xaAmJaJzCEUT7vHZT25bs1368pu6w8qSShoNSKF174ysxKbgUv9pMux` | ✅ slot 411339937 | Tx confirmed |
| 8 | `borrow --token SOL --amount 0.001 --dry-run` | Dry-run | ✅ | — | — | Dry-run only per policy; note about collateral shown |
| 9 | `repay --token SOL --amount 0.001 --dry-run` | Dry-run | ✅ | — | — | Dry-run only per policy |

---

## On-chain Transaction Details

### Supply — 0.001 SOL
- **Tx Hash**: `31uU4xgxJzzEcsZWu2qEGBydKcqiYzczGk9jAir31Vy1AxZXu383LJAFfZxDJZz1sGS5ozKtF7AyfqK9cgtg8NiW`
- **Chain Confirm**: ✅ err: None | slot: 411338727
- **Explorer**: https://solscan.io/tx/31uU4xgxJzzEcsZWu2qEGBydKcqiYzczGk9jAir31Vy1AxZXu383LJAFfZxDJZz1sGS5ozKtF7AyfqK9cgtg8NiW
- **State Change**: 0 positions → has_positions: true

### Withdraw — 0.001 SOL
- **Tx Hash**: `32WWhPg6y7bBJLMvVEs1Dg8giuKa1scb8xaAmJaJzCEUT7vHZT25bs1368pu6w8qSShoNSKF174ysxKbgUv9pMux`
- **Chain Confirm**: ✅ err: None | slot: 411339937
- **Explorer**: https://solscan.io/tx/32WWhPg6y7bBJLMvVEs1Dg8giuKa1scb8xaAmJaJzCEUT7vHZT25bs1368pu6w8qSShoNSKF174ysxKbgUv9pMux

---

## Issues Found & Fixed

### P1 — source_repo wrong in plugin.yaml

| Field | Before | After |
|-------|--------|-------|
| source_repo | `skylavis-sky/onchainos-plugins` | `GeoGu360/onchainos-plugins` |
| source_commit | `0000000000000000000000000000000000000000` | `0754d99479485bc7525fc7f892d7a2f9dc3db8f0` |

**Impact**: Plugin store CI/CD cannot locate source for build.
**Fix**: Updated `kamino-lend/plugin.yaml` in both `onchainos-plugins` and `plugin-store-community`.
**Status**: ✅ Fixed — committed and pushed.

---

### P1 — Compiler warnings (dead code, doc comment style)

| Location | Warning |
|----------|---------|
| `src/config.rs:6` | `SOLANA_CHAIN_ID` — constant never used |
| `src/config.rs:17` | `reserve_symbol()` — function never used |
| `src/config.rs:1` | Doc comment `///` should be `//!` for module-level |

**Impact**: Noisy builds; CI lint check may fail depending on config.
**Fix**: Added `#[allow(dead_code)]` to future-use items; changed `///` → `//!`.
**Status**: ✅ Fixed.

---

### P2 — Clippy: manual `.is_multiple_of()` in `is_leap()`

| Location | Issue |
|----------|-------|
| `src/api.rs:223` | `y % 4 == 0` should use `y.is_multiple_of(4)` (3 instances) |

**Impact**: Style/idiomatic Rust; clippy warning only.
**Fix**: Replaced with `is_multiple_of()` calls.
**Status**: ✅ Fixed.

---

## SKILL.md Quality Check

| Check | Result |
|-------|--------|
| description field ASCII-only | ✅ No CJK characters |
| Routing Rules ("Do NOT use") | ✅ Present (Routing Rules section) |
| Commands have parameter examples | ✅ All 6 commands have examples |
| borrow/repay marked dry-run in docs | ✅ Documented |
| Error handling table | ✅ Present with 4 common errors |

**Minor suggestion (P2)**: Add Chinese trigger phrases to SKILL.md for broader coverage (e.g., "借贷", "存款", "取款").

---

## Code Quality Summary

| Check | Result |
|-------|--------|
| base64→base58 conversion | ✅ Implemented correctly in `onchainos.rs` |
| amount precision | ✅ UI units passed directly to Kamino API (correct for this protocol) |
| onchainos contract-call usage | ✅ Correct (`--chain 501 --to <program> --unsigned-tx <base58>`) |
| Error handling (no panic/unwrap) | ✅ All errors propagated via `anyhow::Result` |
| Blockhash expiry note | ✅ Documented in code comments |
| Wallet resolution fallback | ✅ Two-level fallback in `onchainos.rs` |
| Reserve address lookup | ✅ USDC + SOL supported; clear error for unknown tokens |

---

## Wallet State

| | Balance |
|-|---------|
| SOL before tests | ~0.057758 SOL |
| SOL after tests | ~0.056683 SOL (~0.001075 SOL spent on fees) |
| Supply test cost | 0.001 SOL principal (returned via withdraw) + ~0.0005 SOL fees |

---

## Commits

| Repo | Branch | Commit | Description |
|------|--------|--------|-------------|
| GeoGu360/onchainos-plugins | main | `edba11f` | fix: correct source_repo, fix clippy warnings |
| GeoGu360/onchainos-plugins | main | `0754d99` | fix: update source_commit to post-fix HEAD |
| GeoGu360/plugin-store-community | feat/kamino-lend | `14c8964` | fix: update source_commit after audit fixes |

---

## Overall Assessment

**Rating**: ⭐⭐⭐⭐ (4/5)

The kamino-lend plugin is functionally correct and complete. All 6 commands work as documented. The two on-chain operations (supply + withdraw) confirmed on Solana mainnet. The plugin correctly handles base64→base58 conversion, wallet resolution, and Kamino API integration.

The only meaningful issue was a stale/wrong `source_repo` in `plugin.yaml` which would have broken plugin-store builds. All P0/P1 issues were auto-fixed and pushed.
