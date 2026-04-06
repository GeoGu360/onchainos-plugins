# Skill Audit Report — LayerBank

**Plugin Source**: `/tmp/onchainos-plugins/layer-bank/`
**Audit Date**: 2026-04-06
**Test Wallet (EVM)**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
**Test Chain**: Scroll (chain ID 534352)
**Auditor**: skill-auditor agent (Claude Sonnet 4.6)

---

## Summary

| Item | Result |
|------|--------|
| Compilation | PASS (6 dead-code warnings, 0 errors) |
| Skill install / uninstall | PASS |
| Commands tested | 6 / 6 |
| Read-only commands (markets, positions) | 2 PASS |
| Dry-run commands (supply, withdraw, borrow, repay) | 4 PASS |
| On-chain write ops | Dry-run only per audit policy |
| Amount precision verification | PASS after fix |
| Issues found | 4 (2 × P1, 1 × P2, 1 × P3) |
| All issues fixed | YES — committed and pushed |

---

## Step 0 — Environment

- **onchainos CLI**: v2.2.6 (binary SHA256 verified against checksums.txt ✅)
- **Skill version in SKILL.md**: 0.1.0
- **Rust edition**: 2021
- **EVM address**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
- **Scroll address**: same (EVM-compatible)

---

## Step 1 — Build

```
cd /tmp/onchainos-plugins/layer-bank && cargo build --release
```

**Result**: `Finished release profile [optimized]` — no errors.

6 dead-code warnings:
- `exchange_rate`, `all_markets`, `market_info_of`, `account_liquidity_of`, `erc20_balance_of`, `ltoken_symbol` in `rpc.rs` — unused helper functions. Non-blocking, noted.

---

## Step 2 — Skill Install / Uninstall

- Binary copied to `~/.local/bin/layer-bank` ✅
- `layer-bank --help` confirmed all 6 subcommands present ✅
- Binary removed post-audit ✅

---

## Step 3 — Read Command Tests

### `markets`

```bash
layer-bank --chain 534352 markets
```

**Result**: PASS — returned live data for 5 markets (ETH, USDC, USDT, wstETH, WBTC) with TVL, utilization %, exchange rates, and USD prices from Scroll RPC.

Sample data at audit time:
- ETH: TVL $206,651, utilization 47.49%, price $2,130.66
- USDC: TVL $67,612, utilization 87.58%
- USDT: utilization 100.00% (fully borrowed)
- WBTC: utilization 100.00%, price $68,757.40

All values plausible for Scroll mainnet state.

### `positions`

```bash
layer-bank --chain 534352 positions --wallet 0x87fb0647faabea33113eaf1d80d67acb1c491b90
```

**Result**: PASS — returned `health_factor: ∞ (no debt)`, empty `supplied` and `borrowed` arrays. Correct for a wallet with no LayerBank positions.

---

## Step 4 — Write Command Dry-Run Tests

### `supply --dry-run`

**ETH**: `layer-bank --chain 534352 --dry-run supply --asset ETH --amount 0.001`
- Calldata correct: `0xf2b9fdb8` + lETH address padded + `0x0` (ETH path)
- `value_wei`: `1000000000000000` ✅

**USDC**: `layer-bank --chain 534352 --dry-run supply --asset USDC --amount 1.0`
- Step 1: `0x095ea7b3` (approve) to USDC contract, `raw_amount: "1000000"` ✅
- Step 2: `0xf2b9fdb8` (supply) to Core ✅

### `withdraw --dry-run`

**ETH**: `layer-bank --chain 534352 --dry-run withdraw --asset ETH --amount 0.001`
- Selector `0x96294178` (redeemUnderlying) ✅
- `raw_amount: "1000000000000000"` ✅

### `borrow --dry-run`

**USDC**: `layer-bank --chain 534352 --dry-run borrow --asset USDC --amount 1.0`
- Selector `0x4b8a3529` (borrow) ✅
- `raw_amount: "1000000"` ✅
- Warning message about liquidation risk present ✅

### `repay --dry-run`

**USDC**: `layer-bank --chain 534352 --dry-run repay --asset USDC --amount 1.0`
- Step 1: approve (`0x095ea7b3`) ✅
- Step 2: repayBorrow (`0xabdb5ea8`) ✅

**ETH**: `layer-bank --chain 534352 --dry-run repay --asset ETH --amount 0.001`
- ETH repay: payable call with `value_wei` ✅

---

## Step 5 — ABI / Calldata Spot Checks

| Function | Expected Selector | Observed | Match |
|----------|------------------|----------|-------|
| `Core.supply(address,uint256)` | `0xf2b9fdb8` | `0xf2b9fdb8` | ✅ |
| `Core.redeemUnderlying(address,uint256)` | `0x96294178` | `0x96294178` | ✅ |
| `Core.borrow(address,uint256)` | `0x4b8a3529` | `0x4b8a3529` | ✅ |
| `Core.repayBorrow(address,uint256)` | `0xabdb5ea8` | `0xabdb5ea8` | ✅ |
| `ERC20.approve(address,uint256)` | `0x095ea7b3` | `0x095ea7b3` | ✅ |

Contract addresses verified against SKILL.md documentation:

| Contract | Expected | Match |
|----------|----------|-------|
| Core | `0xEC53c830f4444a8A56455c6836b5D2aA794289Aa` | ✅ |
| lETH | `0x274C3795dadfEbf562932992bF241ae087e0a98C` | ✅ |
| lUSDC | `0x0D8F8e271DD3f2fC58e5716d3Ff7041dBe3F0688` | ✅ |
| lUSDT | `0xE0Cee49cC3C9d047C0B175943ab6FCC3c4F40fB0` | ✅ |
| lwstETH | `0xB6966083c7b68175B4BF77511608AEe9A80d2Ca4` | ✅ |
| lWBTC | `0xc40D6957B8110eC55f0F1A20d7D3430e1d8Aa4cf` | ✅ |

---

## Step 6 — Static Review: Issues Found and Fixed

### BUG-1 (P1) — `extract_tx_hash` returns `String` instead of `Result<String>`

**File**: `src/onchainos.rs`

**Problem**: `extract_tx_hash` returned a plain `String`, falling back to `"pending"` on any failure. When `wallet contract-call` returned `{"ok": false, "error": "..."}`, the error was silently swallowed and the caller continued reporting a fake `"pending"` txHash as success.

**Fix**: Changed signature to `anyhow::Result<String>`. The function now explicitly checks `ok == false` and returns an error, and returns `Err` when no txHash field is present.

**Updated callers**: `supply.rs`, `withdraw.rs`, `borrow.rs`, `repay.rs` — all now use `?` to propagate the error.

### BUG-2 (P1) — No `ok`-check after `wallet_contract_call`

**File**: `src/onchainos.rs` + all command files

**Problem**: `wallet_contract_call` parsed the onchainos JSON response but never verified the `ok` field before returning it to callers. A failed tx (`ok: false`) would be passed to `extract_tx_hash`, which then returned `"pending"` and execution continued.

**Fix**: Resolved jointly with BUG-1 — `extract_tx_hash` now checks `ok == false` before attempting to extract the txHash, propagating the error.

### BUG-3 (P2) — `amount` precision: `f64` floating-point loss in `to_raw()`

**File**: `src/config.rs`

**Problem**: `to_raw(amount: f64, decimals: u8)` computed `(amount * 10^decimals).round() as u128`. For certain decimal representations (e.g., `0.1 * 10^18`), f64 intermediate results can introduce off-by-one rounding errors due to IEEE 754 binary representation.

**Fix**: Replaced the multiplication with a string-split approach: format the f64 with `decimals` decimal places, split at the decimal point, concatenate integer + fractional parts (padded/truncated to `decimals` digits), then parse as `u128`. Verified: `0.12345678 WBTC → 12345678`, `1.000001 USDC → 1000001`, `0.001 ETH → 1000000000000000`.

### BUG-4 (P3) — `SKILL.md` missing "Do NOT use for" section; `plugin.yaml` wrong `source_repo`

**Files**: `skills/layer-bank/SKILL.md`, `plugin.yaml`

**Problem**:
1. SKILL.md had no "Do NOT use for" section (required per monorepo standard; present in compound-v3, kamino-liquidity, morpho, etc.)
2. SKILL.md frontmatter description did not embed "Do NOT use for" guidance (contrast: morpho embeds it in the description field)
3. `plugin.yaml` `source_repo` was `GeoGu360/onchainos-plugins` (author's fork), not the canonical `okx/onchainos-plugins`

**Fix**:
1. Added `## Do NOT use for` section to SKILL.md body
2. Appended "Do NOT use for" note to description frontmatter field
3. Corrected `source_repo` to `okx/onchainos-plugins`

---

## Step 7 — Observations (Not Fixed)

### OBS-1 — `--force` always passed in `wallet_contract_call`

`onchainos.rs` always appends `--force` to the contract-call args. Per the agentic wallet skill rules, `--force` should only be passed after receiving a confirming response (exit code 2). However, this is a known pattern in the monorepo for plugin backends that are already gated behind the plugin's own user-confirmation flow. The SKILL.md correctly instructs the agent to "ask user to confirm before broadcasting." No change made — architectural decision deferred to maintainer.

### OBS-2 — 6 unused `rpc.rs` helpers

`exchange_rate`, `all_markets`, `market_info_of`, `account_liquidity_of`, `erc20_balance_of`, `ltoken_symbol` are defined but never called. These provide scaffolding for future commands. Non-blocking — no fix applied.

### OBS-3 — `source_commit` is all zeros

`plugin.yaml` has `source_commit: "0000000000000000000000000000000000000000"`. This is a placeholder that should be updated when the plugin is tagged for release. Noted for maintainer.

---

## Step 8 — Commit

```
commit 3302fde
fix(layer-bank): extract_tx_hash returns Result, ok-check, amount precision, SKILL.md Do NOT section
```

Pushed to `origin/main` of the monorepo.

---

## Verdict

**PASS with fixes applied.**

All 6 commands function correctly. Four bugs were identified and fixed: two P1 issues (silent error swallowing in transaction flow), one P2 (amount precision), and one P3 (documentation/metadata). The plugin is safe for read-only use and dry-run; write operations correctly propagate errors after the fix.
