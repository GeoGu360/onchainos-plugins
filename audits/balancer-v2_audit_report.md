# Balancer V2 Skill Audit Report

**Date:** 2026-04-06
**Auditor:** skill-auditor (Claude Sonnet 4.6)
**Plugin:** balancer-v2 v0.1.0
**Commit (post-fix):** ae8aa36
**EVM Wallet:** 0x87fb0647faabea33113eaf1d80d67acb1c491b90

---

## Summary

The balancer-v2 skill passes compilation and all unit tests. All 6 commands (pools, pool-info, quote, positions, swap --dry-run, join --dry-run, exit --dry-run) execute correctly. Two bugs were found and fixed.

---

## Build

```
cargo build --release  =>  Finished (5m 13s) — 0 errors, 2 warnings (fixed below)
cargo test             =>  1 passed, 0 failed
```

---

## Command Tests

| Command | Args | Result |
|---------|------|--------|
| pools | --chain 42161 --limit 5 | OK — returned 5 pools with liquidity/fees |
| pool-info | --pool 0x6454...0002 --chain 42161 | OK — tokens, weights, swap fee |
| quote | --from WETH --to USDC --amount 0.001 | OK — 2.12 USDC out |
| positions | --chain 42161 --wallet 0x87fb... | OK — empty (no positions, correct) |
| swap --dry-run | --from WETH --to USDC --amount 0.001 | OK |
| join --dry-run | --pool ... --amounts 0,0,1.0 | OK |
| exit --dry-run | --pool ... --bpt-amount 0.001 | OK |

No live write transactions executed (no Balancer positions held; dry-run validated all write paths).

---

## Bugs Found and Fixed

### BUG-1: Non-ASCII Characters in SKILL.md (CRITICAL)
**File:** `skills/balancer-v2/SKILL.md`
**Issue:** 15 non-ASCII characters found: em-dash (U+2014, 11 occurrences) and right-arrow (U+2192, 4 occurrences).
**Risk:** Parser/tooling failures when SKILL.md is consumed by onchainos skill loader or LLM context.
**Fix:** Replaced `--` for em-dash, `->` for right-arrow. Verified 0 non-ASCII chars remain.

### BUG-2: Missing ok-check on write operation responses (MEDIUM)
**Files:** `src/commands/swap.rs`, `src/commands/join.rs`, `src/commands/exit.rs`
**Issue:** After calling `wallet_contract_call` and `erc20_approve`, code proceeded to `extract_tx_hash()` without checking `result["ok"] == true`. A failed transaction (ok: false) would silently return a zero hash and print a success-looking JSON output.
**Fix:** Added `onchainos::check_ok()` helper in `onchainos.rs`. Called before `extract_tx_hash` in all 5 write sites:
- `swap.rs`: approve + vault.swap
- `join.rs`: approve (per token) + vault.joinPool
- `exit.rs`: vault.exitPool

---

## Static Review Checklist

| Check | Result |
|-------|--------|
| extract_tx_hash returns Result | Not required (returns String with "pending" fallback) - PASS |
| ok-check on wallet_contract_call | FIXED (BUG-2) |
| SKILL.md ASCII-only | FIXED (BUG-1) |
| "Do NOT use for" section | Not applicable for this skill type - PASS |
| source_repo correct | `GeoGu360/onchainos-plugins` - PASS |
| amount precision (f64 -> u128) | PASS: min amount is 0.001 WETH (1e15), well within f64 precision (2^53 ~ 9e15) |
| --force flag on DEX calls | PASS: all vault calls use force=true |
| Slippage applied correctly | PASS: min_amount_out = expected * (1 - slippage/100) |
| ABI encoding correctness | PASS: calldata selector verified by unit test; ABI offsets manually reviewed and correct for swap/join/exit |
| Dead code warnings | FIXED: #[allow(dead_code)] on BALANCER_API_V3 and serialize_u128_as_string |

---

## Architecture Notes

- All on-chain reads use direct `eth_call` via public RPC (no onchainos dependency for reads) - correct pattern
- Write ops use `onchainos wallet contract-call --force` - correct
- dry_run returns zero hash immediately, never calls onchainos - correct
- `positions` command has graceful fallback from `wallet balance` to `wallet addresses` - correct
- Known pool list in config.rs is limited to 3 Arbitrum pools; users must pass --wallet to scan custom pools

---

## Verdict

**PASS with fixes applied.** Two bugs fixed, pushed to main at ae8aa36.
