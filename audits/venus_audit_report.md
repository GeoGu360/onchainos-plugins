# Skill Audit Report — Venus Core Pool

**Repo**: https://github.com/GeoGu360/onchainos-plugins (path: `venus/`)
**Audit date**: 2026-04-06
**Test wallet (EVM)**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
**Test wallet (Solana)**: `DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE`
**Test chain**: BSC (chain 56) — read-only live tests; write ops dry-run only (test wallet has no BNB balance)
**Auditor**: skill-auditor agent

---

## Summary

| Item | Result |
|------|--------|
| Compilation | ✅ (after fixes: 0 warnings) |
| Commands tested | 8 / 8 |
| Live read-only tests passed | 2 / 2 |
| Dry-run write tests passed | 6 / 6 |
| Live write tests | 0 (BSC-only; no BNB in test wallet) |
| Bugs fixed | 5 |
| Remaining issues | 0 P0, 0 P1 (all fixed), 1 P2 |

---

## Command Test Results

| # | Command | Type | Status | Tx Hash | Notes |
|---|---------|------|--------|---------|-------|
| 1 | `venus get-markets --chain 56` | Read | ✅ | — | 48 markets; vUSDC supply APY 0.49%, borrow APY 0.73% |
| 2 | `venus get-positions --chain 56 --wallet 0x87fb...` | Read | ✅ | — | Returns `positions: []` (no BSC position, as expected) |
| 3 | `venus --chain 56 --dry-run supply --asset USDT --amount 1.0` | Write (dry-run) | ✅ | dry_run | Calldata correct; amount_raw=1e18 |
| 4 | `venus --chain 56 --dry-run supply --asset BNB --amount 0.01` | Write (dry-run) | ✅ | dry_run | amount_raw=10000000000000000; selector 0x1249c58b ✓ |
| 5 | `venus --chain 56 --dry-run withdraw --asset USDT --amount 1.0` | Write (dry-run) | ✅ | dry_run | redeemUnderlying selector 0x852a12e3 ✓ |
| 6 | `venus --chain 56 --dry-run borrow --asset USDT --amount 1.0` | Write (dry-run) | ✅ | dry_run | borrow selector 0xc5ebeaec ✓ |
| 7 | `venus --chain 56 --dry-run repay --asset USDT --amount 1.0` | Write (dry-run) | ✅ | dry_run | repayBorrow selector 0x0e752702 ✓ |
| 8 | `venus --chain 56 --dry-run enter-market --asset USDT` | Write (dry-run) | ✅ | dry_run | enterMarkets selector 0xc2998238; ABI array encoding correct |
| 9 | `venus --chain 56 --dry-run claim-rewards` | Write (dry-run) | ✅ | dry_run | claimVenus selector 0xadcd5fb9 ✓ |
| 10 | Error: invalid asset | Error test | ✅ | — | `{"ok":false,"error":"Unsupported asset: INVALID..."}` — clean |
| 11 | Error: wrong chain | Error test | ✅ | — | `{"ok":false,"error":"Unsupported chain ID: 1..."}` — clean |

---

## ABI / Selector Verification

All selectors verified with `cast sig`:

| Function | Expected | Source | Match |
|----------|----------|--------|-------|
| `mint(uint256)` | `0xa0712d68` | supply.rs | ✅ |
| `mint()` (vBNB) | `0x1249c58b` | supply.rs | ✅ |
| `redeemUnderlying(uint256)` | `0x852a12e3` | withdraw.rs | ✅ |
| `borrow(uint256)` | `0xc5ebeaec` | borrow.rs | ✅ |
| `repayBorrow(uint256)` | `0x0e752702` | repay.rs | ✅ |
| `enterMarkets(address[])` | `0xc2998238` | enter_market.rs | ✅ |
| `claimVenus(address)` | `0xadcd5fb9` | claim_rewards.rs | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | onchainos.rs | ✅ |

---

## Bugs Found and Fixed

All fixes committed to `main` in commit `740b285`.

### P1 — `extract_tx_hash` returned `"pending"` silently

**File**: `src/onchainos.rs`  
**Problem**: `extract_tx_hash` returned the string `"pending"` when no txHash was present in the onchainos response. Callers output `"tx_hash": "pending"` without any error, giving the user a false success signal.  
**Fix**: Changed return type from `String` to `anyhow::Result<String>`. Now bails with an error if the hash is empty or equal to `"pending"`. All 6 write command callers updated to use `?` propagation.  
**Status**: ✅ Fixed

### P1 — `wallet_contract_call` did not check `ok` field or exit code

**File**: `src/onchainos.rs`  
**Problem**: The function parsed the stdout JSON but never checked `output.status.success()` or `json["ok"]`. A failed onchainos invocation (non-zero exit, or `"ok": false`) would be silently treated as success.  
**Fix**: Added exit-code check (`bail!` on non-zero status) and `json["ok"]` check (`bail!` if false).  
**Status**: ✅ Fixed

### P1 — BNB `--amt` parameter silently truncated for amounts > 18.44 BNB

**File**: `src/onchainos.rs` and `src/commands/supply.rs`  
**Problem**: `wallet_contract_call`'s `amt` parameter was `Option<u64>`. The BNB supply path passed `Some(amount_raw as u64)` where `amount_raw: u128`. `u64::MAX` ≈ 18.44 BNB in wei. Supplying > 18.44 BNB would silently truncate the native value sent with the transaction.  
**Fix**: Changed `amt: Option<u64>` → `amt: Option<u128>`. BNB supply call updated to pass `Some(amount_raw)` (no cast).  
**Status**: ✅ Fixed

### P2 — SKILL.md missing "Do NOT use for" rules

**File**: `skills/venus/SKILL.md`  
**Problem**: The `description` field was a long single-line string with no disambiguation rules. Claude could potentially trigger this skill for Aave, Compound, or other lending protocol requests.  
**Fix**: Rewrote description as a YAML block scalar (`>`), added three "Do NOT use for" clauses: other lending protocols, DEX swaps/bridging, and non-BSC chains.  
**Status**: ✅ Fixed

### P2 — Compiler warnings (unused constants, unused mut, dead code)

**Files**: `src/config.rs`, `src/rpc.rs`, `src/commands/get_positions.rs`  
**Problem**: `BSC_CHAIN_ID` constant was never used; `VXVS`, `pad_uint256`, `erc20_decimals`, `erc20_balance` generated dead_code warnings; `total_supply_usd` and `total_borrow_usd` in get_positions were declared `mut` but never mutated.  
**Fix**: Removed `BSC_CHAIN_ID`; added `#[allow(dead_code)]` to utility items reserved for future use; changed `mut` bindings to immutable.  
**Status**: ✅ Fixed (0 warnings on clean rebuild)

---

## Outstanding Issues

### P2 — `get-markets` and `get-positions` are slow (serial RPC calls)

**Problem**: Both commands iterate over 48 markets sequentially, making individual `eth_call`s for each. `get-markets` takes ~60–120 seconds and `get-positions` takes ~30–60 seconds due to sequential I/O.  
**Recommendation**: Batch or parallelize the per-market RPC calls using `tokio::join_all` or an `eth_call` multicall helper. This is an architecture change and was not auto-fixed.

---

## Static Code Review

### SKILL.md Quality (post-fix)

- [x] description field is ASCII-only (no CJK)
- [x] Trigger phrases cover common English variants
- [x] "Do NOT use for" rules present (added in fix)
- [x] Each command has usage examples with parameters

### Code Quality

- [x] Contract addresses are correct BSC mainnet addresses (verified via publicnode RPC)
- [x] Amount precision: USDT/USDC on BSC both have 18 decimals — config is correct
- [x] ERC-20 approve uses `contract-call`, not `dex approve`
- [x] No Solana transactions (BSC only)
- [x] Error handling: clean `{"ok":false,"error":"..."}` JSON output, no panics
- [x] `extract_tx_hash` now returns `Result` (fixed)
- [x] `wallet_contract_call` now checks exit code and `ok` field (fixed)
- [x] `source_repo` in plugin.yaml: `GeoGu360/onchainos-plugins` — correct

---

## Decimal Precision Verification

BSC on-chain check confirmed:
- USDT (BSC): 18 decimals ✅
- USDC (BSC): 18 decimals ✅

Config hardcodes `decimals: 18` for all assets — correct.

---

## Overall Assessment

Venus Core Pool plugin is functionally well-structured. The BSC-only scope is appropriate and clearly documented. All five bugs found were directly fixable and have been committed. The main remaining concern is performance of read commands due to serial RPC calls — acceptable for a v0.1.0 but should be addressed before heavy usage.

**Rating**: ⭐⭐⭐⭐ (4/5) — solid implementation with minor robustness fixes applied
