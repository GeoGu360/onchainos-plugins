# Clanker Plugin Audit Report

**Date:** 2026-04-06
**Auditor:** skill-auditor (Claude Sonnet 4.6)
**Plugin:** clanker v0.1.0
**Source:** /tmp/onchainos-plugins/clanker
**Commit after fixes:** e6b05ca

---

## Summary

The clanker plugin passed compilation and all read/dry-run command tests. Four static issues were found and fixed. No write operations were executed on-chain (plugin uses Clanker REST API for deploy, which requires a real partner API key; claim-rewards tested via dry-run only).

**Result: PASS (after fixes)**

---

## Build

```
cargo build --release
Finished `release` profile [optimized] target(s) in 5m 17s
```

8 dead-code warnings (unused structs/functions defined for future use or deserialization). No errors.

---

## Test Results

| Command | Result | Notes |
|---------|--------|-------|
| `list-tokens --limit 5` | PASS | Returns 5 tokens, total=654270, has_more=true |
| `list-tokens --chain 8453 --sort asc --limit 3` | PASS | Correct output shape |
| `search-tokens --query dwr --limit 3` | PASS | Returns 3 tokens, searched_address resolved |
| `token-info --address 0x1D008f50...` | PASS | Returns info + price data |
| `--dry-run deploy-token --name AuditTest --symbol AUDIT --api-key test --from 0x87fb...` | PASS | Preview with request_key UUID |
| `--dry-run claim-rewards --token-address 0x1D008... --from 0x87fb...` | PASS | Correct calldata encoded, fee locker resolved |

No live write operations were performed (no real API key for deploy; claim-rewards tested dry-run only).

---

## Bugs Found and Fixed

### Bug 1 (CRITICAL): `extract_tx_hash` silently swallows contract-call failures

**File:** `src/onchainos.rs`

**Before:**
```rust
pub fn extract_tx_hash(result: &Value) -> &str {
    result["data"]["txHash"]
        .as_str()
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("pending")
}
```

**Problem:** Returns the string `"pending"` if the contract-call failed or returned `ok=false`. The call site in `claim_rewards.rs` would silently write `"pending"` into the output JSON and report `ok: true`, giving the user a false success response while no transaction occurred.

**Fix:** Changed to return `anyhow::Result<String>`, checks `result["ok"]` first and bails with the error message from the response. Also rejects empty or `"pending"` hash values with an actionable error.

**File:** `src/commands/claim_rewards.rs`

**Before:**
```rust
let tx_hash = onchainos::extract_tx_hash(&result);
```

**Fix:** Changed to `let tx_hash = onchainos::extract_tx_hash(&result)?;` so the error propagates correctly.

---

### Bug 2 (MINOR): SKILL.md contains non-ASCII characters

**File:** `skills/clanker/SKILL.md`

**Problem:** SKILL.md contained 120 non-ASCII bytes: Unicode arrows (`->`/U+2192), em-dashes (U+2014), en-dashes (U+2013), and Chinese characters in trigger phrases and troubleshooting table. Plugin store tooling requires strict ASCII.

**Fix:**
- `->` (U+2192) replaced with `->` (ASCII)
- ` - ` (em-dash with spaces, U+2014) replaced with ` - ` (ASCII hyphen)
- `0-90` range (en-dash U+2013) replaced with `0-90` (ASCII)
- Chinese trigger phrases (`bu shu dai bi`, etc.) romanized in place

---

### Bug 3 (MINOR): SKILL.md missing required `Do NOT use for` section

**File:** `skills/clanker/SKILL.md`

**Problem:** The plugin store convention requires a `## Do NOT use for` section to help the AI avoid triggering this skill for unrelated queries. The section was absent.

**Fix:** Added `## Do NOT use for` section listing: general token swaps/trading, cross-chain price queries, deployment without API key, and non-Clanker DeFi operations.

---

## Static Review (No Issues Found)

| Check | Status | Notes |
|-------|--------|-------|
| `source_repo` correct | OK | `GeoGu360/onchainos-plugins` matches git remote |
| Amount precision | OK | `wallet_contract_call` amt is `Option<u64>` (wei); claim-rewards passes `None` (no ETH value sent) |
| `--dry-run` not passed to `onchainos wallet contract-call` | OK | Dry-run handled before CLI call; comment in code confirms |
| `--force` used for reward claim | OK | `force=true` prevents stuck `pending` txHash |
| Security scan before claim | OK | `security_token_scan` called and risk level checked |
| API key not logged | OK | API key read from flag/env, never printed in output |
| `request_key` UUID per call | OK | Prevents accidental double-deployment |
| Fee locker resolved dynamically | OK | Factory lookup with fallback to hardcoded V4 locker |
| RPC calls are read-only | OK | eth_call used; no signing via RPC |
| Error output on stderr, exit code 1 | OK | `eprintln!` + `std::process::exit(1)` |

---

## Warnings (Not Fixed - Acceptable)

8 dead-code warnings in `api.rs` and `onchainos.rs` for structs/functions defined as API types or utilities not yet called. These are forward-looking definitions and pose no runtime risk.

---

## Files Changed

- `src/onchainos.rs` - `extract_tx_hash` now returns `Result<String>`
- `src/commands/claim_rewards.rs` - propagate error with `?`
- `skills/clanker/SKILL.md` - ASCII-only, added `Do NOT use for` section
