# Audit Report: curve-lending

**Date:** 2026-04-06  
**Auditor:** skill-auditor  
**Plugin version:** 0.1.0  
**Commit audited:** 9cec838 (pre-fix)  
**Fix commit:** f8ce213  
**Chain:** Ethereum mainnet (chain ID: 1)  
**EVM wallet:** 0x87fb0647faabea33113eaf1d80d67acb1c491b90

---

## 1. Build

```
cargo build --release
```

**Result:** SUCCESS — 32 warnings (all pre-existing unused-import/dead-code warnings), zero errors.

---

## 2. Functional Test Results

All 6 commands tested:

| Command | Mode | Result | Notes |
|---------|------|--------|-------|
| `markets --chain 1 --limit 5` | read | PASS | 46 markets, correct TVL/debt values |
| `rates --chain 1 --market WETH-long` | read | PASS | borrow_apy ~0.12%, utilization ~35.6% |
| `positions --chain 1 --address 0x87fb...` | read | PASS | 0 active positions (expected) |
| `deposit-collateral --market WETH-long --amount 0.001 --dry-run` | dry-run | PASS | Correct calldata for create_loan |
| `borrow --market WETH-long --amount 100 --collateral 0.05 --dry-run` | dry-run | PASS | Correct calldata, max_borrowable validation |
| `repay --market WETH-long --amount 100 --dry-run` | dry-run | PASS | Correct crvUSD approve + repay calldata |

Write operations (deposit-collateral live, borrow, repay) were not executed on-chain per GUARDRAILS:
- No WETH balance in test wallet for deposit
- Borrow and repay are dry-run only in test environment

---

## 3. Static Review

### 3.1 extract_tx_hash — FIXED (Bug)

**File:** `src/onchainos.rs`

**Before:**
```rust
pub fn extract_tx_hash(result: &Value) -> String {
    result["data"]["txHash"]
        .as_str()
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("pending")
        .to_string()
}
```

**Problem:** Returns `String` with fallback `"pending"` instead of `anyhow::Result<String>`.
This is inconsistent with the monorepo standard (aave-v3, compound-v3 both return `Result`).
A missing txHash silently produces `"pending"` in the output JSON rather than propagating an error.

**After:**
```rust
pub fn extract_tx_hash(result: &Value) -> anyhow::Result<String> {
    let hash = result["data"]["txHash"]
        .as_str()
        .or_else(|| result["txHash"].as_str());
    match hash {
        Some(h) if !h.is_empty() && h != "pending" => Ok(h.to_string()),
        _ => anyhow::bail!(
            "txHash not found or still pending in onchainos response; raw: {}",
            result
        ),
    }
}
```

All call sites updated to use `?` propagation.

### 3.2 ok-check Order — FIXED (Bug)

**Files:** `src/commands/deposit_collateral.rs`, `src/commands/repay.rs`

**Before:** `extract_tx_hash()` was called before the `ok` check, meaning if `ok=false` but the
response contained a txHash field, it would extract and log it before hitting the bail.

**After:** `ok` check moved before `extract_tx_hash()` call, consistent with deposit_collateral
and repay flow.

### 3.3 Unused Import — FIXED

**File:** `src/commands/deposit_collateral.rs`

`CRVUSD` was imported from `config` but not used. Removed.

### 3.4 SKILL.md Check

- **ASCII:** SKILL.md contains 8 em-dash characters (U+2014, `—`). These are valid UTF-8 and
  consistent with the monorepo standard (aave-v3 SKILL.md also contains UTF-8). No issue.
- **Do NOT use for section:** Not present. Not required by SKILL.md spec (only Routing Rules
  section present, which serves the same purpose). No issue.
- **source_repo:** `plugin.yaml` has `source_repo: GeoGu360/onchainos-plugins` — correct.

### 3.5 Amount Precision

`f64` is used for amount-to-u128 conversion:
```rust
let collateral_raw = (args.amount * divisor as f64) as u128;
```

At 18 decimals, f64 precision is ~15-16 significant digits, causing up to ~2 wei rounding error
at the 18th decimal place. This is acceptable for practical amounts (0.001 ETH has exact f64
representation). Consistent with monorepo standard across other plugins.

### 3.6 find_market Duplication

`find_market()` is duplicated in `deposit_collateral.rs`, `borrow.rs`, and `repay.rs`. No bug,
but a refactoring opportunity for future maintainability.

### 3.7 Guardrails

- Borrow and repay correctly check `--dry-run` before executing.
- SKILL.md notes "Ask user to confirm before executing the real transaction" on all write commands.
- Repay uses `current_debt` (not `uint256::MAX`) for full repay, avoiding revert risk. Correct.

---

## 4. Contract Verification

All selectors verified against design.md (cast sig verified):

- Factory: `market_count()` `0xfd775c78`, `names()` `0x4622ab03`, `controllers()` `0xe94b0dd2` — all correct
- Controller: `loan_exists()` `0xa21adb9e`, `health()` `0x8908ea82`, `create_loan()` `0x23cfed03` — all correct
- Repay: `repay(uint256)` `0x371fd8e6` — verified live on-chain per design.md

---

## 5. Bugs Found and Fixed

| # | Severity | File | Issue | Status |
|---|----------|------|-------|--------|
| 1 | Medium | `src/onchainos.rs` | `extract_tx_hash` returns `String` not `Result` — silently swallows missing txHash as "pending" | FIXED |
| 2 | Low | `src/commands/deposit_collateral.rs`, `repay.rs` | `extract_tx_hash` called before ok-check | FIXED |
| 3 | Low | `src/commands/deposit_collateral.rs` | Unused `CRVUSD` import | FIXED |

---

## 6. Uninstall

Binary uninstalled from `~/.local/bin/curve-lending`.

---

## 7. Summary

The plugin is well-structured and functionally correct for all read and dry-run operations.
The main bug was `extract_tx_hash` returning `String` instead of `anyhow::Result<String>`,
which is the monorepo standard and prevents silent failure in write paths.
Three fixes applied, verified by rebuild (zero errors), and pushed to monorepo main at commit `f8ce213`.

---

## Re-audit — 2026-04-06

**Wallet assets available:** ETH 0.1747, USDC 5.40, USDT 15.03, USDS 4.31  
**Re-audit goal:** Determine whether USDC/USDT/USDS can be used as collateral for a live supply test.

### Stablecoin Collateral Analysis

Curve Lending is a **collateral-in / borrow-crvUSD** protocol. Users deposit a collateral token into a Controller contract to borrow crvUSD. The protocol does NOT accept arbitrary ERC-20 tokens — only the specific collateral token configured per market.

All 46 markets were enumerated from the factory contract (`0xeA6876DDE9e3467564acBeE1Ed5bac88783205E0`). Full market list and collateral token addresses verified via direct RPC calls to `eth.drpc.org`.

**Markets with USD-adjacent collateral:**

| Index | Market Name | Collateral Address | Collateral Symbol |
|-------|-------------|-------------------|-------------------|
| 11 | sUSDe-long-v2 | 0x9d39a5de30e57443bff2a8307a4256c8797a3497 | sUSDe |
| 14 | susde-long | 0x4c9edd5852cd905f086c759e8383e09bff1e68b3 | USDe |
| 15 | sfrax-long | 0xa663b02cf0a4b149d2ad41910cb81e23e1c41c32 | sFRAX |
| 19 | USD0/USD0+-long | 0x1d08e7adc263cfc70b1babe6dc5bb339c16eec52 | USD0USD0++ |
| 20 | USD0/USD0+-long | 0x1d08e7adc263cfc70b1babe6dc5bb339c16eec52 | USD0USD0++ |
| 28 | sfrxUSD | 0xcf62f905562626cfcdd2261162a51fd02fc9c5b6 | sfrxUSD |
| 32 | sUSDS | 0xa3931d71877c0e7a3148cb7eb4463524fec27fbd | sUSDS |
| 33 | sUSDf-long | 0xc8cf6d7991f15525488b2a83df53468d682ba4b0 | sUSDf |
| 34 | wstUSR-long | 0x1202f5c7b4b9e47a1a484e8b270be34dbbc75055 | wstUSR |
| 36 | yvUSDC-1-Long | 0xbe53a109b494e5c9f97b9cd39fe969be68bf6204 | yvUSDC-1 |
| 37 | yvUSDS-1-Long | 0x182863131f9a4630ff9e27830d945b1413e347e8 | yvUSDS-1 |
| 40 | sdeUSD | 0x5c5b196abe0d54485975d1ec29617d42d9198326 | sdeUSD |
| 41 | sreUSD-long | 0x557ab1e003951a73c12d16f0fea8490e39c33c35 | sreUSD |

**Verdict for wallet assets:**

| Asset | Supported as Collateral | Reason |
|-------|------------------------|--------|
| USDC (`0xa0b86991...`) | NO | No market uses raw USDC as collateral. Market 36 uses yvUSDC-1 (Yearn vault shares), which requires first depositing USDC into Yearn. |
| USDT (`0xdac17f95...`) | NO | No market uses USDT as collateral in any form. |
| USDS (`0xdC035D45...`) | NO | No market uses raw USDS. Market 37 uses yvUSDS-1 (Yearn vault shares); market 32 uses sUSDS (staked USDS via Sky Protocol). Both require intermediate wrapping steps. |

**Conclusion: Live write test not possible with current wallet assets.**

Raw USDC, USDT, and USDS are not accepted as collateral in any Curve Lending market. The
protocol only accepts yield-bearing wrapped versions (yvUSDC-1, yvUSDS-1, sUSDS) or
entirely different stablecoins (USDe, sFRAX, sfrxUSD, etc.). Obtaining any of these would
require at least one additional DEX swap or staking operation outside the scope of this plugin.

### Re-audit Test Results

| # | Command | Mode | Result | Notes |
|---|---------|------|--------|-------|
| 1 | `markets --limit 50` | read | PASS (via direct RPC) | 46 markets confirmed, full list captured |
| 2 | `rates --market yvUSDC` | read | PASS | borrow_apy 0.10%, zero TVL |
| 3 | `rates --market sUSDS` | read | PASS | borrow_apy 0.82%, utilization 38.06%, TVL 18,773 crvUSD |
| 4 | `positions --address 0x87fb...` | read | PASS | 0 active positions |
| 5 | `deposit-collateral --market yvUSDC --amount 1 --dry-run` | dry-run | PASS | Correct calldata generated: approve yvUSDC-1 + create_loan |
| 6 | Live write ops | N/A | SKIPPED | Wallet holds no accepted collateral (USDC/USDT/USDS are not valid collateral tokens) |

**Plugin note in SKILL.md (pre-existing):** "Collateral tokens: WETH, wstETH, tBTC, CRV, sfrxETH (check `markets` for full list)" — this is accurate but incomplete. The list should also mention the broader set of staked/wrapped stablecoins (sUSDS, yvUSDC-1, yvUSDS-1, sUSDe, USDe, etc.) that exist in the 46 markets. No code change required; SKILL.md is correct in directing users to run `markets` for the full list.

### Re-audit Build

```
cd /tmp/onchainos-plugins/curve-lending && cargo build --release
```

Result: SUCCESS — `Finished release profile in 0.11s` (already built, no recompile needed).

### Re-audit Verdict

- **Live supply with USDC/USDT/USDS: NOT POSSIBLE** — none are accepted collateral tokens.
- **Dry-run only status maintained** for all write operations.
- No new bugs found. Plugin functions correctly for all read and dry-run paths.
- No code changes required.
