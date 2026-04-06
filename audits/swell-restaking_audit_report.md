# Skill Audit Report — Swell Restaking (rswETH)

**Repo**: https://github.com/GeoGu360/onchainos-plugins (path: `swell-restaking/`)
**Audit date**: 2026-04-06
**Auditor model**: claude-sonnet-4-6
**Test wallet (EVM)**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
**Test chain**: Ethereum mainnet (chain 1)

---

## Summary

| Item | Result |
|------|--------|
| Compilation | ✅ |
| Commands tested | 5 / 5 |
| Read commands passing | 3 / 3 |
| Live write ops | 1 success |
| Bugs fixed | 1 |
| SKILL.md issues fixed | 1 |
| Remaining issues | 1 (P2) |

---

## Test Plan

| # | Command | Type | Key Parameters | Test Input |
|---|---------|------|----------------|------------|
| 1 | `get-rates` | Read | none | — |
| 2 | `get-positions` | Read | `--address` | test wallet |
| 3 | `get-positions` (no addr) | Read | none (wallet resolution) | — |
| 4 | `stake --dry-run` | Dry-run | `--amount`, `--dry-run` | 0.00005 ETH |
| 5 | `stake` (live) | Write | `--amount`, `--from` | 0.001 ETH |

---

## Step 3: Compilation

```
cargo build --release
Finished `release` profile [optimized] target(s) in 2m 34s
```

Binary: `./target/release/swell-restaking`

**Result: ✅ Compiled cleanly, zero warnings.**

---

## Step 5: Command Test Results

| # | Command | Status | Tx Hash | Notes |
|---|---------|--------|---------|-------|
| 1 | `get-rates` | ✅ | — | ETH_per_rswETH: 1.069097, rswETH_per_ETH: 0.93536, total_eth_deposited: 147006, total_supply: 14930 |
| 2 | `get-positions --address ...` | ✅ | — | Returns rswETH balance + ETH equivalent; pre-stake: 0.001028 rswETH |
| 3 | `get-positions` (no arg) | ✅ | — | Wallet auto-resolved via `onchainos wallet balance --chain 1` |
| 4 | `stake --amount 0.00005 --dry-run` | ✅ | 0x000...000 | Shows calldata `0xd0e30db0`, amount_wei, ok:true |
| 5 | `stake --amount 0.001` (live) | ✅ | `0x0b09cc8a0ebe08a8e30211928367c4cb63afeffc858d00e55138a85e3425aa0c` | Received 0.000935367 rswETH; confirmed on-chain |

**Error handling:**
- `stake --amount 0` → exit 1 with `Error: Stake amount must be greater than 0` — clean
- `--chain 5 get-rates` → warns "only supports chain 1", then exits with `Error: Unsupported chain_id for eth_call: 5` — acceptable

### Live Stake Verification

**Pre-stake balance:** 0.001028923767632844 rswETH  
**Tx hash:** `0x0b09cc8a0ebe08a8e30211928367c4cb63afeffc858d00e55138a85e3425aa0c`  
**Etherscan:** https://etherscan.io/tx/0x0b09cc8a0ebe08a8e30211928367c4cb63afeffc858d00e55138a85e3425aa0c  
**Status:** Confirmed (status 0x1, block 0x17ab7dd)  
**Gas used:** 0x15fda (90074 gas)  
**rswETH minted:** 0.000935367956763914 rswETH (matches `ethToRswETHRate` prediction exactly)  
**Post-stake balance:** 0.001964291724396758 rswETH (+0.000935 rswETH ✅)

---

## Step 6: Static Code Review

### 6a. SKILL.md Quality

| Check | Result |
|-------|--------|
| description is ASCII-only | ✅ No CJK characters |
| Trigger phrases cover common phrasings | ✅ 9 phrases including English variants |
| "Do NOT use for" rule | ⚠️ Missing — **Fixed** (added 6 routing rules) |
| All commands have parameter examples | ✅ |
| Minimum amount documented | ✅ (0.00005 ETH) |

### 6b. Code Quality

| Check | Result |
|-------|--------|
| Contract address hardcoded | Acceptable — verified on Etherscan, single immutable contract |
| amount precision (ETH → wei) | ✅ Correct 18-decimal parse/format |
| `onchainos wallet contract-call` usage | ✅ Correct: `--chain 1 --to ... --input-data ... --amt ... --force` |
| ERC-20 approve needed? | N/A — ETH-in, no token approval required |
| `ok`-check after contract-call | ❌ Missing — **Fixed** |
| Error messages user-friendly | ✅ No panics; anyhow errors printed cleanly |
| Proxy support in HTTP client | ✅ Reads HTTPS_PROXY / HTTP_PROXY env vars |

### 6c. ABI / Selector Verification

All selectors verified with `cast sig`:

| Function | Expected | In Code | Status |
|----------|----------|---------|--------|
| `deposit()` | `0xd0e30db0` | `d0e30db0` | ✅ |
| `rswETHToETHRate()` | `0xa7b9544e` | `a7b9544e` | ✅ |
| `ethToRswETHRate()` | `0x780a47e0` | `780a47e0` | ✅ |
| `totalETHDeposited()` | `0x7b2c9070` | `7b2c9070` | ✅ |
| `balanceOf(address)` | `0x70a08231` | `70a08231` | ✅ |
| `totalSupply()` | `0x18160ddd` | `18160ddd` | ✅ |

---

## Bugs Found and Fixed

### Fix 1 — ok-check missing in `stake` command (P1)

**File:** `src/commands/stake.rs`

**Problem:** After calling `onchainos wallet contract-call`, the stake command did not check `result["ok"]` before constructing a success response. If onchainos returned `{"ok": false, "error": "insufficient funds"}`, the plugin would still output `{"ok": true, "txHash": "pending", ...}` — a false positive.

**Fix:** Added `ok` field check immediately after receiving the result. If `ok` is not `true`, the function bails with the error message from the onchainos response.

```rust
// Added after wallet_contract_call returns:
let ok = result["ok"].as_bool().unwrap_or(false);
if !ok {
    let err_msg = result["error"]
        .as_str()
        .or_else(|| result["message"].as_str())
        .unwrap_or("unknown error");
    anyhow::bail!("Transaction failed: {}", err_msg);
}
```

### Fix 2 — SKILL.md missing "Do NOT use for" routing guard (P2)

**File:** `skills/swell-restaking/SKILL.md`

**Problem:** No explicit "Do NOT use for" section. Agent routing could accidentally invoke this skill for swETH staking, Lido, unstaking, or other chains.

**Fix:** Added a "Do NOT use for" section with 6 clear exclusion rules before the existing "Skill Routing" section.

---

## Remaining Issues

### P2 — `totalETHDeposited` is all-time cumulative, not current TVL

**File:** `src/commands/get_rates.rs`

**Observation:** The `get-rates` output shows `total_eth_deposited: 147006 ETH` vs `total_supply: 14930 rswETH`. At the current rate of 1.069 ETH/rswETH, the implied current TVL would be ~15,960 ETH — far less than 147,006 ETH. Investigation confirms that the `totalETHDeposited()` function on the rswETH contract returns a **cumulative all-time deposit counter**, not the current pool TVL.

**Impact:** Display is technically accurate (it is what the contract returns) but highly misleading — users might assume 147,006 ETH is current TVL when the real pool value is ~15,960 ETH.

**Recommendation:** Add a label or note clarifying this is "all-time cumulative deposits" not current TVL. Alternatively, derive current TVL as `totalSupply * ETH_per_rswETH` and display it separately:

```rust
// Recommended addition to get_rates output:
"current_tvl_eth": format_eth(total_supply_wei * rsweth_to_eth_rate / 1e18),
"total_eth_deposited_note": "all-time cumulative; not current TVL",
```

---

## SKILL.md Assessment

| Criterion | Score |
|-----------|-------|
| Description clarity | ✅ Clear, correct |
| Trigger phrase coverage | ✅ 9 relevant phrases |
| Command documentation | ✅ All 3 commands documented |
| Parameter tables | ✅ Present for all write ops |
| Error table | ✅ 4 error cases documented |
| Do NOT use routing | ✅ Fixed (added) |
| ASCII-only description | ✅ |

---

## Overall Assessment

**Rating: 4/5 — Production ready with minor caveats**

The plugin is well-structured, clean Rust code with correct ABI selectors, proper proxy support, and solid documentation. The live stake of 0.001 ETH executed successfully on-chain, received the exactly predicted amount of rswETH, and gas usage was reasonable (~90k gas). The single P1 bug (missing ok-check) has been fixed. The only remaining issue is a display clarity concern about `totalETHDeposited` semantics — functionally the plugin works correctly end-to-end.
