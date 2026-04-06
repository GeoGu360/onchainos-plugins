# Skill Audit Report — USDe Staking (Ethena sUSDe)

**Repo**: https://github.com/GeoGu360/onchainos-plugins (path: `usde-staking/`)
**Audit date**: 2026-04-06
**Auditor**: skill-auditor agent (claude-sonnet-4-6)
**Test wallet**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90` (EVM, Ethereum mainnet)
**Test chain**: Ethereum mainnet (chain ID 1) — only supported chain
**Binary**: `usde-staking` (Rust, async tokio)

---

## Summary

| Item | Result |
|------|--------|
| Compilation | ✅ Success (clean after fixes) |
| Commands tested | 5 / 5 |
| Read commands passing | 2 / 2 (live on-chain) |
| Dry-run commands passing | 3 / 3 |
| Live write operations | 0 — no USDe in test wallet (dry-run only) |
| ABI selectors verified | 11 / 11 correct |
| Bugs found | 5 |
| Bugs fixed | 5 |

**Overall assessment**: The plugin is well-structured and functionally correct. All five bugs are now fixed. Write operations were tested in dry-run only (wallet holds 0 USDe; staking requires USDe which is Ethena's synthetic dollar not obtainable without a swap first). Read operations work correctly against live contracts.

---

## Test Plan

| # | Command | Type | Key Params | Test Input |
|---|---------|------|-----------|-----------|
| 1 | `get-rates` | Read (API + on-chain) | none | - |
| 2 | `get-positions` | Read (on-chain) | `--address` | test wallet |
| 3 | `stake` | Write (approve + deposit) | `--amount`, `--dry-run` | dry-run only |
| 4 | `request-unstake` | Write (cooldown) | `--shares`/`--assets`, `--dry-run` | dry-run only |
| 5 | `claim-unstake` | Write (claim) | `--dry-run` | dry-run only |

---

## Command Test Results

| # | Command | Status | Tx Hash | On-chain Confirm | Notes |
|---|---------|--------|---------|-----------------|-------|
| 1 | `get-rates` | ✅ | - | - | APY 3.49%, TVL 3.52B USDe, exchange rate 1.226277 USDe/sUSDe |
| 2 | `get-positions --address 0x87fb...` | ✅ | - | - | 0 USDe, 0 sUSDe, no pending unstake — correct |
| 3 | `stake --amount 10.0 --dry-run` | ✅ dry-run | - | - | Correct approve + deposit calldata generated |
| 4 | `request-unstake --shares 5.0 --dry-run` | ✅ dry-run | - | - | Correct cooldownShares calldata |
| 4b | `request-unstake --assets 5.0 --dry-run` | ✅ dry-run | - | - | Correct cooldownAssets calldata |
| 5 | `claim-unstake --dry-run` | ✅ dry-run | - | - | Correct unstake(receiver) calldata |

**Error handling tests:**
- `stake --amount 999999` → ✅ friendly error: "Insufficient USDe balance. Have 0.000000 USDe, need 999999.000000 USDe."
- `request-unstake --shares 999999` → ✅ friendly error: "Insufficient sUSDe balance. Have 0.000000 sUSDe, need 999999.000000 sUSDe."
- `stake --amount 0.0` → ✅ error: "Stake amount must be greater than 0"

**Write operation note**: Test wallet has ~0.197 ETH and ~15 USDT on Ethereum but 0 USDe. Since staking requires USDe (Ethena's synthetic dollar — not obtainable via simple ETH spend), all write commands were tested dry-run only. Dry-run outputs show correct calldata for approve, deposit, cooldownShares, cooldownAssets, and unstake.

---

## ABI / Selector Verification

All 11 selectors verified with `cast sig`:

| Function | Expected | Source | Match |
|----------|----------|--------|-------|
| `approve(address,uint256)` | `0x095ea7b3` | `0x095ea7b3` | ✅ |
| `deposit(uint256,address)` | `0x6e553f65` | `0x6e553f65` | ✅ |
| `cooldownShares(uint256)` | `0x9343d9e1` | `0x9343d9e1` | ✅ |
| `cooldownAssets(uint256)` | `0xcdac52ed` | `0xcdac52ed` | ✅ |
| `unstake(address)` | `0xf2888dbb` | `0xf2888dbb` | ✅ |
| `balanceOf(address)` | `0x70a08231` | `0x70a08231` | ✅ |
| `convertToAssets(uint256)` | `0x07a2d13a` | `0x07a2d13a` | ✅ |
| `cooldowns(address)` | `0x01320fe2` | `0x01320fe2` | ✅ |
| `cooldownDuration()` | `0x35269315` | `0x35269315` | ✅ |
| `totalAssets()` | `0x01e1d114` | `0x01e1d114` | ✅ |
| `previewDeposit(uint256)` | `0xef8b30f7` | `0xef8b30f7` | ✅ |

---

## Bugs Found and Fixed

### P1 — `extract_tx_hash` silently returned `"pending"` on missing hash

**File**: `src/onchainos.rs:141`
**Problem**: `extract_tx_hash` returned the string `"pending"` when no hash was present. This caused the binary to print `"Deposit tx: pending"` and continue as if the transaction succeeded, masking failures silently.
**Fix**: Changed return type to `anyhow::Result<String>`; now bails with a descriptive error if hash is empty or `"pending"`. All callers updated to propagate with `?`.
**Status**: ✅ Fixed — commit `8dd08f9`

---

### P1 — `wallet_contract_call` did not check exit code or `ok` field

**File**: `src/onchainos.rs:106`
**Problem**: After calling `onchainos wallet contract-call`, the code parsed stdout and returned whatever JSON was there without verifying `output.status.success()` or `result["ok"] == true`. A failed onchainos call could return `{"ok": false, ...}` and the plugin would proceed to extract a tx hash and report success.
**Fix**: Added explicit checks — bail if `output.status` is non-zero (stderr included in error) or if `result["ok"] != true`.
**Status**: ✅ Fixed — commit `8dd08f9`

---

### P2 — `source_commit` was all-zeros placeholder

**File**: `plugin.yaml:14`
**Problem**: `source_commit: "0000000000000000000000000000000000000000"` — placeholder never replaced with actual commit hash.
**Fix**: Updated to `9204514fdf7b5179e5f3f3483d06b431ed7bafe7` (HEAD at audit time).
**Status**: ✅ Fixed — commit `8dd08f9`

---

### P2 — Unused structs `YieldEntry` / `YieldResponse` in `get_rates.rs`

**File**: `src/commands/get_rates.rs:6-23`
**Problem**: Two `#[derive(Deserialize, Debug)]` structs were defined but never instantiated — the code uses raw `serde_json::Value` access instead. Generated compiler warnings.
**Fix**: Removed both structs and unused `serde::Deserialize` import.
**Status**: ✅ Fixed — commit `8dd08f9`

---

### P2 — Unused constants in `config.rs`

**File**: `src/config.rs`
**Problem**: Three constants were defined but never referenced:
- `TOKEN_DECIMALS: u32 = 18`
- `SEL_CONVERT_TO_SHARES: &str = "c6e6f592"`
- `SEL_PREVIEW_REDEEM: &str = "4cdad506"`

These generated compiler dead-code warnings.
**Fix**: Removed all three unused constants.
**Status**: ✅ Fixed — commit `8dd08f9`

---

## SKILL.md Quality

### Before audit
- Description was minimal one-liner (ASCII-only — good)
- No trigger phrases for common user intents
- No "Do NOT use for" disambiguation rules
- No `>-` block scalar (single line, no examples of alternate phrasings)

### After audit (fixed)
- Expanded to `>-` block with explicit trigger phrases in both English variants
- Added "Do NOT use for" rules covering: dex swaps, bridging, non-USDe tokens, non-Ethereum chains
- All trigger phrases are ASCII-only
- **Status**: ✅ Fixed — commit `8dd08f9`

---

## Code Quality Notes

### Strengths
- Architecture is clean: read ops use direct `eth_call`, write ops go through `onchainos wallet contract-call`
- `dry_run` mode is implemented consistently across all three write commands
- Error messages are user-friendly throughout (no raw panics, no bare RPC codes)
- Amount precision: `u128` used for all wei amounts (correct — no truncation risk)
- 2-transaction stake flow (approve → 15s wait → deposit) is properly sequenced
- `cooldowns(address)` return value correctly decoded as two uint256 (cooldown_end, underlying_amount)
- Cooldown gating in `claim-unstake` correctly aborts if cooldown not yet complete

### Minor notes (no fix required)
- `approve_calldata` in dry-run output contains mixed-case address hex (cosmetic only; EVM accepts both)
- The 15-second hardcoded wait between approve and deposit is a reasonable heuristic but may occasionally be too short on congested mainnet; not a blocker

---

## Compilation Summary

**Before fixes**: 5 warnings (2 dead structs, 3 unused constants)
**After fixes**: 0 warnings — clean build

```
Finished `release` profile [optimized] target(s) in 1.32s
```

---

## Commit History

| Commit | Description |
|--------|-------------|
| `8dd08f9` | fix(usde-staking): extract_tx_hash Result, ok-check, dead code, SKILL.md quality |

---

## Re-audit — 2026-04-06 (Live Write Verified)

**Context**: Initial audit was dry-run only because the test wallet had 0 USDe. Wallet now holds 2.155 USDe on Ethereum mainnet.

**Re-audit date**: 2026-04-06
**Test wallet**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
**Pre-conditions**: ETH 0.1747, USDe 2.155, USDT 15.03 on Ethereum mainnet

### Pre-flight Baseline

```
USDe (unstaked): 2.155439 USDe
sUSDe (staked):  0.000000 sUSDe
Pending Unstake: None
```

`get-rates`: APY 3.49%, TVL 3.52B USDe, exchange rate 1 sUSDe = 1.226285 USDe

### Live Write Results

| # | Command | Status | Tx Hash | On-chain Confirm | Notes |
|---|---------|--------|---------|-----------------|-------|
| 1 | `stake --amount 0.1` (Step 1: approve) | ✅ | `0x3d18550cc33f2dc4a30c9341c536d9543224c658ab6df59ef545e6902f7f772b` | ✅ status=1 block 24820162 | USDe approve to sUSDe vault |
| 2 | `stake --amount 0.1` (Step 2: deposit) | ✅ | `0x8acfc9e1d810b984d9cd7b2e0ee40ce49d29eb6a903db3bf56bb2e7dc6efa9ea` | ✅ status=1 block 24820164 | 0.1 USDe → 0.081547 sUSDe |
| 3 | `request-unstake --shares 0.081547` | ✅ | `0x17ba416671e37492f59c435f4447bd9c71d1c50a3c4501e53c51448c287cc85f` | ✅ status=1 block 24820166 | Cooldown initiated |

### Post-operation State Verification

```
USDe (unstaked): 2.055439 USDe   (was 2.155439 — delta: -0.1 ✅)
sUSDe (staked):  0.000000 sUSDe  (in cooldown after request-unstake)
Pending Unstake: 0.100000 USDe, cooldown ends in ~24h (status: COOLING DOWN)
```

### New Bugs Found

None. All fixes from the initial audit hold. The two-tx stake flow (approve → 15s wait → deposit) performed correctly under live conditions. `extract_tx_hash` correctly returned valid hashes for all three transactions. `wallet_contract_call` exit-code and `ok` field checks did not trigger (all calls succeeded).

### claim-unstake

Not tested in this re-audit — cooldown period is 1 day. The sUSDe is now in the cooldown queue; `claim-unstake` can be verified 24h from now.

### Re-audit Summary

| Item | Result |
|------|--------|
| Live stake (approve) | ✅ on-chain confirmed block 24820162 |
| Live stake (deposit) | ✅ on-chain confirmed block 24820164 |
| Live request-unstake | ✅ on-chain confirmed block 24820166 |
| State change verified | ✅ USDe -0.1, sUSDe in cooldown |
| New bugs found | 0 |
| Overall verdict | **PASS — production ready** |
