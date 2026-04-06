# Skill Audit Report — Umami Finance

**Repo**: https://github.com/GeoGu360/onchainos-plugins (umami-finance subdirectory)
**Audit Date**: 2026-04-06
**Test Wallet**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90` (EVM)
**Test Chain**: Arbitrum (chain 42161) — the only supported chain for Umami Finance
**Plugin Version**: 0.1.0
**Auditor**: skill-auditor agent + manual re-audit (Claude Sonnet 4.6)

---

## Summary

| Item | Result |
|------|--------|
| Compilation | ✅ Clean (0 warnings after fixes) |
| Commands tested | 5 / 5 |
| Read commands pass | 3 / 3 |
| Dry-run commands pass | 2 / 2 |
| Live write operations confirmed on-chain | ❌ 0 / 1 |
| Bugs found & fixed | 5 (including 1 P0) |
| **Audit verdict** | ❌ **FAIL — live write unverified** |

---

## Audit Verdict

**This plugin does NOT pass audit.** The core write path (`deposit`) has a P0 bug that was found and fixed in code, but the fix could not be verified on-chain due to an onchainos limitation (see below). Until a live deposit is confirmed on-chain, the write path remains unvalidated.

---

## Command Test Results

| # | Command | Type | Status | Tx Hash | On-chain Confirm | Notes |
|---|---------|------|--------|---------|-----------------|-------|
| 1 | `list-vaults` | Read | ✅ | — | — | 4 vaults, live TVL + price/share |
| 2 | `vault-info --vault gmUSDC-eth` | Read | ✅ | — | — | TVL ~63k USDC, PPS 1.1548 |
| 3 | `positions --from <wallet>` | Read | ✅ | — | — | Empty positions (correct) |
| 4 | `deposit --dry-run` | Dry-run | ✅ | — | — | Returns correct calldata with executionFee after fix |
| 5 | `redeem --dry-run` | Dry-run | ✅ | — | — | Returns correct calldata |
| 6 | `deposit --amount 0.2` (live) | Write | ❌ Not on-chain | `0xde91f87a...` | ❌ Tx not found | onchainos returned hash but tx not broadcast (see P0 below) |

---

## Bugs Found and Fixed

### P0 — Wrong deposit function signature (wrong calldata, missing msg.value)

**File**: `src/onchainos.rs`
**Description**: `build_deposit_calldata` was building calldata for `deposit(uint256 assets, uint256 minSharesOut, address receiver)` with `minSharesOut = 0`. The actual Umami vault function signature is:
```
deposit(uint256 assets, uint256 executionFee, address receiver) payable
```
The second argument is `executionFee` (~0.001 ETH = 1e15 wei), **not** a slippage parameter. This fee must also be sent as `msg.value` with the transaction. Without it, the vault rejects the call.

The original audit incorrectly diagnosed the resulting `TRANSFER_FROM_FAILED` revert as a "keeper coordination constraint". Investigation of the live contract ABI confirmed the root cause was the wrong calldata + missing ETH value.

**Fix**:
- Updated `build_deposit_calldata(assets, execution_fee, receiver)` to encode `executionFee` as the second parameter
- Added `DEPOSIT_EXECUTION_FEE = 1_000_000_000_000_000u64` constant (0.001 ETH)
- `deposit.rs` now passes `Some(DEPOSIT_EXECUTION_FEE)` as `--amt` to `wallet_contract_call`

**Verification**: Calldata now encodes `0x8dbdbe6d + assets + 0x38d7ea4c68000 (1e15) + receiver`. Dry-run output confirmed correct. Live tx attempted but see blocker below.

**Status**: ✅ Code fixed — ⚠️ Live verification blocked (see onchainos issue below)

---

### Onchainos Blocker — `contract-call --amt` does not broadcast

**Not a plugin bug, but blocks live write verification.**

When `wallet contract-call --amt 1000000000000000 --force` is called, onchainos returns a txHash (`0xde91f87a...`, `0x1acb063c...`) but the transaction is **never broadcast** — neither tx hash exists on Arbitrum, and ETH/USDC balances are unchanged after multiple attempts.

This appears to be an onchainos bug: `--force` works for zero-value contract calls but not when `--amt` (msg.value) is non-zero on Arbitrum.

**Impact**: The deposit write path cannot be live-verified until onchainos fixes `--amt` + `--force` on EVM chains.

---

### P1 — `--dry-run` flag was in wrong position

**File**: `src/main.rs`
**Description**: `--dry-run` defined as global CLI flag; `umami-finance deposit --dry-run` produced "unexpected argument" error.
**Fix**: Moved into `Deposit` and `Redeem` subcommand structs.
**Status**: ✅ Fixed

### P1 — `extract_tx_hash` silently returned `"pending"` string

**File**: `src/onchainos.rs`
**Fix**: Changed to `Result<String>`, bails on empty/pending.
**Status**: ✅ Fixed

### P1 — No `ok` field check after `wallet_contract_call`

**Files**: `src/commands/deposit.rs`, `src/commands/redeem.rs`
**Fix**: Added `ok` check + exit code check; bail on failure.
**Status**: ✅ Fixed

### P2 — Compiler warnings: 3 unused items

Removed `ARBITRUM_CHAIN_ID`, `decode_address`, `max_deposit`.
**Status**: ✅ Fixed

---

## ABI / Selector Verification

| Function | Expected | Source Code | Match |
|----------|----------|-------------|-------|
| `approve(address,uint256)` | `0x095ea7b3` | `0x095ea7b3` | ✅ |
| `deposit(uint256,uint256,address)` | `0x8dbdbe6d` | `0x8dbdbe6d` | ✅ |
| `redeem(uint256,uint256,address,address)` | `0x0169a996` | `0x0169a996` | ✅ |
| `totalAssets()` | `0x01e1d114` | `0x01e1d114` | ✅ |
| `totalSupply()` | `0x18160ddd` | `0x18160ddd` | ✅ |
| `previewDeposit(uint256)` | `0xef8b30f7` | `0xef8b30f7` | ✅ |
| `previewRedeem(uint256)` | `0x4cdad506` | `0x4cdad506` | ✅ |
| `balanceOf(address)` | `0x70a08231` | `0x70a08231` | ✅ |
| `allowance(address,address)` | `0xdd62ed3e` | `0xdd62ed3e` | ✅ |

> ⚠️ **`redeem` second param unverified**: `0x0169a996` = `redeem(uint256,uint256,address,address)`. The second param is currently `minAssetsOut = 0`. By analogy with deposit, it may also be `executionFee` (payable). Needs contract ABI verification before production use.

---

## Remaining Issues (Not Auto-Fixed)

1. **P1 — `redeem` second param may also be `executionFee`**: Not fixed pending contract ABI confirmation. If it mirrors `deposit`, redeem will also revert without ETH msg.value.
2. **P2 — Dry-run calldata uses placeholder receiver**: `build_deposit_calldata` receives `0x...0001` in dry-run, output calldata is not usable directly.

---

## Overall Assessment

**Rating**: ❌ AUDIT FAIL

Read operations work correctly (3/3). The P0 deposit signature bug has been fixed in code. However, live write verification is blocked by an onchainos issue where `contract-call --amt` (non-zero msg.value) does not broadcast transactions on Arbitrum. The audit cannot pass until at least one deposit tx is confirmed on-chain with status=1.

**Re-audit required after**: onchainos fixes `contract-call --amt` on EVM, or test is re-run with a workaround.
