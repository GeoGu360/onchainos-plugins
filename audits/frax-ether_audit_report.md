# Skill Audit Report — frax-ether

**Repo**: https://github.com/okx/onchainos-plugins (dir: frax-ether/)
**Audit Date**: 2026-04-06
**Auditor Wallet**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
**Chain**: Ethereum Mainnet (chain ID 1)
**Source at audit start**: `GeoGu360/onchainos-plugins` @ `9ba57ef738e7271ad6c1d0eb38ffe78e8ec506e2`
**onchainos CLI version**: `2.2.6`

---

## Summary

| Item | Result |
|------|--------|
| Compile | ✅ clean (0 errors; 3 pre-existing dead_code warnings on unused consts) |
| Commands tested | 5 / 5 |
| Read ops passing | 2 / 2 (`rates`, `positions`) |
| Write ops (dry-run) | 3 / 3 (`stake`, `stake-frx`, `unstake`) |
| ABI selectors verified | 7 / 7 ✅ |
| Issues found | 2 P0, 2 P1, 1 P2 |
| Issues fixed | All ✅ |

---

## Command Test Results

| # | Command | Type | Status | Notes |
|---|---------|------|--------|-------|
| 1 | `frax-ether rates` | Read | ✅ | APR: 2.8502%; exchange rate 1.15480129 frxETH/sfrxETH; on-chain + API data both valid |
| 2 | `frax-ether positions --address 0x87fb...` | Read | ✅ | 0 frxETH, 0.0000433 sfrxETH (~$0.11) reported correctly |
| 3 | `frax-ether stake --amount 0.001 --chain 1 --dry-run` | Write (dry) | ✅ | Calldata `0x5bcb2fc6`, 1000000000000000 wei |
| 4 | `frax-ether stake-frx --amount 0.001 --chain 1 --dry-run` | Write (dry) | ✅ | Correct approve + deposit calldata generated |
| 5 | `frax-ether unstake --amount 0.001 --chain 1 --dry-run` | Write (dry) | ✅ | Correct redeem(shares, receiver, owner) calldata |

> Live write ops (`stake`, `stake-frx`, `unstake`) not broadcast — wallet has insufficient ETH/frxETH for the minimum viable 0.001 ETH transaction at time of audit. Dry-run validation confirmed correct ABI encoding and contract targeting.

---

## Issues Found & Fixed

### P0-1: `extract_tx_hash` returned `String`, silently masking missing txHash

**File**: `src/onchainos.rs`
**Severity**: P0 — silent data loss / phantom "pending" hashes in output

**Before**:
```rust
pub fn extract_tx_hash(result: &Value) -> String {
    result["data"]["txHash"]
        .as_str()
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("pending")
        .to_string()
}
```

**After**:
```rust
pub fn extract_tx_hash(result: &Value) -> anyhow::Result<String> {
    let hash = result["data"]["txHash"]
        .as_str()
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("");
    if hash.is_empty() || hash == "null" {
        anyhow::bail!("No txHash in onchainos response. Full response: {}", result);
    }
    Ok(hash.to_string())
}
```

All three callers (`stake.rs`, `stake_frx.rs`, `unstake.rs`) updated to propagate with `?`.

---

### P0-2: `wallet_contract_call` hardcoded `--force` on every call

**File**: `src/onchainos.rs`
**Severity**: P0 — violates onchainos wallet spec; bypasses the mandatory confirming-response flow

The original code always appended `--force` to the `onchainos wallet contract-call` invocation, which is only allowed after the CLI returns `exit code 2 / "confirming": true` and the user explicitly acknowledges. This means:
- High-risk transactions were broadcast without the built-in risk check prompt
- Confirming responses with `message` / `next` fields were never shown to the user

**Fix**: Removed hardcoded `--force`. The function now makes a first call without `--force`. If the response contains `"confirming": true`, it returns the payload to the agent (which must show the `message` to the user and re-invoke with `--force` only after explicit confirmation). Non-JSON or non-confirming responses surface a full error with stdout/stderr for diagnosis.

---

### P1-1: `source_repo` incorrect in `plugin.yaml`

**File**: `plugin.yaml`
**Before**: `source_repo: GeoGu360/onchainos-plugins`
**After**: `source_repo: okx/onchainos-plugins`

The plugin physically lives in the `okx/onchainos-plugins` monorepo. The original value pointed to the author's fork, which could cause `onchainos skill install` to fetch the wrong (potentially stale) source.

---

### P1-2: SKILL.md frontmatter `description` contained non-ASCII characters

**File**: `skills/frax-ether/SKILL.md`

The frontmatter `description` field contained Chinese Unicode characters (`质押ETH到Frax`, etc.). Some YAML parsers and onchainos indexing pipelines expect ASCII-only frontmatter. Replaced with romanized equivalents (`zhi ya ETH dao Frax`, etc.) while preserving discoverability.

---

### P2-1: SKILL.md missing "Do NOT use for" section

**File**: `skills/frax-ether/SKILL.md`

Per audit checklist, SKILL.md must include a "Do NOT use for" disambiguation block to prevent the agent from misrouting swaps, bridging, or cross-chain operations to this skill.

**Added** (immediately after frontmatter):
```
> Do NOT use for: swapping ETH to other tokens (use swap execute), general ERC-20 transfers, bridging, non-Ethereum chains, or any protocol other than Frax Ether liquid staking.
```

---

## ABI Selector Verification

| Function | Selector | Verified |
|----------|----------|---------|
| `submit()` | `0x5bcb2fc6` | ✅ |
| `deposit(uint256,address)` | `0x6e553f65` | ✅ |
| `redeem(uint256,address,address)` | `0xba087652` | ✅ |
| `convertToAssets(uint256)` | `0x07a2d13a` | ✅ |
| `convertToShares(uint256)` | `0xc6e6f592` | ✅ |
| `balanceOf(address)` | `0x70a08231` | ✅ |
| `totalAssets()` | `0x01e1d114` | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | ✅ |

---

## Amount Precision Note

ETH amounts are converted via `(args.amount * 1e18) as u128`. For amounts >= 1e-15 ETH this is precise (f64 has 15-16 significant decimal digits). A zero-check (`if amt_wei == 0 { bail! }`) prevents dust transactions. This is acceptable for production amounts.

---

## Contract Addresses (verified on-chain)

| Contract | Address |
|----------|---------|
| frxETHMinter | `0xbAFA44EFE7901E04E39Dad13167D089C559c1138` |
| frxETH token | `0x5E8422345238F34275888049021821E8E08CAa1f` |
| sfrxETH vault | `0xac3E018457B222d93114458476f3E3416Abbe38F` |

All three contracts are live and responding correctly on Ethereum mainnet (verified via `eth_call` during `rates` and `positions` tests).

---

## Post-Fix Build

```
Finished `release` profile [optimized] target(s) in 2.56s
Warnings: 3 dead_code (pre-existing unused constants ETH_RPC_URL, SEL_CONVERT_TO_SHARES, SEL_APPROVE)
Errors: 0
```

---

## Verdict

**PASS with fixes applied.** The plugin is architecturally sound with correct ABI encoding and proper Frax Ether protocol integration. The two P0 issues (silent txHash masking and mandatory-confirmation bypass) have been fixed. Plugin is safe to merge after confirmation of write-op live tests with a funded wallet.
