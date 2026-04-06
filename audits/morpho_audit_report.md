# Morpho Plugin Audit Report

**Date:** 2026-04-06  
**Auditor:** skill-auditor (Claude Sonnet 4.6)  
**Plugin:** morpho v0.1.0  
**Repo:** GeoGu360/onchainos-plugins  
**Commit audited / fixed:** d0ba4d4

---

## Test Wallet

**EVM Address:** `0x87fb0647faabea33113eaf1d80d67acb1c491b90`  
**Chain used for write tests:** Base (8453) — had USDC balance (0.257885 USDC)

---

## Command Test Results

| Command | Type | Status | Notes |
|---------|------|--------|-------|
| `morpho --chain 1 markets` | Read | PASS | 47 markets returned, data looks correct |
| `morpho --chain 1 markets --asset USDC` | Read | PASS | 20 USDC markets filtered correctly |
| `morpho --chain 8453 markets --asset USDC` | Read | PASS | 40 USDC markets on Base |
| `morpho --chain 1 vaults` | Read | PASS | 50 vaults with APY and TVL |
| `morpho --chain 1 vaults --asset USDC` | Read | PASS | 17 USDC vaults filtered |
| `morpho --chain 1 --from 0x87fb... positions` | Read | PASS | Blue positions and vault positions returned |
| `morpho --chain 8453 --dry-run supply ...` | Dry-run | PASS | Correct calldata emitted |
| `morpho --chain 1 --dry-run borrow ...` | Dry-run | PASS | Correct calldata for market; rawAmount correct via GraphQL decimals |
| `morpho --chain 1 --dry-run supply-collateral ...` | Dry-run | PASS | Correct calldata emitted |
| `morpho --chain 1 --dry-run repay ...` | Dry-run | PASS (not tested on-chain) | |
| `morpho --chain 1 --dry-run claim-rewards` | Dry-run | N/A | Merkl API returned 500 (server error) |
| `morpho --chain 8453 supply --vault ... --asset USDC --amount 0.1` | Write | **PASS** | See below |
| `morpho --chain 8453 withdraw --vault ... --asset USDC --amount 0.05` | Write | **PASS** | See below |
| `morpho --chain 1 supply-collateral --market-id ... --amount 0.00003` | Write | SKIP | Wallet has no matching collateral token balance; simulation reverts |
| `morpho --chain 1 repay ...` | Write | SKIP | No active borrow position |
| `morpho --chain 1 borrow ...` | Write | SKIP | No collateral deposited; risky |
| `morpho --chain 8453 claim-rewards` | Write | N/A | Merkl API 500 error |

---

## On-Chain Confirmations

### Supply (Base chain 8453)
- **Vault:** `0xc1256Ae5FF1cf2719D4937adb3bbCCab2E00A2Ca` (Moonwell Flagship USDC)
- **Amount:** 0.1 USDC
- **Approve Tx:** `0x3f042f09a2fc912a68d81106db6f85e6df9db00fcb268f03813a1c07e989676b`
  - Status: **1 (success)**, Block: `0x2a474bf`
- **Supply Tx:** `0x5b0485a1df2a6a36df095845a1ce99f911a03b159ff900142e814eece194a1f7`
  - Status: **1 (success)**, Block: `0x2a474c1`

### Withdraw (Base chain 8453)
- **Vault:** `0xc1256Ae5FF1cf2719D4937adb3bbCCab2E00A2Ca`
- **Amount:** 0.05 USDC
- **Tx:** `0xd7d516ded5971063627c9c33f49be92d65244d018ed16465db668e73070a2073`
  - Status: **1 (success)**, Block: `0x2a474cc`

---

## Issues Found and Fix Status

### 1. `wallet_contract_call` did not check exit code or `ok` field
- **Severity:** High
- **Description:** On onchainos CLI failure (non-zero exit code), the function would attempt to JSON-parse stdout and produce a confusing JSON parse error rather than the actual error message. Also did not check `ok: false` in the response.
- **Fix:** Added exit code check (`!output.status.success()`) and `ok` field validation. Now produces clear error messages.
- **Status:** FIXED in `src/onchainos.rs`

### 2. `extract_tx_hash` silently returned `"pending"` placeholder
- **Severity:** High
- **Description:** The function returned `&str` and fell back to `"pending"` if txHash was absent, causing callers to silently record `"pending"` as a real hash.
- **Fix:** Changed return type to `anyhow::Result<String>`. Returns `Err` for missing hash and explicitly for `"pending"` value. All call sites updated with `?` propagation.
- **Status:** FIXED in `src/onchainos.rs` and all 5 command files

### 3. `erc20_decimals` silently defaulted to 18 on RPC failure
- **Severity:** Critical
- **Description:** On RPC error or rate-limit, the function returned `Ok(18)` instead of `Err`. For USDC (6 decimals), this would cause amounts to be inflated by 10^12 — approving/depositing 1 trillion times the intended amount.
- **Fix:** Changed to propagate the error. All callers updated from `.unwrap_or(18)` to `?`.
- **Status:** FIXED in `src/rpc.rs` and all 5 command files

### 4. Unreliable RPC endpoint (`eth.llamarpc.com`)
- **Severity:** Medium
- **Description:** `eth.llamarpc.com` was frequently returning "Too many connections" / empty bodies during testing, triggering the now-strict decimals check.
- **Fix:** Switched to `ethereum.publicnode.com` which was consistently available.
- **Status:** FIXED in `src/config.rs`

### 5. SKILL.md description contained non-ASCII em dash
- **Severity:** Low
- **Description:** The frontmatter `description` field contained `—` (U+2014 em dash) which violates the ASCII-only rule.
- **Fix:** Replaced `—` with ` - ` (ASCII hyphen).
- **Status:** FIXED in `skills/morpho/SKILL.md`

### 6. SKILL.md missing "Do NOT use for" rules
- **Severity:** Low
- **Description:** No guidance on when NOT to trigger this skill.
- **Fix:** Added `. Do NOT use for: Aave, Compound, Uniswap, non-Morpho lending protocols, or generic token swaps.` to description.
- **Status:** FIXED in `skills/morpho/SKILL.md`

### 7. SKILL.md `positions --from` example incorrect
- **Severity:** Low
- **Description:** SKILL.md showed `morpho --chain 1 positions --from 0xYourAddress` but `--from` is a global flag that must precede the subcommand. The correct form is `morpho --chain 1 --from 0xYourAddress positions`.
- **Fix:** Updated the example in SKILL.md.
- **Status:** FIXED in `skills/morpho/SKILL.md`

### 8. `erc20_symbol` not called with `?` — still silently falls back to "UNKNOWN"
- **Severity:** Low (cosmetic)
- **Description:** Symbol resolution still uses `.unwrap_or_else(|_| "UNKNOWN")` which causes display outputs to show `UNKNOWN` on RPC failures. This is acceptable for display-only use since the actual operation uses the token address.
- **Status:** NOT FIXED — cosmetic issue, does not affect correctness

---

## Static Code Review Checklist

| Check | Result |
|-------|--------|
| SKILL.md description ASCII-only | FIXED (was using em dash `—`) |
| SKILL.md has "Do NOT use for" rules | FIXED (added) |
| `wallet_contract_call` checks exit code | FIXED |
| `wallet_contract_call` checks `ok` field | FIXED |
| `extract_tx_hash` returns `Result<String>` | FIXED |
| `extract_tx_hash` rejects `"pending"` | FIXED |
| `plugin.yaml source_repo` correct | PASS (`GeoGu360/onchainos-plugins`) |
| Amount precision conversion correct | PASS (uses on-chain `decimals()` call, now strict) |
| Dry-run emits correct calldata | PASS |
| SKILL.md `--from` flag position | FIXED |

---

## Improvement Suggestions

1. **Cache token decimals / symbols** — Each command makes multiple sequential eth_call RPCs. Caching within a single invocation (or using GraphQL `decimals` field from the API response which is already fetched) would eliminate the RPC dependency.

2. **Use GraphQL `decimals` instead of eth_call** — The GraphQL API already returns `decimals` for each asset in market/vault responses. Using it would make write commands work without a separate RPC call for decimals, making them more robust.

3. **`positions` should display health factor** — The SKILL.md documents health factor output, but `positions.rs` does not compute or display it. The collateral and borrow amounts are present but HF calculation is missing.

4. **`borrow.rs` and `supply_collateral.rs` use `unwrap_or` for borrower/supplier address** — They fall back to the zero address if `from` is None, instead of resolving the active wallet. This is inconsistent with `supply.rs` and `withdraw.rs` which call `resolve_wallet()`. Should use `resolve_wallet()` for consistency.

5. **Multi-step tx delay** — `supply` adds a 3s sleep between approve and deposit to avoid nonce conflicts. This pattern should be applied to `repay` and `supply-collateral` as well.
