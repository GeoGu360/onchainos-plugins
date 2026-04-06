# Skill Audit Report — Solayer

**Repo**: https://github.com/GeoGu360/onchainos-plugins/tree/main/solayer
**Audit Date**: 2026-04-06
**Test Wallet**: `DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE`
**Chain**: Solana (chainIndex 501)
**Audited Binary**: `solayer` (Rust)

---

## Summary

| Item | Result |
|------|--------|
| Compilation | ✅ (after fixes) |
| Commands tested | 4 / 4 |
| On-chain write ops | 1 confirmed |
| P0 issues | 0 |
| P1 issues fixed | 2 |
| P2 issues | 1 |

---

## Step 3: Compilation

**Pre-fix**: compiled successfully with 4 warnings (unused constants + unused dead function).  
**Post-fix**: compiled successfully with **0 warnings**.

Binary: `/tmp/onchainos-plugins/solayer/target/release/solayer`

---

## Step 5: Command Test Results

| # | Command | Type | Status | Tx Hash | On-chain Confirmation | Notes |
|---|---------|------|--------|---------|----------------------|-------|
| 1 | `solayer rates` | Read | ✅ | — | — | APY 6.69%, TVL 712,185 SOL, epoch 952 |
| 2 | `solayer positions` | Read | ✅ | — | — | Wallet resolved correctly; sSOL balance 0.00087361 (pre-stake) |
| 3 | `solayer stake --amount 0.001` | Write | ✅ | `5cZGGDkXTNVnAHewsuGnr7AJgVqGYPewxCLXPjEwhoowvzdXF956vXnxEJMnsrFEJyBhffUfE9Mi5PmLjR3sjCoc` | ✅ err: None \| slot: 411339595 | Staked 0.001 SOL → 0.000875313 sSOL via Jupiter |
| 4 | `solayer stake --amount 0.001 --dry-run` | Dry-run | ✅ | — | — | Fixed in audit (was broken pre-fix) |
| 5 | `solayer unstake --amount 0.001` | Guidance | ✅ | — | — | Returns UI URL as documented |
| 6 | `solayer stake --amount 999999999` | Error test | ✅ | — | — | Clear error: "value difference > 90%" — no panic |

### Stake Write Op Detail

```
Pre-stake sSOL balance:  0.00087361  sSOL
Post-stake sSOL balance: 0.001748761 sSOL
Change:                  +0.000875151 sSOL (≈ 0.000875313 sSOL received per plugin output)
```

- Tx Hash: `5cZGGDkXTNVnAHewsuGnr7AJgVqGYPewxCLXPjEwhoowvzdXF956vXnxEJMnsrFEJyBhffUfE9Mi5PmLjR3sjCoc`
- On-chain: `err: None | slot: 411339595` ✅

---

## Step 6: Static Code Review

### 6a. SKILL.md Quality

| Check | Pre-fix | Post-fix |
|-------|---------|----------|
| description ASCII-only | ✅ | ✅ |
| Trigger words (EN+CN) | ⚠️ EN only | ⚠️ EN only (P2) |
| "Do NOT use for" rule | ❌ missing | ✅ added |
| Parameter examples in commands | ✅ | ✅ |
| `--dry-run` position correct | ❌ after subcommand shown but arg was global | ✅ fixed |

### 6b. Code Quality

| Check | Result |
|-------|--------|
| Hardcoded addresses (sSOL mint, pool) | ✅ in config.rs — acceptable for Solana SPL tokens with stable addresses |
| Amount precision | ✅ UI units in, 1e9 divisor for lamports (correct) |
| onchainos command usage | ✅ uses `swap execute` for SOL→sSOL via Jupiter (correct approach) |
| base64 → base58 conversion | N/A (REST API path removed; Jupiter path used instead) |
| Error handling | ✅ no panics; structured JSON errors with exit code 1 |
| Dead code | ❌ `wallet_contract_call_solana` + 3 unused consts → **fixed** |
| Unused Cargo deps | ❌ `base64`, `bs58` unused → **fixed** |

---

## Issues Found & Fixed

### P1 — Fixed

**P1-1: `--dry-run` unusable after subcommand**
- **Description**: SKILL.md documented `solayer stake --amount 0.001 --dry-run` but clap only accepted `--dry-run` as a global flag (before the subcommand). Users following the docs would get `error: unexpected argument '--dry-run' found`.
- **Fix**: Added `--dry-run` as a per-subcommand flag to `Stake` and `Unstake` variants in `src/main.rs`. Both global and subcommand-level `--dry-run` are now accepted (OR'd together).
- **Files**: `src/main.rs`

**P1-2: Dead code and unused dependencies**
- **Description**: `wallet_contract_call_solana()` in `onchainos.rs` was dead (unused). Constants `SOLANA_CHAIN_ID`, `RESTAKING_PROGRAM`, `STAKE_POOL` in `config.rs` were unused. `base64` and `bs58` crate deps were unused.
- **Fix**: Removed the dead function, removed unused constants, removed unused Cargo deps.
- **Files**: `src/onchainos.rs`, `src/config.rs`, `Cargo.toml`

### P2 — Not Fixed (Informational)

**P2-1: No Chinese trigger words in SKILL.md**
- SKILL.md description is English-only. Comparable plugins (jito, etc.) include Chinese trigger phrases. Low priority since routing works well.
- **Recommendation**: Add CN triggers: `质押SOL`, `Solayer质押`, `获取sSOL`, `查询sSOL余额`, `Solayer收益`.

---

## SKILL.md Changes Applied

1. Added `"Do NOT use for: Jito staking, Marinade staking, general Solana DeFi not involving Solayer, non-Solana chains."` to description
2. Removed erroneous `[--chain 501]` from `rates` and `positions` command signatures (chain is not a subcommand arg)
3. Fixed `[--chain 501] [--dry-run]` → `[--dry-run]` in `stake` and `unstake` signatures
4. Bumped version to `0.1.1`

---

## Commits

| Commit | Message |
|--------|---------|
| `eae6ba8` | fix: solayer P1 audit fixes — --dry-run subcommand flag and dead code cleanup |
| `3049f3d` | chore: update solayer plugin.yaml source_commit to eae6ba84 |

Pushed to: `GeoGu360/onchainos-plugins` main  
plugin-store-community `feat/solayer` updated to `source_commit: 3049f3dce3f1e08609e80efa9fb74af08e52bfb8`
