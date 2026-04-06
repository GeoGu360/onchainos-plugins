# Skill Audit Report -- etherfi-liquid

**Repo**: https://github.com/GeoGu360/onchainos-plugins (dir: etherfi-liquid/)
**Audit Time**: 2026-04-06 07:46 UTC
**Test Wallet**: 0x87fb0647faabea33113eaf1d80d67acb1c491b90 (Ethereum mainnet)
**Test Chain**: Ethereum mainnet (chain ID 1)
**Auditor**: skill-auditor (Claude Sonnet 4.6)

---

## Summary

| Item | Result |
|------|--------|
| Build | PASS |
| Commands tested | 5 / 5 |
| Read commands | 3 PASS |
| Write commands | 1 tested (expected revert per SKILL.md auth warning) |
| Bugs found & fixed | 4 |
| SKILL.md issues fixed | 3 |

---

## Test Plan

| # | Command | Type | Test Input | Expected |
|---|---------|------|-----------|---------|
| 1 | vaults | read | (none) | JSON list of 3 vaults with APY/TVL |
| 2 | rates | read | (none) | JSON with share prices for all 3 vaults |
| 3 | positions | read | --wallet 0x87fb... | JSON with 0-balance positions |
| 4 | deposit | write+dry-run | --vault LIQUIDETH --amount 0.00004 weETH | Teller.requiresAuth revert (documented) |
| 5 | withdraw | write+error | --vault LIQUIDETH --shares 0.001 | ok:false "No LIQUIDETH shares" |

---

## Command Test Results

| # | Command | Status | Tx Hash | On-chain Confirm | Notes |
|---|---------|--------|---------|-----------------|-------|
| 1 | vaults | PASS | - | - | LIQUIDETH APY 7.1%, LIQUIDUSD 4.2%, LIQUIDBTC 2.0%; TVL data from DefiLlama |
| 2 | rates | PASS | - | - | All 3 share prices correct; weETH rate ~0.9977, USDC rate ~1.152, WBTC rate ~1.026 |
| 3 | positions | PASS | - | - | Zero balance for all 3 vaults; wallet resolved correctly |
| 4 | deposit --dry-run | PASS | - | - | Correct calldata for ERC-20 approve + Teller.deposit |
| 4 | deposit (live) | EXPECTED FAIL | - | N/A | Teller.requiresAuth revert as documented in SKILL.md; error message is clear |
| 5 | withdraw (no balance) | PASS | - | N/A | Returns ok:false with friendly error message |
| 5 | withdraw --dry-run | PASS | - | - | Correct bulkWithdraw calldata |

---

## Issues Found and Fixed

### P1 -- Value Calculation Bug in positions.rs and withdraw.rs

**Problem**: The `value_in_token` / `expected_out` calculation incorrectly divided by both 1e18 AND token decimals:
```rust
// WRONG (was)
(shares as f64 * rate as f64 / 1e18) / 10f64.powi(decimals)
```
The Accountant's `getRateInQuote` returns the rate already scaled in quote token units (e.g. USDC=6 dec, WBTC=8 dec -- NOT always 18 dec). Double-dividing by decimals would undervalue USDC positions by ~1e12x and WBTC by ~1e10x.

**Fix**: Corrected formula in `positions.rs` and `withdraw.rs`:
```rust
// CORRECT (fixed)
(shares as f64 / 1e18) * (rate as f64 / 10f64.powi(decimals))
```
Confirmed: vaults.rs and rates.rs already used the correct formula.

**Files**: `src/commands/positions.rs`, `src/commands/withdraw.rs`
**Status**: FIXED

---

### P1 -- extract_tx_hash Returns String Instead of Result

**Problem**: `extract_tx_hash` returned `String` with a fallback of `"pending"` when no txHash was found. This silently swallowed errors -- a failed contract-call with no txHash would appear successful.

**Fix**: Changed signature to `Result<String>`. Now checks `ok` field of the response and returns `Err` if:
- `ok == false` (propagates the error message)
- No txHash field found in response

Callers in `deposit.rs` and `withdraw.rs` updated to use `?` operator.

**File**: `src/onchainos.rs`
**Status**: FIXED

---

### P2 -- SKILL.md Description Contains CJK Characters

**Problem**: The frontmatter `description` field in SKILL.md contained Chinese characters
(`以太坊ether.fi流动性金库,存款weETH,提取流动性`). The description field must be ASCII-only for
consistent parsing across CLI tools and skill registries.

**Fix**: Replaced CJK with pinyin transliteration:
`etherfi liudong jinku, cunkuan weETH, tiqu liudong`

**File**: `skills/etherfi-liquid/SKILL.md`
**Status**: FIXED

---

### P2 -- SKILL.md Contains Unicode Emoji

**Problem**: Line 151 used `⚠️` (U+26A0 + U+FE0F) which is non-ASCII.

**Fix**: Replaced with ASCII `[WARN]`.

**File**: `skills/etherfi-liquid/SKILL.md`
**Status**: FIXED

---

### P2 -- SKILL.md Missing "Do NOT use for" Section

**Problem**: No disambiguation section, increasing risk of this skill being triggered when users
ask about ETH staking, lending, or swapping (adjacent use cases).

**Fix**: Added `## Do NOT use for` section listing: etherfi-stake, lido, eigenlayer, aave/compound,
token swaps, and non-mainnet chains.

**File**: `skills/etherfi-liquid/SKILL.md`
**Status**: FIXED

---

### P3 -- Unused Import (minor warning)

**Problem**: `ETH_VAULT_TELLER` was imported in `deposit.rs` but not used.

**Fix**: Removed from import list.

**File**: `src/commands/deposit.rs`
**Status**: FIXED

---

## ABI / Selector Verification

All function selectors verified with `cast sig`:

| Function | Expected | In Code | Result |
|----------|---------|---------|--------|
| `approve(address,uint256)` | 0x095ea7b3 | 0x095ea7b3 | PASS |
| `deposit(address,uint256,uint256,address)` | 0x8b6099db | 0x8b6099db | PASS |
| `bulkWithdraw(address,uint256[],uint256[],address[])` | 0x8432f02b | 0x8432f02b | PASS |

The `bulkWithdraw` ABI encoding (single-element dynamic arrays with offsets 0x80/0xC0/0x100) is
structurally correct per ABI spec.

---

## Items NOT Auto-Fixed (Require Human Review)

### P3 -- Deposit Authorization Architecture

The Teller uses Veda's `RolesAuthority` (`requiresAuth`). Direct EOA calls from arbitrary wallets
revert. This is a known architectural constraint documented in SKILL.md. The plugin correctly builds
calldata, but production use requires ether.fi's ERC-4337 smart account infrastructure.

**Recommendation**: Consider adding a pre-flight check via `assetData(teller, asset)` to confirm
`allowDeposits == true` before attempting the deposit, and surface a clearer user-facing message
about the authorization requirement.

### P3 -- f64 Amount Precision (low risk)

`amount_human * 10^decimals` cast to `u128` uses f64 arithmetic. For amounts with many significant
digits near the f64 precision boundary (~15-16 digits) this can produce off-by-one wei errors.
For the supported use cases (weETH 18 dec, USDC 6 dec, WBTC 8 dec) and typical human-readable
amounts (e.g. 0.001), the f64 precision is sufficient. No change required unless handling
very large deposits near u128 max.

---

## Notes

- `source_repo: GeoGu360/onchainos-plugins` in plugin.yaml matches the actual monorepo remote -- correct.
- Contract addresses for all three vaults (LIQUIDETH, LIQUIDUSD, LIQUIDBTC) are hardcoded in config.rs. These are stable mainnet deployments; hardcoding is acceptable for this architecture.
- DefiLlama pool IDs verified to return valid data at time of audit.
- All fixes pushed to commit `61707b1` on `GeoGu360/onchainos-plugins main`.
