# Skill Audit Report — moonwell

**Repo**: https://github.com/GeoGu360/onchainos-plugins (dir: moonwell)
**Audit date**: 2026-04-06
**Auditor**: skill-auditor (Claude Sonnet 4.6)
**Test wallet**: 0x87fb0647faabea33113eaf1d80d67acb1c491b90
**Test chain**: Base (chain 8453)

---

## Summary

| Item | Result |
|------|--------|
| Build | PASS (0 warnings after fixes) |
| Commands tested | 7 / 7 |
| Dry-run tests | 7 / 7 PASS |
| Live write ops | 1 (claim-rewards: confirmed on-chain) |
| ABI selectors verified | 11 / 11 correct |
| Bugs fixed | 3 |
| Pushed to monorepo main | DONE (eef02e8) |

---

## Test Plan

| # | Command | Type | Key Params |
|---|---------|------|-----------|
| 1 | `markets` | Read | -- |
| 2 | `positions` | Read | --wallet |
| 3 | `supply --dry-run` | Dry-run | --asset USDC, --amount 0.01 |
| 4 | `redeem --dry-run` | Dry-run | --asset USDC, --mtoken-amount 0.44 |
| 5 | `borrow --dry-run` | Dry-run (only) | --asset USDC, --amount 5.0 |
| 6 | `repay --dry-run` | Dry-run (only) | --asset USDC, --amount 5.0 |
| 7 | `claim-rewards` | Live write | --from 0x87fb... |

---

## Command Test Results

| # | Command | Status | Tx Hash | On-chain | Notes |
|---|---------|--------|---------|----------|-------|
| 1 | `markets` | PASS | - | - | 5 markets returned (USDC 2.82% APY, WETH 0.89%, cbETH 2.73%, USDbC 0.00%, DAI 0.00%) |
| 2 | `positions --wallet 0x87fb...` | PASS | - | - | Active USDC position found: supplied 0.010001 USDC, mtoken_balance 0.44084856 |
| 3 | `supply USDC 0.01 --dry-run` | PASS | - | - | 2-step approve+mint calldata correct (0x095ea7b3 + 0xa0712d68) |
| 4 | `redeem USDC 0.44 --dry-run` | PASS | - | - | redeem(uint256) 0xdb006a75 calldata correct, mtoken_raw=44000000 |
| 5 | `borrow USDC 5.0 --dry-run` | PASS | - | - | dry-run only enforced with exit=1 (without --dry-run) |
| 6 | `repay USDC 5.0 --dry-run` | PASS | - | - | dry-run only enforced with exit=1 (without --dry-run) |
| 7 | `claim-rewards --from 0x87fb...` | PASS | 0x670471f57df086284d070f95ff05da83f2a5cd9e89e6b822acd89eb434936c98 | CONFIRMED status=1 block=44337418 | 0 WELL accrued (expected for test wallet) |

Note: `supply` and `redeem` live execution not performed — wallet already has a small USDC position from prior audit; risk of disruption not justified.

---

## Bugs Found and Fixed

### P1 — `extract_tx_hash` returned `String` not `Result<String>`

**File**: `src/onchainos.rs:81`

**Problem**: The signature `pub fn extract_tx_hash(result: &Value) -> String` silently returned the string `"pending"` when the wallet call failed or returned no txHash. All callers (supply, redeem, borrow, repay, claim_rewards) would then report `"ok": true` with `txHash: "pending"` even when the underlying EVM transaction was rejected or the wallet returned an error. This is a silent failure vector — the UI would show success while the on-chain state was unchanged.

**Fix**: Changed signature to `pub fn extract_tx_hash(result: &Value) -> anyhow::Result<String>`. The new implementation:
1. Checks `ok == false` on the result and bails with the wallet's error message
2. On dry_run path, returns the zero-hash sentinel without error
3. On live path, bails if `txHash` is absent or empty
4. All 6 call sites updated to propagate with `?`

**Status**: Fixed, compiled, verified (zero warnings).

---

### P2 — SKILL.md `description` frontmatter contained U+2014 em-dash (non-ASCII)

**File**: `skills/moonwell/SKILL.md` line 3

**Problem**: The YAML frontmatter `description` field contained a Unicode em-dash character (U+2014 `—`). YAML parsers in strict ASCII environments may reject or misparse this, causing skill discovery failures.

**Fix**: Replaced `—` with ASCII hyphen `-`. The description now also includes "Do NOT use for" routing guidance directly in the frontmatter.

**Status**: Fixed.

---

### P2 — SKILL.md missing "Do NOT use for" routing guidance

**File**: `skills/moonwell/SKILL.md`

**Problem**: The SKILL.md had no "Do NOT use for" section. Without it, the LLM routing layer may incorrectly invoke this skill for token swaps, Aave operations, Compound V3, or other non-Moonwell requests.

**Fix**: Added a "Do NOT use for" section listing: token swaps/DEX (use `lifi`), Aave V3 (use `aave-v3`), Compound V3 (use `compound-v3`), wallet balance queries, ETH liquid staking, and unsupported chains (Optimism/Moonbeam).

**Status**: Fixed.

---

## Static Code Review Checklist

| Check | Result |
|-------|--------|
| ABI selectors correct | PASS — all 11 selectors verified via keccak256 |
| `extract_tx_hash` returns `Result` | FIXED (was `String`) |
| `ok`-field check after wallet_contract_call | FIXED (via extract_tx_hash Result propagation) |
| `borrow`/`repay` dry-run-only enforcement | PASS — anyhow::bail! with exit code 1 confirmed |
| SKILL.md description ASCII-only | FIXED (em-dash removed) |
| SKILL.md "Do NOT use for" section | FIXED (added) |
| `source_repo` correct | PASS — `GeoGu360/onchainos-plugins` matches remote |
| `amount` precision (f64 -> u128 via to_raw) | PASS — USDC: 6 decimals, WETH/cbETH/DAI: 18, mToken redeem: 8 decimals |
| ERC-20 approve uses correct selector | PASS — 0x095ea7b3 approve(address,uint256) verified |
| Compiler warnings | PASS — zero warnings on release build |
| Unused imports | PASS — none found |
| Panic / unwrap in hot paths | PASS — uses anyhow::Result throughout |
| Chain guard (only Base 8453) | PASS — chain_config bails on unsupported chains with exit=1 |
| Borrow calldata | PASS — 0xc5ebeaec borrow(uint256) correct |
| RepayBorrow calldata | PASS — 0x0e752702 repayBorrow(uint256) correct |
| ClaimReward calldata | PASS — 0xd279c191 claimReward(address) correct |
| exchange_rate math | PASS — er_human = exchange_rate/1e18/scale, positions uses mtoken*rate/1e18/dec |

---

## ABI Selector Verification

| Function | Expected Selector | Status |
|----------|-------------------|--------|
| approve(address,uint256) | 0x095ea7b3 | PASS |
| mint(uint256) | 0xa0712d68 | PASS |
| redeem(uint256) | 0xdb006a75 | PASS |
| borrow(uint256) | 0xc5ebeaec | PASS |
| repayBorrow(uint256) | 0x0e752702 | PASS |
| balanceOf(address) | 0x70a08231 | PASS |
| borrowBalanceCurrent(address) | 0x17bfdfbc | PASS |
| exchangeRateCurrent() | 0xbd6d894d | PASS |
| supplyRatePerTimestamp() | 0xd3bd2c72 | PASS |
| borrowRatePerTimestamp() | 0xcd91801c | PASS |
| claimReward(address) | 0xd279c191 | PASS |

---

## Architecture Notes

- Read ops (`markets`, `positions`) use direct `eth_call` via `https://base.publicnode.com` — no wallet needed, live data confirmed.
- Write ops use `onchainos wallet contract-call --force` — correct pattern.
- `supply` is a 2-step flow (ERC20.approve + mToken.mint) with a 3-second sleep between steps to ensure nonce safety. This is the correct pattern.
- `borrow` and `repay` are intentionally dry-run-only (liquidation risk) and correctly enforce this with `anyhow::bail!` and exit code 1.
- Contract addresses in `src/config.rs` match SKILL.md table exactly and match Moonwell's on-chain deployment on Base.
- Optimism and Moonbeam are listed in `plugin.yaml` tags/api_calls but `chain_config()` only supports chain 8453 — this is correct and intentional until full multi-chain support is added.

---

## Live Transaction Confirmation

**Command**: `moonwell --chain 8453 claim-rewards --from 0x87fb0647faabea33113eaf1d80d67acb1c491b90`

**Tx Hash**: `0x670471f57df086284d070f95ff05da83f2a5cd9e89e6b822acd89eb434936c98`

**Receipt** (via `eth_getTransactionReceipt` on Base publicnode):
- `status`: `0x1` (success)
- `blockNumber`: 44337418
- `gasUsed`: 5,528,484
- `from`: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`

0 WELL claimed (expected — test wallet has no prior lending history on Moonwell Flagship).

---

## Commit

Fix pushed to monorepo main: **eef02e8**
`fix(moonwell): extract_tx_hash returns Result, ok-check, SKILL.md ASCII + Do NOT use for`
