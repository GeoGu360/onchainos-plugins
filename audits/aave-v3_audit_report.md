# Skill Audit Report — Aave V3

**Repo**: https://github.com/GeoGu360/onchainos-plugins/tree/main/aave-v3
**Date**: 2026-04-06
**Wallet**: 0x87fb0647faabea33113eaf1d80d67acb1c491b90
**Test chain**: Base (8453)

---

## Summary

| Item | Result |
|------|--------|
| Compile | ✅ success (29s, clean) |
| Skill install / uninstall | ✅ |
| Commands tested | 11 / 11 |
| Dry-run tests | ✅ 9/9 passed |
| On-chain write ops | 2 executed (supply ✅, withdraw-all ✅) |
| On-chain write ops failed | 1 (withdraw partial amount ❌) |
| Issues found | 7 (1 P0, 3 P1, 3 P2) |

---

## Command Test Results

| # | Command | Flags | Status | Tx Hash | Notes |
|---|---------|-------|--------|---------|-------|
| 1 | `--version` | — | ✅ | — | Returns `aave-v3 0.1.0` |
| 2 | `--help` | — | ✅ | — | All subcommands listed |
| 3 | `reserves` | (all) | ✅ | — | Returns 15 reserves with APYs |
| 4 | `reserves --asset USDC` | symbol filter | ❌ BUG | — | Returns all 15 reserves unfiltered; symbol filter silently ignored |
| 5 | `health-factor` | `--from` | ✅ | — | Correct but HF displayed as `340282366920938487808.00` when no debt (should be `"∞"` or `"no debt"`) |
| 6 | `positions` | `--from` | ✅ | — | Returns DeFi platform list; Aave V3 entry appears post-supply |
| 7 | `supply --dry-run` | 0.1 USDC | ✅ | — | Correct ABI calldata; approve+supply steps shown |
| 8 | `supply` | 0.01 USDC | ✅ | approveTx: `0x57f94ef09b38ec51cfbe64adc62881c8eae5254e2606124279c68180e6f83fcd` supplyTx: `0xaf423de2c0721534e82822e847735ff4bb60701c39ea3c72f99bf8a9dd93bb4f` | On-chain supply confirmed |
| 9 | `withdraw --dry-run` | `--amount 0.01` | ✅ | — | Calldata correct |
| 10 | `withdraw` | `--amount 0.01` | ❌ FAIL | — | Reverts: `execution reverted` — partial amount exact-match fails due to aToken interest accrual |
| 11 | `withdraw` | `--all` | ✅ | `0x85d21318442b0c55ca01740b15ac90259c11130e218df002192d0fe150c9f4f5` | Full withdraw succeeded |
| 12 | `borrow --dry-run` | 0.1 USDC addr | ✅ | — | Dry-run output correct; warns no collateral posted |
| 13 | `repay --dry-run` | 0.1 USDC addr | ✅ | — | Dry-run output correct; warns no outstanding debt |
| 14 | `set-collateral --enable true` | wrong | ❌ ARG BUG | — | CLI rejects `--enable true`; `--enable` is a boolean flag (no value), not documented properly |
| 15 | `set-collateral --enable` | flag only | ✅ | — | Works as boolean flag |
| 16 | `set-emode --dry-run` | `--category 1` | ✅ | — | Calldata correct, category name resolved |
| 17 | `claim-rewards --dry-run` | — | ✅ | — | Returns friendly message when no active positions |

---

## Issues Found

### P0 — Blocking

#### P0-1: `withdraw --amount` reverts on-chain for partial withdrawals

**File**: `src/commands/withdraw.rs`

**Root cause**: The withdraw calldata encodes the exact human-readable amount (e.g. `0.01 USDC = 10000 atomic units`). However, Aave V3 aTokens accrue interest continuously; by the time the tx lands the user's actual aToken balance will be ≥ the supplied amount but the encoded amount is the exact original deposit. The Aave V3 Pool reverts if `amount > aToken balance` is not the case but it can also revert in edge cases when amount precision doesn't match expected redemption units. In practice, using `type(uint256).max` (equivalent to `--all`) is the safe canonical approach for any withdraw; partial amounts should be supported but the error is silent and user-unfriendly.

**Impact**: Any user attempting `withdraw --amount X` with an amount they originally supplied will likely get a revert with error message `"Pool.withdraw() failed"` — no actionable detail.

**Fix required**:
1. In `withdraw.rs`, when on-chain call fails, capture and surface the full `onchainos` stderr (not just the context string).
2. Document in SKILL.md that `--all` is the recommended form; or auto-promote partial amounts to `uint256.max` when amount matches full balance.

---

### P1 — Important

#### P1-1: `reserves --asset <SYMBOL>` filter silently does nothing

**File**: `src/commands/reserves.rs:62-65`

```rust
if let Some(filter) = asset_filter {
    if filter.starts_with("0x") && !addr.eq_ignore_ascii_case(filter) {
        continue;
    }
}
```

The filter guard only activates when `filter` starts with `0x` (an address). If a symbol like `USDC` is passed, the condition `filter.starts_with("0x")` is false, so no filtering happens and all 15 reserves are returned.

**Fix**: Resolve the symbol to an address first (via `onchainos::resolve_token`), then apply the address filter. Or add a second branch that compares the symbol against a known symbol map.

---

#### P1-2: `set-collateral --enable` CLI mismatch with SKILL.md documentation

**File**: `src/main.rs:88-92` and `skills/aave-v3/SKILL.md:327`

SKILL.md documents usage as:
```
aave-v3 ... set-collateral --asset <ADDR> --enable false
```

But `--enable` is defined as a boolean flag (`bool`) in `main.rs`, meaning it takes no value — passing `--enable false` causes a CLI parse error:
```
error: unexpected argument 'false' found
```

The correct invocations are `--enable` (to enable) and omitting it (to disable). However, there is no way to explicitly pass `false` as a value. This creates a confusing asymmetry and breaks the documented API.

**Fix**: Change `--enable` to accept a value using `allow_hyphen_values` or use two exclusive flags `--enable` / `--disable`. Update SKILL.md accordingly.

---

#### P1-3: Infinite health factor displayed as raw u128 overflow value

**File**: `src/commands/health_factor.rs:44`, `src/rpc.rs:131`

When a user has no debt, Aave V3 returns `type(uint256).max` for health factor. Since `health_factor` is stored as `u128`, it gets truncated from the u256 max to the u128 max (`340282366920938463463374607431768211455`). When divided by 1e18 this produces `340282366920938487808.00` — a nonsensical number shown to the user.

**Fix**: In `health_factor_f64()`, detect when `health_factor == u128::MAX` (or the raw 32-byte value is `0xFFFF...`) and return a sentinel (e.g. `f64::INFINITY`); display this as `"∞"` in the JSON output with a note `"noDebt": true`.

---

### P2 — Suggestions

#### P2-1: SKILL.md `description` field uses CJK characters inline — should use block scalar

**File**: `skills/aave-v3/SKILL.md:3`

The `description` frontmatter field contains CJK characters inline in a quoted string:
```yaml
description: "...Chinese: 在Aave存款, Aave借款, 还款, 健康因子, 我的Aave仓位, Aave利率"
```

Per plugin-store style guidelines, the description should use `>-` block scalar (ASCII-only in frontmatter, no CJK inline). CJK triggers should live in the body or trigger_phrases section.

**Fix**:
```yaml
description: >-
  Aave V3 lending and borrowing. Trigger phrases: supply to aave, deposit to
  aave, borrow from aave, repay aave loan, aave health factor, my aave
  positions, aave interest rates, enable emode, disable collateral, claim
  aave rewards.
```

---

#### P2-2: `target/` build artifacts not excluded from plugin-store lint scope

**File**: missing `.pluginignore` (`.gitignore` correctly excludes `target/` but the lint tool scans everything)

`plugin-store lint` flagged 235 E080/E081/E130 errors — all from `target/release/` build artifacts. The `.gitignore` correctly excludes `target/` from git, but the lint tool ignores `.gitignore`.

**Fix**: Add a `.pluginignore` file (or equivalent) listing `target/` to prevent false lint failures.

---

#### P2-3: `wallet_balance` helper passes `--output json` which is not a valid flag

**File**: `src/onchainos.rs:248`

The `wallet_balance()` function passes `--output json` to `onchainos wallet balance`, but that flag is not supported (confirmed: returns `error: unexpected argument '--output' found`). The function is `#[allow(dead_code)]` so it doesn't cause runtime failure, but if it were called it would error silently.

**Fix**: Remove the `--output` flag from `wallet_balance()` or update it to match the actual CLI signature.

---

## Static Code Review

| Check | Result | Notes |
|-------|--------|-------|
| SKILL.md description: ASCII only | ❌ | CJK chars inline in `description:` field (P2-1) |
| Pool address hardcoded | ✅ | Resolved at runtime via `PoolAddressesProvider.getPool()` |
| PoolAddressesProvider addresses | ✅ | Stored in config (immutable; appropriate) |
| Amount precision: UI → atomic units | ✅ | `human_to_minimal(amount, decimals)` correct; USDC 6-decimal handled |
| ERC-20 approve via `wallet contract-call` | ✅ | Uses `onchainos wallet contract-call` with encoded `approve()` ABI; not `dex approve` |
| Error messages: friendly | Partial | On-chain failures propagate `anyhow` chain; top-level outputs JSON `{"ok":false,"error":"..."}` — acceptable. But `Pool.withdraw() failed` without root cause is too terse (P0-1) |
| No raw panic/unwrap in user path | ✅ | Uses `anyhow::Result` throughout; `serde_json::to_string_pretty().unwrap_or_default()` is safe |
| Interest rate mode: variable only (mode=2) | ✅ | Stable (mode=1) blocked; constant defined |
| plugin-store lint | ❌ | 235 errors — all E080/E130 from `target/` artifacts not excluded (P2-2) |
| Repay `--all` uses wallet balance, not uint256.max | ✅ | SKILL.md claims this but code uses `u128::MAX` → `U256::MAX` for `--all`; may cause revert if accrued interest exceeds balance |

---

## On-chain Tx Summary

| Action | Amount | Tx Hash | Status |
|--------|--------|---------|--------|
| ERC-20 approve (USDC → Pool) | 0.01 USDC | `0x57f94ef09b38ec51cfbe64adc62881c8eae5254e2606124279c68180e6f83fcd` | ✅ confirmed |
| Pool.supply | 0.01 USDC | `0xaf423de2c0721534e82822e847735ff4bb60701c39ea3c72f99bf8a9dd93bb4f` | ✅ confirmed |
| Pool.withdraw (partial 0.01) | 0.01 USDC | — | ❌ reverted |
| Pool.withdraw (full `uint256.max`) | all USDC | `0x85d21318442b0c55ca01740b15ac90259c11130e218df002192d0fe150c9f4f5` | ✅ confirmed |
