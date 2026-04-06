# Benqi Skill Audit Report

**Date:** 2026-04-06
**Auditor:** skill-auditor (Claude Sonnet 4.6)
**Plugin path:** /tmp/onchainos-plugins/benqi
**Chain:** Avalanche C-Chain (43114)
**EVM wallet:** 0x87fb0647faabea33113eaf1d80d67acb1c491b90

---

## 1. Build

```
cargo build --release
```

Result: SUCCESS. Initial build produced 4 warnings (dead code). Post-fix: 0 warnings.

---

## 2. SKILL.md Review

- All ASCII: PASS
- Frontmatter complete (name, description, license, metadata): PASS
- No "Do NOT use for" clause present: NOTE - not required by spec for this plugin type
- Commands documented: markets, positions, supply, redeem, borrow (dry-run), repay (dry-run), claim-rewards
- Dry-run vs execute distinction clearly documented: PASS
- borrow and repay clearly labeled DRY-RUN ONLY: PASS

---

## 3. plugin.yaml Review

| Field | Value | Status |
|-------|-------|--------|
| source_repo | GeoGu360/onchainos-plugins | PASS (matches convention) |
| source_commit | 5e32ab6db42925a2aa1a18cd4411bd61b2fa3146 | PASS |
| binary_name | benqi | PASS |
| api_calls | https://avalanche-c-chain-rpc.publicnode.com | PASS |
| chain ID | 43114 | PASS |

---

## 4. Command Tests

| Command | Result | Notes |
|---------|--------|-------|
| markets | PASS | Returns 8 markets with supply/borrow APR and exchange rates |
| positions --wallet 0x87fb... | PASS | Returns empty positions, no error |
| --dry-run supply --asset USDC --amount 0.01 | PASS | Correct approve+mint calldata, raw_amount=10000 (6 decimals) |
| --dry-run supply --asset AVAX --amount 0.001 | PASS | Correct payable mint calldata, value_wei=1000000000000000 |
| --dry-run redeem --asset USDC --amount 0.01 | PASS | redeemUnderlying calldata correct |
| borrow --asset USDC --amount 1.0 | PASS | Dry-run only, no on-chain tx |
| repay --asset AVAX --amount 0.001 | PASS | Dry-run only, correct payable repayBorrow |
| --dry-run claim-rewards --reward-type 0 | PASS | claimReward(0, wallet) calldata |
| --dry-run claim-rewards --reward-type 1 | PASS | claimReward(1, wallet) calldata |
| supply --asset AVAX --amount 0.001 (live) | PASS (expected fail) | ok=false, error: insufficient funds; exit code 1 |
| --chain 1 markets | PASS (expected fail) | ok=false, unsupported chain |
| supply --asset INVALID | PASS (expected fail) | ok=false, unknown asset error |

---

## 5. Bugs Found and Fixed

### BUG-1: CRITICAL - extract_tx_hash returns String, not Result

**File:** src/onchainos.rs  
**Severity:** Critical  
**Description:** `extract_tx_hash()` returned `String` unconditionally, using `"pending"` as fallback. If a `contract-call` returned `ok: false` (transaction simulation failure, gas estimation failure, etc.), the function silently returned `"pending"` instead of propagating the error. The calling code would then set `ok: true` in the response, hiding the failure from the caller.

**Fix:** Changed signature to `fn extract_tx_hash(result: &Value) -> anyhow::Result<String>`. Now:
1. If `result["ok"] == false`, bails with the error message from the response
2. If `txHash` is missing/empty, bails with an explicit error
3. Returns `Ok(hash)` on success

**Callers updated (5 sites):**
- supply.rs: AVAX mint path (line 70)
- supply.rs: ERC20 approve path (line 125)
- supply.rs: ERC20 mint path (line 133)
- redeem.rs: redeemUnderlying path (line 63)
- claim_rewards.rs: claimReward path (line 66)

### BUG-2: MINOR - Dead code warnings (4 unused functions in rpc.rs)

**File:** src/rpc.rs  
**Severity:** Minor (warnings, no functional impact)  
**Description:** Four utility functions lacked `#[allow(dead_code)]`:
- `get_all_markets`
- `decode_address_array`
- `eth_call_raw`
- `erc20_balance_of`

**Fix:** Added `#[allow(dead_code)]` to each. These are utility functions intended for future use.

### BUG-3: MISSING - .gitignore absent

**File:** (new file) benqi/.gitignore  
**Severity:** Minor (repo hygiene)  
**Description:** The benqi plugin lacked a `.gitignore`, causing `cargo build` artifacts in `target/` to be staged and committed. All other plugins (e.g., aave-v3) have this file.

**Fix:** Added `.gitignore` with `target/` and `/target/`. Removed 1084 accidentally-committed build artifacts.

---

## 6. Static Analysis Checklist

| Check | Result |
|-------|--------|
| extract_tx_hash returns Result | FIXED |
| ok-check on contract-call results | FIXED (via Result propagation) |
| SKILL.md all ASCII | PASS |
| No "Do NOT use for" in SKILL.md | N/A (not required) |
| source_repo correct | PASS |
| amount precision (to_raw uses round()) | PASS - uses `(amount * factor).round() as u128` |
| f64 usage for amounts | ACCEPTABLE - uses .round() to mitigate precision loss |
| borrow/repay are truly dry-run only | PASS - no wallet_contract_call in these paths |
| Chain validation on every command | PASS - all commands check chain_id == 43114 |
| Wallet resolution fallback chain | PASS - 3-level fallback in onchainos.rs |
| 3s delay between approve and mint | PASS |
| Error responses use {ok: false, error: ...} | PASS |
| Exit code 1 on error | PASS |

---

## 7. Write Operation Test

Attempted `benqi supply --asset AVAX --amount 0.001` with live wallet.

- Lock acquired: YES (queue position 5, waited ~8 minutes)
- Transaction attempted: YES
- Result: `ok: false`, error: `insufficient funds for gas * price + value: address 0x87fB... have 0 want 1000000000000000`
- Exit code: 1
- Receipt check: N/A (transaction was rejected at simulation, never broadcast)
- Behavior with fix: Error correctly propagated; old code would have set `ok: true` with `txHash: "pending"`

---

## 8. Commits Pushed

```
52e7569 fix(benqi): extract_tx_hash returns Result + ok-check + suppress dead_code warnings
6250d0d fix(benqi): add .gitignore to exclude target/ build artifacts
```

Pushed to: `git@github.com:GeoGu360/onchainos-plugins.git` (main)

---

## 9. Summary

The benqi plugin is well-structured and functionally correct for all read operations and dry-run preview modes. One critical bug was found and fixed: `extract_tx_hash` silently swallowed failed transaction results. Two minor issues were also corrected (dead code warnings and missing .gitignore). After fixes, the plugin builds cleanly with zero warnings and all 9 command tests pass.

**Overall verdict: APPROVED after fixes**
