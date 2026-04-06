# Stader Plugin Audit Report

**Date:** 2026-04-06  
**Auditor:** skill-auditor (Claude Sonnet 4.6)  
**Plugin:** stader v0.1.0  
**Repo:** GeoGu360/onchainos-plugins  
**Commit after fixes:** f9d6681

---

## Test Wallet

| Chain | Address |
|-------|---------|
| EVM (Ethereum Mainnet) | `0x87fb0647faabea33113eaf1d80d67acb1c491b90` |

---

## Command Test Results

| Command | Type | Status | Tx Hash | Block Confirmed | Notes |
|---------|------|--------|---------|-----------------|-------|
| `stader rates` | Read | PASS | n/a | n/a | Returns exchange rate 1 ETHx=1.086235 ETH, total staked 135,442 ETH, vault healthy=true |
| `stader rates --preview-amount 1000000000000000000` | Read | PASS | n/a | n/a | Consistent with default run |
| `stader positions --address 0x87fb...` | Read | PASS | n/a | n/a | Returns ETHx balance 0.00009, eth_value 0.00010, 0 pending withdrawals |
| `stader --dry-run stake --amount 100000000000000` | Dry-run | PASS | 0x000...000 | n/a | Returns calldata with selector 0xf340fa01 |
| `stader --dry-run unstake --amount 1000000000000000000` | Dry-run | PASS | 0x000...000 | n/a | Returns step1 (approve) + step2 (requestWithdraw) calldatas |
| `stader --dry-run claim --request-id 12345` | Dry-run | PASS | 0x000...000 | n/a | Returns calldata with selector 0x379607f5 |
| `stader stake --amount 1000000000000000 --receiver 0x87fb...` | Write | PASS | `0x741204f0faac62f0e28b259e882db1c7524a5c7829b24742fb5509c2677ca169` | Block 24,886,146 (0x17ab382), status=0x1 | 0.001 ETH staked, ETHx minted to receiver confirmed via Transfer log |

**Write operations skipped (require pre-existing state):**
- `unstake` — requires ETHx balance (only 0.00009 ETHx in wallet; protocol minimum not specified for unstake but this amount is very small)
- `claim` — requires a finalized withdrawal request ID

---

## Issues Found & Fix Status

### 1. wallet_contract_call does not check exit code or ok field
**Severity:** HIGH  
**Status:** FIXED (commit f9d6681)  
**Description:** The function called `onchainos wallet contract-call` but never checked `output.status.success()`, so a non-zero exit code was silently ignored. Likewise, it did not check the `ok` field in the parsed JSON response, meaning a failed transaction (ok=false) would be treated as success.  
**Fix:** Added exit-code check that bails with stderr/stdout context; added `ok` field check that extracts and surfaces the `error` message.

### 2. extract_tx_hash returns String and silently falls back to "pending"
**Severity:** HIGH  
**Status:** FIXED (commit f9d6681)  
**Description:** `extract_tx_hash` returned `String` and used `.unwrap_or("pending")`, so callers could silently get `"pending"` as a tx hash and proceed to print it as if the transaction succeeded.  
**Fix:** Changed return type to `anyhow::Result<String>`. Now returns `Err` for empty, `"pending"`, or zero-hash values. All callers updated to propagate with `?`.

### 3. stake --amount uses u64 instead of u128
**Severity:** MEDIUM  
**Status:** FIXED (commit f9d6681)  
**Description:** `StakeArgs.amount` was typed as `u64`, capping maximum staking at ~18.4 ETH (u64::MAX wei). The `unstake` command correctly uses `u128`. Large-amount stakers (>18 ETH) would get a clap parse error.  
**Fix:** Changed `amount` to `u128` in `StakeArgs`. Also changed `wallet_contract_call` `amt` parameter from `Option<u64>` to `Option<u128>` for consistency.

### 4. SKILL.md description contains non-ASCII Chinese characters
**Severity:** LOW  
**Status:** FIXED (commit f9d6681)  
**Description:** The `description` field contained Chinese characters (`质押ETH到Stader`, etc.), which violates the ASCII-only requirement for SKILL.md description fields.  
**Fix:** Replaced Chinese text with ASCII pinyin transliterations.

### 5. SKILL.md missing "Do NOT use for" rules
**Severity:** LOW  
**Status:** FIXED (commit f9d6681)  
**Description:** No disambiguation rules to prevent the skill from triggering on Lido/Rocket Pool queries or L2 staking requests.  
**Fix:** Added "Do NOT use for" clause covering: L2 staking, other liquid staking protocols, DEX swaps of ETHx, cross-chain bridging.

---

## No Issues Found

- `plugin.yaml source_repo`: Correctly set to `GeoGu360/onchainos-plugins`
- Amount precision: All wei arithmetic uses `u128` (post-fix), and `format_eth` correctly divides by 1e18 with 5 decimal places
- Minimum deposit guard: Correctly enforced at 100000000000000 wei (0.0001 ETH)
- Contract addresses: Match official Stader docs (StaderStakePoolsManager, UserWithdrawManager, ETHx Token)
- Function selectors: Verified correct (deposit=0xf340fa01, requestWithdraw=0xccc143b8, claim=0x379607f5, approve=0x095ea7b3)
- Dry-run handling: Correctly returns early before any onchainos calls

---

## Code Improvement Suggestions (not blocking)

1. **Parallel RPC calls in `rates`**: The five `eth_call` reads in `rates.rs` are sequential. Using `tokio::join!` would reduce latency.
2. **Retry logic for RPC**: No retry on transient RPC failures. A simple 1-retry would improve reliability on flaky public nodes.
3. **Unstake minimum**: The unstake command does not validate a minimum ETHx amount. The Stader contract has a `minWithdrawAmount` — consider checking it before submitting.
4. **approve tx confirmation**: The 5-second sleep between approve and requestWithdraw is fragile. Polling `eth_getTransactionReceipt` until the approve is confirmed would be more robust.
