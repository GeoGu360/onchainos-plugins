# Skill Audit Report — Synthetix V3

**Repo**: https://github.com/GeoGu360/onchainos-plugins (synthetix-v3/)
**Audit Date**: 2026-04-06
**Test Wallet (EVM)**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
**Test Wallet (Solana)**: `DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE`
**Test Chain**: Base (8453)

---

## Summary

| Item | Result |
|------|--------|
| Compilation | ✅ (with 20 initial warnings, reduced to 12 after fixes) |
| Commands Tested | 5 / 5 |
| Commands Passing | 5 / 5 |
| Live Write Ops | 1 partial (approve tx confirmed, deposit failed — no sUSDC balance) |
| Issues Found | 6 |
| Issues Fixed | 6 |

---

## Command Test Results

| # | Command | Type | Status | Tx Hash | On-chain | Notes |
|---|---------|------|--------|---------|----------|-------|
| 1 | `markets` | Read | ✅ | - | - | Returns 4 markets with funding rates; total_markets=108 |
| 2 | `markets --market-id 100` | Read | ✅ | - | - | ETH market: skew=14.02, funding_rate=0.10972436 |
| 3 | `positions --account-id 1234567890` | Read | ✅ | - | - | Returns empty positions for non-existent account |
| 4 | `collateral --account-id 1234567890` | Read | ✅ | - | - | Returns empty collaterals for non-existent account |
| 5 | `--dry-run deposit-collateral --account-id 1234567890 --amount 1.0` | Dry-run | ✅ | - | - | Returns calldata preview correctly |
| 6 | `--dry-run withdraw-collateral --account-id 1234567890 --amount 1.0` | Dry-run | ✅ | - | - | Returns calldata preview correctly |
| 7 | `withdraw-collateral --account-id 1234567890 --amount 1.0` | Write (error path) | ✅ | - | - | Pre-flight check correctly rejects: "Insufficient available collateral" |
| 8 | `deposit-collateral --account-id 1234567890 --amount 999999` | Write (partial) | ✅ | `0xc10813a5121c0febe04a4d4752ddf956e1569d7e7ca434bf3d41270a63a7a748` (approve) | ✅ status=1, block 44338844 | Approve confirmed on-chain; deposit step failed (no sUSDC balance). Full round-trip not possible without sUSDC. |

**Live write test note**: The wallet holds 0.2 USDC on Base but 0 sUSDC. Synthetix V3 requires sUSDC (obtained via SpotMarket wrap), which this plugin does not support. A full round-trip deposit-collateral → collateral-query → withdraw-collateral could not be performed. The approve step did execute successfully on-chain, proving the onchainos integration works.

---

## Issues Found and Fixed

### P0 — Blocking Issues

**1. `resolve_wallet` crashes with JSON parse error (broken before any write op)**

- **File**: `src/onchainos.rs:7-14`
- **Root cause**: Used `--output json` flag which is not a valid `onchainos wallet balance` argument. This caused the command to exit with an error and produce no stdout. The subsequent `serde_json::from_str("")` call panicked with "EOF while parsing a value at line 1 column 0". Also, the code expected `json["data"]["address"]` but the actual API response structure is `data.details[0].tokenAssets[0].address`.
- **Impact**: All write operations (deposit-collateral, withdraw-collateral) crash immediately.
- **Fix**: Removed `--output json`, updated JSON path to `data.details[0].tokenAssets[0].address`, added graceful `map_err` on JSON parse, added clear error message if address not found.
- **Status**: ✅ Fixed (commit `1fbe77a`)

**2. `extract_tx_hash` silently returns `"pending"` on failure**

- **File**: `src/onchainos.rs:69-76`
- **Root cause**: Function returned `String`, falling back to `"pending"` if no hash found. This would silently propagate an invalid hash into the output JSON without any error.
- **Impact**: Failed transactions would appear to succeed with a `"pending"` tx hash.
- **Fix**: Changed return type to `Result<String>`, bails with descriptive error if hash is empty or `"pending"`.
- **Status**: ✅ Fixed (commit `1fbe77a`)

### P1 — Important Issues

**3. `wallet_contract_call` does not check exit code or `ok` field**

- **File**: `src/onchainos.rs:37-65`
- **Root cause**: After calling `onchainos wallet contract-call`, the code did not check `output.status.success()` or `result["ok"]`. A failed contract call would silently proceed.
- **Impact**: Errors from onchainos would be swallowed; code would continue as if the transaction succeeded.
- **Fix**: Added exit code check (bail on non-zero), added `ok` field check (bail if `ok == false`).
- **Status**: ✅ Fixed (commit `1fbe77a`)

**4. `markets` command: `size` and `max_open_interest` fields swapped**

- **File**: `src/commands/markets.rs:111-116`
- **Root cause**: `size_f` was computed as `max_oi as f64 / 1e18` instead of `size as f64 / 1e18`. The `size` variable (decoded from `size_raw`, the 3rd ABI slot) was unused. Both fields showed `max_open_interest` value.
- **Impact**: Users querying market size would see incorrect data (max OI shown for both size and max OI fields).
- **Verification**: Before fix, BTC market showed size=0.8200 (same as max_oi). After fix, BTC shows size=0.0000, max_oi=0.8200 (correct — actual open positions vs max allowed).
- **Fix**: Changed `size_f` to use `size as f64 / 1e18`.
- **Status**: ✅ Fixed (commit `1fbe77a`)

**5. SKILL.md description contains CJK characters (violates ASCII-only rule)**

- **File**: `skills/synthetix-v3/SKILL.md:3`
- **Root cause**: Description field ended with `"Chinese: Synthetix市场, Synthetix持仓, Synthetix存款, Synthetix抵押品"` — embedded CJK characters in the frontmatter description.
- **Impact**: May cause encoding issues in agent systems that process skill metadata as ASCII.
- **Fix**: Replaced with additional English trigger phrase descriptors.
- **Status**: ✅ Fixed (commit `1fbe77a`)

**6. SKILL.md missing "Do NOT use for" disambiguation rule**

- **File**: `skills/synthetix-v3/SKILL.md`
- **Root cause**: No `Do NOT use for` section to prevent mistaken invocation for Synthetix V2, Optimism deployments, or general swap operations.
- **Impact**: Skill may be incorrectly triggered for Synthetix V2 or other chains.
- **Fix**: Added `Do NOT use for: general token swaps, Synthetix V2 (legacy), staking SNX on Ethereum, Synthetix on Optimism or Arbitrum unless specified, or non-Synthetix DeFi protocols.`
- **Status**: ✅ Fixed (commit `1fbe77a`)

---

## Code Warnings (Remaining)

After fixes, 12 compiler warnings remain (down from 20). All are `dead_code` warnings on constants and helper functions in `config.rs` and `rpc.rs` that appear reserved for future use:

- `config.rs`: `BASE_CHAIN_ID`, `ACCOUNT_PROXY`, `USD_PROXY`, `SPOT_MARKET_PROXY`, `PERPS_ACCOUNT_PROXY`, `USDC`, `SPARTAN_COUNCIL_POOL_ID`, `PERPS_SUPER_MARKET_ID`, `USDC_DECIMALS`
- `rpc.rs`: `decode_uint128`, `decode_int256`, `erc20_balance_of`

These are not blocking but could be suppressed with `#[allow(dead_code)]` or removed if truly unused.

---

## ABI Selector Verification

All function selectors verified with `cast sig`:

| Function | Expected | Actual |
|----------|----------|--------|
| `deposit(uint128,address,uint256)` | `0x83802968` | ✅ Match |
| `withdraw(uint128,address,uint256)` | `0x95997c51` | ✅ Match |
| `getAccountAvailableCollateral(uint128,address)` | `0x927482ff` | ✅ Match |
| `getAccountCollateral(uint128,address)` | `0xef45148e` | ✅ Match |
| `getMarkets()` | `0xec2c9016` | ✅ Match |
| `getMarketSummary(uint128)` | `0x41c2e8bd` | ✅ Match |
| `getAccountOpenPositions(uint128)` | `0x35254238` | ✅ Match |
| `getOpenPosition(uint128,uint128)` | `0x22a73967` | ✅ Match |
| `getAvailableMargin(uint128)` | `0x0a7dad2d` | ✅ Match |
| `approve(address,uint256)` | `0x095ea7b3` | ✅ Match |

---

## Static Code Review Notes

- `plugin.yaml` `source_repo: GeoGu360/onchainos-plugins` — correct, matches actual repo remote
- `amount` uses `f64` input then converts to `u128` raw. For values up to ~9.2 × 10^18 (sUSDC with 18 decimals) this is fine; f64 precision loss could occur for amounts >2^53 tokens (~9 quadrillion sUSDC), which is not a practical concern.
- `wallet_contract_call` `amt: Option<u64>` — unused for collateral ops; using `u64` is acceptable here since it's for ETH-value (not token amount), and Base ETH values don't exceed u64 range.
- The 5-second `std::thread::sleep` between approve and deposit in `deposit_collateral.rs` is a pragmatic approach but may fail on congested networks. A future improvement could poll for tx confirmation instead.
- Markets command silently skips markets requiring ERC-7412 Pyth price feed updates — appropriate behavior with a clear code comment.

---

## P2 — Improvement Suggestions

1. **Add sUSDC wrapping command**: Users need to wrap USDC → sUSDC via SpotMarket before depositing. A `wrap-usdc` command would complete the user journey.
2. **Suppress or remove dead_code items**: Either use `#[allow(dead_code)]` on reserved constants or remove them to clean up compiler output.
3. **Replace sleep with confirmation polling**: The 5s sleep between approve and deposit is brittle; polling for receipt would be more reliable.
4. **Add `--account-id` discovery**: Users may not know their Synthetix V3 account ID. A `list-accounts` or `create-account` command querying the AccountProxy would improve usability.
5. **ERC-7412 support**: Most markets fail to return data due to Pyth price feed staleness requirement (ERC-7412). The markets command shows only 4/108 markets. Implementing the multicall-based ERC-7412 fulfillment pattern would expose all markets.
