# Beefy Finance Skill Audit Report

**Date**: 2026-04-06  
**Auditor**: skill-auditor (Claude Sonnet 4.6)  
**Plugin path**: /tmp/onchainos-plugins/beefy  
**Plugin version**: 0.1.0  
**Wallet (EVM)**: 0x87fb0647faabea33113eaf1d80d67acb1c491b90  

---

## 1. Build

```
cargo build --release
```

Result: SUCCESS with 3 dead_code warnings (see Bug #4).

---

## 2. Test Plan (from SKILL.md)

| Command | Mode | Result |
|---------|------|--------|
| `vaults --chain 8453 --limit 5` | read | PASS |
| `vaults --chain 8453 --asset USDC --limit 3` | read | PASS |
| `apy --chain 8453 --asset USDC --limit 5` | read | PASS |
| `apy --chain 8453 --vault morpho-base-gauntlet-prime-usdc` | read | PASS |
| `positions --chain 8453 --wallet 0x87fb...` | read | PASS (after fix) |
| `deposit --vault ... --amount 0.01 --chain 8453 --dry-run` | dry-run | PASS |
| `withdraw --vault ... --chain 8453 --dry-run` | dry-run | PASS |
| `deposit --vault morpho-base-gauntlet-prime-usdc --amount 0.01 --chain 8453` | write | PASS |
| `withdraw --vault morpho-base-gauntlet-prime-usdc --chain 8453` | write | PASS |

### Write Transaction Receipts

- **Deposit**: `0x0def48869670f9fb06cea0b0c746f0d55774d1c0e5375296b18a09471330d5ea`  
  - Block: 0x2a47c64, status: **0x1 (success)**  
  - https://basescan.org/tx/0x0def48869670f9fb06cea0b0c746f0d55774d1c0e5375296b18a09471330d5ea

- **Withdraw**: `0xffd5a591622893a34d5fc491895eee6a87bbbaf782f9f94097db2e62032d9e68`  
  - status: **0x1 (success)**  
  - https://basescan.org/tx/0xffd5a591622893a34d5fc491895eee6a87bbbaf782f9f94097db2e62032d9e68

---

## 3. Bugs Found and Fixed

### Bug #1 (HIGH): positions cap=50 misses most vaults
**File**: `src/commands/positions.rs`  
**Problem**: `active_vaults.len().min(50)` limits vault scan to 50. Base chain has 248+ active vaults. The test vault `morpho-base-gauntlet-prime-usdc` is at index 190, so it was never checked — `positions` returned empty even after a successful deposit.  
**Fix**: Raised cap to 300.

### Bug #2 (MEDIUM): extract_tx_hash returns String instead of Result
**File**: `src/onchainos.rs`  
**Problem**: `extract_tx_hash` returned `String`, silently returning `"pending"` on missing hash. This hides RPC failures and violates the audit rule requiring `Result` return.  
**Fix**: Changed return type to `anyhow::Result<String>`, returns error when hash not found. Updated call sites in `deposit.rs` and `withdraw.rs` to propagate with `?`.

### Bug #3 (MEDIUM): withdraw --shares parameter inconsistency
**File**: `src/commands/withdraw.rs` + `skills/beefy/SKILL.md`  
**Problem**: SKILL.md example shows `--shares 0.5` (human-readable float) but code expected raw integer (e.g. `9927`). User following the docs would get an error.  
**Fix**: Changed code to parse `--shares` as human-readable float and convert to raw units using token decimals. Now `0.5` means "0.5 mooTokens".

### Bug #4 (LOW): dead_code warnings for unused utility functions
**Files**: `src/onchainos.rs`, `src/rpc.rs`  
**Problem**: `wallet_contract_call`, `get_total_supply`, `get_vault_balance` are defined but not used, generating compiler warnings.  
**Fix**: Added `#[allow(dead_code)]` to reserved utility functions.

### Bug #5 (LOW): SKILL.md missing "Do NOT use for" section
**File**: `skills/beefy/SKILL.md`  
**Problem**: No boundary/exclusion section, violating audit requirements.  
**Fix**: Added "Do NOT use for" section.

---

## 4. Static Analysis

| Check | Result |
|-------|--------|
| extract_tx_hash returns Result | FIXED (was String) |
| ok-check on onchainos responses | PASS (deposit/withdraw check ok: true) |
| SKILL.md ASCII-only | PASS (no non-ASCII bytes found) |
| Do NOT use for section | FIXED (added) |
| source_repo field | PASS (GeoGu360/onchainos-plugins matches git remote origin) |
| amount precision | PASS (f64->u128 via pow(decimals); USDC 6 dec: 0.01 -> 10000 raw verified) |
| Vault selector correctness | PASS: deposit=0xb6b55f25, withdraw=0x2e1a7d4d, approve=0x095ea7b3 |
| Approve uses unlimited (u128::MAX) | PASS (avoids repeated approvals) |
| 15s delay after approve | PASS (ensures approve is mined before deposit) |
| dry_run never calls onchainos | PASS (early return before Command::new) |
| HTTPS_PROXY support | PASS (all HTTP clients respect env proxy) |

---

## 5. Commit

All fixes committed to monorepo main:

```
commit e6b05ca23667c2409cf1e598a4121ec8a4e71c00
fix(clanker): extract_tx_hash returns Result + ok-check + SKILL.md ASCII
(also includes all beefy fixes listed above)
```

Files changed:
- `beefy/.gitignore` (new: exclude target/)
- `beefy/skills/beefy/SKILL.md`
- `beefy/src/commands/deposit.rs`
- `beefy/src/commands/positions.rs`
- `beefy/src/commands/withdraw.rs`
- `beefy/src/onchainos.rs`
- `beefy/src/rpc.rs`

---

## 6. Summary

**Overall verdict**: PASS after fixes.

4 bugs fixed. All write operations confirmed on-chain with status=0x1. The skill correctly implements Beefy's BeefyVaultV7 interface (non-ERC4626), with proper approve+deposit flow and mooToken redemption.
