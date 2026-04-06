# Swell Staking Skill Audit Report

**Date:** 2026-04-06
**Auditor:** skill-auditor (Claude Code / claude-sonnet-4-6)
**Plugin path:** `/tmp/onchainos-plugins/swell-staking`
**Source repo:** `GeoGu360/onchainos-plugins`
**Commit after fixes:** `953783a`

---

## Test Wallet Address

| Network | Address |
|---------|---------|
| EVM (Ethereum mainnet) | `0x87fb0647faabea33113eaf1d80d67acb1c491b90` |

---

## Step 3: Compilation

| Result | Details |
|--------|---------|
| SUCCESS | `cargo build --release` completed with 4 dead_code warnings only (unused constants: `CHAIN_ID`, `ETH_RPC_URL`, `SEL_DEPOSIT_WITH_REFERRAL`, `SEL_TOTAL_SUPPLY`) |

---

## Step 5: Command Test Results

### Read-Only / Off-Chain Commands

| Command | Status | Notes |
|---------|--------|-------|
| `swell-staking rates` | PASS | Returns live swETH rate ~1.119 ETH/swETH, rswETH ~1.069 ETH/rswETH. JSON valid. |
| `swell-staking positions --address 0x87fb...` | PASS | Returns swETH balance 0.000044 swETH, rswETH balance 0.000093 rswETH with ETH values. |
| `swell-staking stake --amount 0.001 --dry-run` | PASS | Returns ok=true, dry_run=true, zero txHash. |
| `swell-staking restake --amount 0.001 --dry-run` | PASS | Returns ok=true, dry_run=true, zero txHash. |

### On-Chain Write Commands

| Command | Status | Tx Hash | Block | Chain Confirmed | Notes |
|---------|--------|---------|-------|-----------------|-------|
| `stake --amount 0.001 --from 0x87fb...` | PASS | `0x0874cc6a6270db76a5fc29d5deef3e27b5f2e694eeed4692ec1399f1a8674a14` | `0x17ab379` (24abb257) | status=0x1 | swETH Transfer event emitted; ~0.000893 swETH minted |
| `restake --amount 0.001 --from 0x87fb...` | PASS | `0x64d68f5296cf26df2a4ccd90388eef4eb694d1e6d2e7881edabe6e2d512463ea` | `0x17ab37a` (24abb258) | status=0x1 | rswETH Transfer event emitted; ~0.000935 rswETH minted |

Etherscan links:
- Stake: https://etherscan.io/tx/0x0874cc6a6270db76a5fc29d5deef3e27b5f2e694eeed4692ec1399f1a8674a14
- Restake: https://etherscan.io/tx/0x64d68f5296cf26df2a4ccd90388eef4eb694d1e6d2e7881edabe6e2d512463ea

---

## Step 6: Static Code Review — Issues Found & Fix Status

| # | Severity | File | Issue | Status |
|---|----------|------|-------|--------|
| 1 | HIGH | `SKILL.md` (frontmatter) | `description` contained non-ASCII Chinese characters (`Swell质押ETH`, `获取swETH`, etc.), violating ASCII-only requirement | FIXED |
| 2 | HIGH | `SKILL.md` | Missing required "Do NOT use for" section | FIXED |
| 3 | HIGH | `src/onchainos.rs` | `wallet_contract_call` did not check process exit code or `ok` field; a failed transaction would silently parse garbage JSON and return | FIXED |
| 4 | HIGH | `src/onchainos.rs` | `extract_tx_hash` returned `String` and silently fell back to `"pending"` instead of returning `Result<String>` and propagating failure | FIXED |
| 5 | MEDIUM | `src/commands/rates.rs` | `description` fields used raw string literals with `{var}` placeholders that appeared literally in output (not Rust format strings). e.g. `"1 ETH = ~{swETH_per_ETH} swETH"` | FIXED |
| 6 | LOW | `src/rpc.rs` | `decode_uint256` truncates values > u128::MAX silently. Exchange rate / balance values fit in u128 (rates are ~1e18, balances for audit amounts are <1e18) but large whale balances could silently overflow | ADVISORY (no fix applied — would require `U256` dep) |
| 7 | INFO | `src/config.rs` | `plugin.yaml source_repo` is correctly set to `GeoGu360/onchainos-plugins` | OK |
| 8 | INFO | `src/config.rs` | Unused constants `CHAIN_ID`, `ETH_RPC_URL`, `SEL_DEPOSIT_WITH_REFERRAL`, `SEL_TOTAL_SUPPLY` generate dead_code warnings | ADVISORY |

All HIGH/MEDIUM issues fixed. Commit `953783a` pushed to `main`.

---

## SKILL.md Improvement Summary

- **Frontmatter description:** Changed to ASCII-only; Chinese trigger phrases rephrased in English.
- **Added "Do NOT use for" section** covering: unstaking, non-mainnet chains, Lido staking, generic DEX swaps.
- Existing routing section retained.

---

## Code Improvement Summary

- **`wallet_contract_call`:** Now checks `output.status.success()` and `json["ok"] == true`; fails fast with descriptive errors.
- **`extract_tx_hash`:** Returns `anyhow::Result<String>`; callers in `stake.rs` and `restake.rs` propagate with `?`.
- **`rates.rs`:** Description strings now use `format!()` to embed live rate values.
- **Advisory — `decode_uint256` u128 truncation:** For full correctness with whale-scale positions a `U256` crate (e.g. `primitive-types`) would be needed. Not a concern for sub-1 ETH amounts tested here.
