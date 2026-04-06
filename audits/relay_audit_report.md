# Skill Audit Report — Relay Cross-Chain Bridge

**Repo**: https://github.com/GeoGu360/onchainos-plugins/tree/main/relay
**Audit date**: 2026-04-06
**Test wallet**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
**Test chain**: Base (8453) → Ethereum (1)
**Auditor**: Claude Sonnet 4.6

---

## Summary

| Item | Result |
|------|--------|
| Compile | PASS |
| Commands tested | 5 / 5 |
| On-chain write ops | 1 confirmed |
| Issues found (pre-fix) | 2 P1, 3 P2 |
| Issues fixed | 2 P1 fixed |
| Fix commit | `c5a14116b221f2520abfccb8b5c5fef539e6f81f` |

---

## Step 0 — Environment

- Wallet ETH balance on Base at test start: `0.002773383044077846 ETH` (above 0.001 ETH minimum)
- onchainos version: connected and authenticated
- Rust toolchain: stable, cargo build --release succeeded in ~2m35s

---

## Step 2 — Test Plan

| # | Command | Type | Key Params | Test Input |
|---|---------|------|------------|-----------|
| 1 | `relay chains` | Read | `--filter` | (no filter) + `--filter base` |
| 2 | `relay currencies` | Read | `--chain` | `--chain 8453` |
| 3 | `relay quote` | Read | `--from-chain --to-chain --token --amount --from` | `8453 -> 1, ETH, 0.00005` |
| 4 | `relay bridge --dry-run` | Read | all bridge params | `8453 -> 1, ETH, 0.00005, --dry-run` |
| 5 | `relay bridge` | Write | all bridge params | `8453 -> 1, ETH, 0.00005` |
| 6 | `relay status` | Read | `--request-id` | request ID from bridge |
| 7 | Error handling | Read | oversized amount | `--amount 999999999` |

---

## Step 3 — Compile

```
cargo build --release
Finished `release` profile [optimized] target(s) in 2m 35s
```

Binary: `/tmp/onchainos-plugins/relay/target/release/relay`
Status: PASS

---

## Step 4 — Install

```
npx skills add /tmp/onchainos-plugins/relay/skills/relay --yes --global
```

Installed to: `~/.agents/skills/relay`
Status: PASS

---

## Step 5 — Command Test Results

| # | Command | Status | Tx Hash | On-chain Confirm | Notes |
|---|---------|--------|---------|-----------------|-------|
| 1 | `relay chains` | PASS | - | - | 74 chains returned including Ethereum, Base, Arbitrum |
| 2 | `relay chains --filter base` | PASS | - | - | 1 result: Base (8453), active |
| 3 | `relay currencies --chain 8453` | PASS | - | - | 8 tokens: USDC, WETH, DAI, DEGEN, WBTC, USDe, ETH, USDT |
| 4 | `relay quote --from-chain 8453 --to-chain 1 --token ETH --amount 0.00005 --from 0x87fb...` | PASS | - | - | Fee: 0.0000113 ETH, ~8s, receive 0.0000387 ETH |
| 5 | `relay bridge --dry-run ...` | PASS | - | - | Calldata and contract address shown correctly |
| 6 | `relay bridge --from-chain 8453 --to-chain 1 --token ETH --amount 0.00005` | PASS | `0xc773b614e8f440e11b7780e0c070dac81ea531ff3ed44c4e415e8dd7c4e4718f` | PASS block 44330492 | Bridge completed; relayer delivered ETH on Ethereum |
| 7 | `relay status --request-id 0x484faaae...` | PASS | - | - | Status: `success`, destination tx: `0xee3b483ed445841edc3e2527b97a799aada1f22c63417a97a8bf5bd2df3113ba` |
| 8 | `relay quote --amount 999999999` (error test) | PASS | - | - | Clear error: "Amount is higher than available liquidity. Max amount is $1,224,763 USD." |
| 9 | `relay quote --amount 0.0` (error test) | PASS | - | - | Clear error: "Amount must be greater than 0" |
| 10 | `relay currencies --chain 99999` (invalid chain) | PASS | - | - | Returns 0 tokens, no crash |

### Bridge Transaction Detail

```
Tx Hash   : 0xc773b614e8f440e11b7780e0c070dac81ea531ff3ed44c4e415e8dd7c4e4718f
Chain     : Base (8453)
Block     : 44330492
Status    : 1 (success)
Amount    : 0.00005 ETH (50000000000000 wei)
To        : 0x4cd00e387622c35bddb9b4c962c136462338bc31 (Relay router)
State change: 0.002773 ETH -> 0.002720 ETH on Base (delta ~0.000053 ETH = bridge amount + gas)

Bridge Status (request 0x484faaae48d06250206aa23ba45116f829af0a704d1f65a7d1b8cb45cc26ee92):
  Status: success
  Destination tx: 0xee3b483ed445841edc3e2527b97a799aada1f22c63417a97a8bf5bd2df3113ba
  Destination chain: Ethereum (1)
```

---

## Step 6 — Static Code Review

### 6a. SKILL.md Quality

- [x] `description` field in front matter is ASCII-only
- [x] Commands have parameter tables with examples
- [x] "Skill Routing" section provides redirection guidance
- [ ] Missing explicit "Do NOT use for" front-matter rule (P2)
- [ ] Body text uses Unicode symbols: `→` (U+2192), `≥` (U+2265), `—` (U+2014) — may cause display issues in some agents (P2)
- [ ] No Chinese-language trigger phrases — limits discoverability for Chinese-speaking users (P2)

### 6b. Code Quality

- [x] Contract addresses NOT hardcoded — dynamically returned from quote API (`steps[].items[].data.to`)
- [x] `onchainos wallet contract-call` used correctly with `--force` for write ops
- [x] `reqwest::blocking::Client` with 30s timeout — appropriate
- [x] Error messages are user-friendly (no raw panic/unwrap)
- [x] `anyhow::bail!` used consistently for error propagation
- [FIXED] Amount conversion previously always used 18 decimals regardless of token — now uses actual decimals from currencies API
- [FIXED] Non-ETH token symbols (USDC, USDT) previously resolved to ETH address — now resolved via currencies API

### 6c. ABI/Selector Verification

The plugin does not hardcode any function selectors — all calldata is returned by the Relay quote API. No ABI verification needed.

### Additional Observations

- `bridge.rs` only processes `steps[0]`. Relay's ERC-20 bridge may return multi-step flows (approve + deposit). If a user bridges an ERC-20 token requiring approval, only the deposit step is submitted. (P2 — not blocking for ETH-only use case, but limits ERC-20 robustness)
- `value_str.parse().unwrap_or(0)` for `value_wei` in bridge.rs silently defaults to 0 on parse failure. Should propagate the error. (P2)

---

## Issues Found

### P1 — Important Issues (Fixed)

**P1-1: Non-ETH token symbol resolved to ETH address**

- File: `src/commands/quote.rs:42`, `src/commands/bridge.rs:43`
- Description: `resolve_currency("USDC")` fell through to the fallback branch and returned `0x000...0000` (ETH address) instead of the USDC contract address. Any non-ETH symbol-based bridge would silently bridge ETH instead of the intended token.
- Fix: Replaced `resolve_currency()` helper with a call to `rpc::resolve_token(chain_id, symbol)` which fetches the Relay currencies API to look up the canonical address and decimals for the symbol on the given chain.
- Commit: `c5a14116b221f2520abfccb8b5c5fef539e6f81f`

**P1-2: Amount always converted with 18 decimals regardless of token**

- File: `src/commands/quote.rs:45`, `src/commands/bridge.rs:47`
- Description: `(args.amount * 1e18) as u128` hardcoded 18 decimals. For USDC/USDT (6 decimals), `--amount 1.0` would produce `1000000000000000000` (10^18) instead of `1000000` (10^6), causing the Relay API to reject the request or bridge a completely wrong amount.
- Fix: The new `rpc::resolve_token()` returns `(address, decimals)`. The amount is now scaled by `10^decimals`.
- Commit: `c5a14116b221f2520abfccb8b5c5fef539e6f81f`

### P2 — Improvement Suggestions (Not Fixed)

**P2-1: Multi-step ERC-20 bridge not handled**

- File: `src/commands/bridge.rs:103`
- Description: `bridge.rs` only processes `steps[0]`. ERC-20 tokens that require a prior approval step (some tokens on non-Base chains) would have the approval step skipped, causing the bridge deposit to fail on-chain.
- Suggestion: Iterate over all steps, executing each item sequentially. Check `step["id"]` for "approve" vs "deposit" and handle accordingly.

**P2-2: `value_str.parse().unwrap_or(0)` silently defaults**

- File: `src/commands/bridge.rs:123`
- Description: If the Relay API returns a non-numeric value string in `step.data.value`, the amount defaults to 0 wei. This would submit a bridge tx with no ETH value, which would fail on-chain without a clear error message.
- Suggestion: Use `.map_err(|e| anyhow::anyhow!("Invalid value field: {}", e))?` instead of `unwrap_or(0)`.

**P2-3: SKILL.md uses non-ASCII Unicode characters**

- File: `skills/relay/SKILL.md`
- 16 non-ASCII characters: `→` (U+2192 ×6), `—` (U+2014 ×8), `≥` (U+2265 ×1), `+` (U+FF0B ×1)
- Suggestion: Replace `→` with `->`, `—` with `-`, `≥` with `>=` for maximum compatibility.

**P2-4: No "Do NOT use for" guidance in SKILL.md front matter**

- Description: The SKILL.md lacks a "Do NOT use for" directive. An agent could mistakenly use `relay bridge` for same-chain swaps or Solana transfers.
- Suggestion: Add to front matter:
  ```
  Do NOT use for: same-chain token swaps (use uniswap/aerodrome), Solana or non-EVM chains, wallet balance queries (use onchainos wallet balance).
  ```

**P2-5: No Chinese trigger phrases**

- Description: SKILL.md has no Chinese-language triggers. Chinese-speaking users saying "跨链桥" or "转移ETH到以太坊" may not trigger this skill.
- Suggestion: Add trigger examples in Chinese.

---

## SKILL.md Improvement Suggestions

1. Add front-matter `do_not_use_for` field with explicit exclusions
2. Replace Unicode arrows/em-dashes with ASCII equivalents
3. Add Chinese trigger phrases: `跨链桥`, `relay桥`, `bridge ETH`, `转移ETH到以太坊`, `跨链转账`
4. Add `--from` parameter description note: "wallet address auto-resolved from onchainos if omitted"

---

## Code Improvement Suggestions

1. `src/commands/bridge.rs:103` — iterate all steps for multi-step ERC-20 support
2. `src/commands/bridge.rs:123` — propagate parse error instead of silently defaulting to 0
3. `src/commands/quote.rs` / `src/commands/bridge.rs` — add input validation: `--amount` must be > 0 before API call
4. `src/onchainos.rs` — `wallet_contract_call` uses `std::process::Command` synchronously inside an async function; consider `tokio::process::Command` for true async

---

## Auto-Fix Summary

| Issue | Severity | Status | Commit |
|-------|----------|--------|--------|
| Token symbol resolves to wrong address | P1 | Fixed | `c5a14116` |
| Amount always uses 18 decimals | P1 | Fixed | `c5a14116` |
| Multi-step ERC-20 not handled | P2 | Not fixed (out of scope) | - |
| value parse silent default | P2 | Not fixed | - |
| SKILL.md non-ASCII | P2 | Not fixed | - |
| SKILL.md missing Do NOT use | P2 | Not fixed | - |

**Push**: `c5a14116b221f2520abfccb8b5c5fef539e6f81f` and `af464ea` pushed to `GeoGu360/onchainos-plugins` main.

**plugin-store-community**: No `feat/relay` branch exists — nothing to update.

---

## Step 7 — Uninstall

```
npx skills remove relay --yes --global
```

Verification: `npx skills list -g | grep relay` returned no output. Uninstall confirmed.

---

## Final Assessment

The relay plugin is **functionally correct for native ETH bridging** (the primary use case). The on-chain bridge test executed cleanly: Base tx confirmed at block 44330492 (status=1), Relay solver delivered ETH on Ethereum within ~2 minutes. Error messages are clear and user-friendly.

The two P1 bugs (token symbol resolution and decimal scaling) affected only ERC-20 token bridging (USDC, USDT, etc.) and are now fixed. ETH-to-ETH bridging was never broken.

**Overall score**: 4/5 — solid ETH bridge, P1 ERC-20 path fixed, P2 items documented.
