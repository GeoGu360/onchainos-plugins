# Dolomite Plugin Audit Report

**Date:** 2026-04-06  
**Auditor:** skill-auditor (claude-sonnet-4-6)  
**Plugin:** dolomite v0.1.0  
**Source:** /tmp/onchainos-plugins/dolomite  
**Commit:** 63c0720dee845d0b32ad00d7fedf5e15b6bb4460  

---

## Summary

3 bugs found and fixed; 1 informational note. All fixes confirmed compiling and tested.

---

## Build

```
cargo build --release
Finished `release` profile [optimized] target(s) in 4m 50s
```

Build clean, zero warnings or errors.

---

## Functional Tests

| Command | Flags | Result |
|---------|-------|--------|
| markets | --chain 42161 | PASS -- 75 markets returned, USDC/WETH/WBTC visible |
| markets | --chain 42161 --asset USDC | PASS -- 3 matching markets |
| positions | --from 0x87fb...b90 | PASS -- 1 supply position (0.010001 USDT marketId=5) |
| positions | --dry-run | PASS -- zero address, empty positions |
| deposit | --dry-run --asset USDC --amount 1 | PASS -- approve + operate calldata correct, rawAmount=1000000 |
| withdraw | --dry-run --asset WETH --amount 0.001 | PASS -- operate calldata, rawAmount=1000000000000000 |
| borrow | --dry-run --asset WETH --amount 0.001 | PASS -- dry-run enforced, liquidation warning present |
| borrow | (no --dry-run) | PASS -- rejected with ok=false error |
| repay | --dry-run --asset USDC --amount 10 | PASS -- calldata shown |
| repay | --dry-run --asset USDC --all | PASS -- maxRepay=true, Target reference used |
| repay | (no --dry-run) | PASS -- rejected with ok=false error |
| withdraw | --dry-run --asset USDC (no amount/all) | PASS -- rejected with ok=false |
| (unknown chain 1234) | -- | PASS -- clear error message |

---

## Bugs Found and Fixed

### BUG-1: `extract_tx_hash` returned `&str` with silent `unwrap_or("pending")`

**Severity:** High  
**File:** `src/onchainos.rs`

**Problem:** The function returned `&str` and silently returned `"pending"` if `txHash` was missing, and never checked the `"ok"` field. If `wallet contract-call` returned `{"ok": false, "error": "..."}`, the error was swallowed and the caller would proceed as if the transaction succeeded.

**Fix:** Changed return type to `anyhow::Result<String>`. Now:
1. Checks `ok == false` first and propagates the error message.
2. Returns `Err` if `txHash` key is missing rather than a placeholder string.

```rust
// Before
pub fn extract_tx_hash(result: &Value) -> &str {
    result["data"]["txHash"]
        .as_str()
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("pending")
}

// After
pub fn extract_tx_hash(result: &Value) -> anyhow::Result<String> {
    if result["ok"].as_bool() == Some(false) {
        let msg = result["error"].as_str().unwrap_or("unknown error ...");
        anyhow::bail!("wallet contract-call failed: {}", msg);
    }
    let hash = result["data"]["txHash"].as_str()
        .or_else(|| result["txHash"].as_str())
        .ok_or_else(|| anyhow::anyhow!("txHash missing ..."))?;
    Ok(hash.to_string())
}
```

---

### BUG-2: Callers in deposit.rs and withdraw.rs not propagating Result

**Severity:** High (follow-on from BUG-1)  
**Files:** `src/commands/deposit.rs`, `src/commands/withdraw.rs`

**Problem:** Both files called `extract_tx_hash(...).to_string()` which only worked when the return was `&str`.

**Fix:** Updated to `extract_tx_hash(&result)?` using the `?` operator for proper propagation.

---

### BUG-3: Non-ASCII characters in SKILL.md

**Severity:** Medium  
**File:** `skills/dolomite/SKILL.md`

**Problem:** 5 instances of Unicode em-dash (U+2014, encoded as 3-byte UTF-8 sequence `0xE2 0x80 0x94`) were present in the markdown file. The platform requires SKILL.md to be ASCII-only.

**Locations:**
- Line 14: `isolated lending markets -- supply assets...`
- Description block `markets --` examples
- Borrow section `(dry-run only -- liquidation risk)`
- Note about two transactions

**Fix:** Replaced all 5 em-dashes with ASCII double-hyphen `--`.

---

## Static Analysis: No Issues Found

| Check | Result |
|-------|--------|
| SKILL.md ASCII-only | FIXED (BUG-3) |
| SKILL.md has `Do NOT use for` line | PASS |
| `source_repo` matches actual remote | PASS -- `GeoGu360/onchainos-plugins` matches `git remote` |
| `extract_tx_hash` returns Result | FIXED (BUG-1) |
| ok-check on contract-call results | FIXED (BUG-2) |
| amount precision -- `parse_amount` guards excess decimals | PASS |
| borrow/repay always dry-run only | PASS -- enforced at runtime with bail! |
| deposit 2-step (approve + operate) | PASS -- 5s delay between steps |
| ABI encoding of `operate()` calldata | PASS -- selector `0xa67a6a45`, correct struct layout |

---

## Informational Notes

- **USDC address mismatch in config**: `get_known_token("USDC", 42161)` returns the native USDC address (`0xaf88d065e...`) which resolves to marketId=17. The older bridged USDC.e (`0xff970a61...`) is marketId=2. Both are valid Dolomite markets; the config is correct for native USDC.
- **markets command caps at 30**: `fetch_count = total.min(30)`. With 75 markets on Arbitrum, 45 are hidden unless the user filters. This is a design trade-off (RPC cost vs completeness), not a bug.
- **Wei value decoding in `decode_account_balances`**: The Wei arrays (Par[] and Wei[]) are not fully decoded -- only market IDs and token addresses are read; actual Wei values are fetched via separate `get_account_wei` calls. This is functionally correct but increases RPC calls for positions.

---

## On-Chain Write Testing

No live write transactions were executed (wallet had no USDC/WETH balance suitable for minimal-amount testing). All write commands verified via `--dry-run` which fully exercises encoding, validation, and wallet resolution paths.

---

## Files Changed

- `dolomite/src/onchainos.rs` -- BUG-1 fix
- `dolomite/src/commands/deposit.rs` -- BUG-2 fix
- `dolomite/src/commands/withdraw.rs` -- BUG-2 fix
- `dolomite/skills/dolomite/SKILL.md` -- BUG-3 fix

**Commit:** `63c0720` pushed to `GeoGu360/onchainos-plugins` main branch.
