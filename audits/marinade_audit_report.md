# Skill Audit Report — Marinade Finance

**Source**: `/tmp/onchainos-plugins/marinade/`
**Audit Date**: 2026-04-06
**Auditor**: skill-auditor workflow v1.0.0
**Test Wallet (Solana)**: `DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE`
**Test Chain**: Solana (501)
**SOL Balance at Start**: ~0.0598 SOL (~$4.90)

---

## Summary

| Item | Result |
|------|--------|
| Compile | ✅ (3 unused-constant warnings) |
| Skill Install (`npx skills add`) | ✅ |
| Commands Tested | 7 / 7 |
| Read-only Commands Passed | 2 / 2 |
| Dry-run Commands Passed | 2 / 2 |
| On-chain Write Commands Passed | 2 / 2 |
| Error Handling Tests | 1 / 1 (exposes P0 bug) |
| plugin-store lint (clean tree) | ✅ pass (1 trivial warning) |
| Skill Uninstall | ✅ |
| Issues Found | 4 (1× P0, 1× P1, 2× P2) |

---

## Test Plan

| # | Command | Type | Key Params | Test Input |
|---|---------|------|------------|------------|
| 1 | `marinade rates` | Read-only | none | — |
| 2 | `marinade positions` | Read-only | none | — |
| 3 | `marinade --dry-run stake` | Dry-run | --amount | 0.001 SOL |
| 4 | `marinade --dry-run unstake` | Dry-run | --amount | 0.0001 mSOL |
| 5 | `marinade stake` | On-chain write | --amount | 0.001 SOL |
| 6 | `marinade unstake` | On-chain write | --amount | 0.0002 mSOL |
| 7 | `marinade stake --amount 999999` | Error handling | --amount | 999999 SOL |

---

## Command Test Results

| # | Command | Status | Tx Hash | Notes |
|---|---------|--------|---------|-------|
| 1 | `marinade rates` | ✅ | — | msol_per_sol=1.3719, APY=~7%, supply=2,030,641 mSOL |
| 2 | `marinade positions` | ✅ | — | msol_balance=0.00023951, wallet resolved correctly |
| 3 | `marinade --dry-run stake --amount 0.001` | ✅ | — | Dry-run returns correct from/to mints |
| 4 | `marinade --dry-run unstake --amount 0.0001` | ✅ | — | Dry-run returns correct from/to mints |
| 5 | `marinade stake --amount 0.001` | ✅ | `3sudS3v8zCEAtp72i6z2fj8g85wM4LVAKfW7FQRPT3dRT5gLqS5ctYZkptY8RjyYW6nvweoZsoWFxwYu4aH1tTH9` | Staked 0.001 SOL → +0.000733 mSOL; balance confirmed via positions query |
| 6 | `marinade unstake --amount 0.0002` | ✅ | `pxFJKsWc1xcLn2C8hjBYs2yLve5AyECWB9tk1qC3e29mqSNkWZTE7vuYDBWH4GeiG2RAAfaxtydt2t8shkD53iq` | Unstaked 0.0002 mSOL → ~0.000275 SOL; balance confirmed via positions query |
| 7 | `marinade stake --amount 999999` | ❌ (P0) | — | Returns `ok: true` + exit 0 despite `onchainos` reporting `ok: false`; raw.error = "transaction simulation failed: InstructionError[2]: {\"Custom\":1}" |

### Pre/Post State Comparison (Stake)

| | mSOL Balance | SOL Equivalent |
|-|-------------|----------------|
| Before stake | 0.00023951 | 0.000329 SOL |
| After stake (+0.001 SOL) | 0.000972471 | 0.001334 SOL |
| Delta | +0.000733 mSOL | — |

### Pre/Post State Comparison (Unstake)

| | mSOL Balance |
|-|-------------|
| Before unstake | 0.000972471 |
| After unstake (−0.0002 mSOL) | 0.000772471 |
| Delta | −0.0002 mSOL ✅ |

---

## Issues Found

### P0 — Blocking Issues (Functionality Broken)

#### P0-1: Silent error swallowing on failed on-chain tx

**File**: `src/commands/stake.rs:24–38`, `src/commands/unstake.rs:24–38`

**Description**: When `onchainos swap execute` returns `{"ok": false, "error": "..."}`, the plugin does not check this field. It unconditionally calls `extract_tx_hash(&result)` (which returns `"pending"` for missing hashes) and outputs `{"ok": true, ...}` with exit code 0.

**Reproduction**:
```bash
./target/release/marinade stake --amount 999999
```
Output shows `"ok": true` and `"txHash": "pending"` while `raw.error` contains the actual error.

**Expected behavior**: Check `result["ok"].as_bool() == Some(false)` after `swap_execute`; if so, propagate the error via `anyhow::bail!` so the `main` error handler outputs `{"ok": false, "error": "..."}` with exit code 1.

**Suggested fix** (stake.rs and unstake.rs):
```rust
let result = onchainos::swap_execute(SOL_NATIVE, MSOL_MINT, amount, &slippage_str, false).await?;
if result["ok"].as_bool() != Some(true) {
    let msg = result["error"].as_str().unwrap_or("swap failed");
    anyhow::bail!("{}", msg);
}
let tx_hash = onchainos::extract_tx_hash(&result);
```

---

### P1 — Important Issues (User Experience)

#### P1-1: `--dry-run` flag position mismatch with SKILL.md documentation

**File**: `src/main.rs:16` (Cli struct), `skills/marinade/SKILL.md`

**Description**: SKILL.md documents `--dry-run` as a per-subcommand flag:
```
marinade stake --amount 0.001 --dry-run
```
But the binary only accepts `--dry-run` as a global flag before the subcommand:
```
marinade --dry-run stake --amount 0.001
```
Running `marinade stake --amount 0.001 --dry-run` fails with:
```
error: unexpected argument '--dry-run' found
```

**Suggested fix**: Either move `dry_run` into each subcommand's argument struct (preferred for clarity), or update SKILL.md documentation to show the correct global flag position.

---

### P2 — Improvement Suggestions

#### P2-1: CJK characters embedded in SKILL.md `description` field

**File**: `skills/marinade/SKILL.md:3`

**Description**: The `description` field contains Chinese characters (`质押SOL获取mSOL, 查询mSOL余额, 解质押mSOL, Marinade质押利率`). Per the audit standard, the `description` field should be ASCII-only. Chinese trigger phrases should be placed in a dedicated `triggers` section or in the command-level trigger phrases, not in the front-matter description.

**Suggested fix**: Remove CJK from the `description` front-matter value. Keep Chinese trigger phrases only in the `## Commands` section under each command's "Trigger phrases" list.

#### P2-2: Unused constants and placeholder `source_commit` in plugin.yaml

**Files**: `src/config.rs:4,13,16` and `plugin.yaml`

**Description**:
1. Three constants are defined but never used: `SOLANA_CHAIN_ID`, `MARINADE_PROGRAM_ID`, `MARINADE_STATE_ACCOUNT`. The compiler emits dead-code warnings. These constants suggest the author may have intended a direct Marinade protocol integration (rather than the current Jupiter-only routing). They should either be used or removed.
2. `plugin.yaml` `source_commit` is set to all zeros (`0000000000000000000000000000000000000000`), which is a placeholder. This should be the real git commit SHA when submitting to the plugin store.

#### P2-3: No `Do NOT use for` disambiguation rules in SKILL.md

**File**: `skills/marinade/SKILL.md`

**Description**: The SKILL.md has no "Do NOT use for" rules. Without disambiguation, an agent may trigger the marinade skill when the user asks about other Solana LSTs (JitoSOL, sSOL) or other staking protocols. Recommended additions:
```
Do NOT use for: Jito staking (use jito skill), Solayer staking, non-Solana staking, EVM staking protocols.
```

---

## SKILL.md Quality Assessment

| Check | Result | Notes |
|-------|--------|-------|
| description ASCII-only | ❌ | Contains CJK trigger phrases — see P2-1 |
| Trigger phrases cover English | ✅ | Good coverage |
| Trigger phrases cover Chinese | ✅ | Chinese phrases present (but in wrong location) |
| "Do NOT use for" rules | ❌ | Missing entirely — see P2-3 |
| Each command has param examples | ✅ | Yes, all 4 commands have examples |
| Amount unit documented clearly | ✅ | SOL and mSOL units documented per command |
| Execution flow documented | ✅ | Both stake/unstake show 4-step flow |

---

## Code Quality Assessment

| Check | Result | Notes |
|-------|--------|-------|
| Hardcoded contract addresses | Acceptable | mSOL mint and SOL native are fixed protocol constants; documented in config.rs with comments |
| Amount precision handling | ✅ | Uses `--readable-amount` (human units) passed directly to onchainos; no raw lamport conversion needed |
| onchainos command usage | ✅ | Uses `onchainos swap execute --chain 501` correctly for SOL/mSOL swaps |
| Solana base64→base58 conversion | N/A | Plugin does not handle raw transactions; delegates entirely to onchainos |
| Error messages user-friendly | Partial | clap errors are clear; but on-chain errors silently wrapped (P0-1) |
| No unsafe unwrap panics | ✅ | Only `unwrap()` in main.rs is on `serde_json::to_string_pretty` which is infallible for valid JSON |
| Wallet address resolution | ✅ | Correctly reads from `onchainos wallet balance --chain 501` → `data.details[0].tokenAssets[0].address` |

---

## Static Analysis: Address Verification

| Address | Purpose | Verified |
|---------|---------|---------|
| `mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So` | mSOL mint | ✅ Confirmed correct Marinade mSOL mint on Solana mainnet |
| `11111111111111111111111111111111` | Native SOL (System Program) | ✅ Standard Solana System Program address used by Jupiter as SOL placeholder |
| `MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD` | Marinade liquid staking program | ✅ Correct Marinade Finance program ID (unused in current Jupiter-routing implementation) |
| `8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC` | Marinade state account | ✅ Correct Marinade state account (unused in current Jupiter-routing implementation) |

---

## plugin-store Lint Results

```
plugin-store lint /tmp/marinade-clean (target/ excluded)
✓ Plugin 'marinade' passed with 1 warning(s)
  W120: directory name 'marinade-clean' does not match plugin name 'marinade'
  (artifact of audit copy — not present in real submission)
```

**Result**: PASS on clean source tree.

---

## Compile Warnings

```
warning: constant `SOLANA_CHAIN_ID` is never used   (src/config.rs:4)
warning: constant `MARINADE_PROGRAM_ID` is never used (src/config.rs:13)
warning: constant `MARINADE_STATE_ACCOUNT` is never used (src/config.rs:16)
```

All three are dead-code warnings for constants that appear intended for future direct-protocol integration.

---

## Overall Assessment

The marinade plugin is **functionally sound for the happy path**. Both read-only queries and on-chain swap operations (stake/unstake) work end-to-end with correct state changes confirmed on-chain. The Jupiter-based routing approach is appropriate and robust.

The **single blocking issue (P0-1)** — silent error swallowing when `onchainos swap execute` fails — must be fixed before production use. An on-chain tx failure currently appears as success to the calling agent, which could cause the agent to report a false successful stake/unstake.

The **P1 flag placement mismatch** is a documentation/UX inconsistency that will confuse users attempting to use `--dry-run` per the documented syntax.

**Recommended action**: Fix P0-1 and P1-1, clean up unused constants, then the plugin is ready for plugin-store submission.

---

*Report generated by skill-auditor workflow — 2026-04-06*
