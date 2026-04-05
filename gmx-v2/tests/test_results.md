# GMX V2 Plugin — Test Results

- **Date:** 2026-04-05
- **Test chain:** Arbitrum (42161)
- **Compile:** PASS
- **Lint:** PASS

---

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Fail | Blocked | Skipped |
|-------|-----------|---------|-------------|-------------|------|---------|---------|
| 13    | 2         | 4       | 7           | 0           | 0    | 0       | 3 (L4)  |

---

## Detailed Results

| # | Scenario (User View) | Level | Command | Result | Calldata / TxHash | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Plugin compiles to release binary | L1 | `cargo build --release` | PASS | — | 12 dead-code warnings only, no errors |
| 2 | Plugin passes store lint checks | L1 | `cargo clean && plugin-store lint .` | PASS | — | "passed all checks!" |
| 3 | User views all active GMX V2 markets | L2 | `gmx-v2 --chain arbitrum list-markets` | PASS | — | 122 markets returned; name, liquidity, OI, rates all present |
| 4 | User queries current ETH oracle price | L2 | `gmx-v2 --chain arbitrum get-prices --symbol ETH` | PASS | — | ETH midPrice_usd ~$2059.77; price bug fixed (was showing $0.000000) |
| 5 | User views open positions for test wallet | L2 | `gmx-v2 --chain arbitrum get-positions --address 0xee385...` | PASS | — | Returns `ok:true`, positions:[] (test wallet has no positions) |
| 6 | User views pending orders for test wallet | L2 | `gmx-v2 --chain arbitrum get-orders --address 0xee385...` | PASS | — | Returns `ok:true`, orders:[] (test wallet has no orders) |
| 7 | User dry-runs opening ETH long position | L3 | `gmx-v2 --chain arbitrum --dry-run open-position --market "ETH/USD" --collateral-token 0xaf88d065e77c8cC2239327C5EDb3A432268e5831 --collateral-amount 1000000 --size-usd 2.0 --long --from 0xee385ac7ac70b5e7f12aa49bf879a441bed0bae9` | PASS | `0xac9650d8...` (outer multicall); inner: `7d39aaf1` (sendWnt), `e6d66ac8` (sendTokens), `97aedce2` (createOrder) | ETH/USD market, $2 size, 2x leverage, executionFee=0.001 ETH |
| 8 | User dry-runs opening ETH short position | L3 | Same as #7 without `--long` | PASS | `0xac9650d8...`; inner: `7d39aaf1`, `e6d66ac8`, `97aedce2` | Short direction, calldata ~1445 bytes |
| 9 | User dry-runs depositing USDC into ETH/USD GM pool | L3 | `gmx-v2 --chain arbitrum --dry-run deposit-liquidity --market "ETH/USD" --short-amount 1000000 --from 0xee385...` | PASS | `0xac9650d8...`; inner: `7d39aaf1` (sendWnt), `e6d66ac8` (sendTokens USDC), `adc567e6` (createDeposit) | depositVault=0xF89e77...; min-market-tokens=0 |
| 10 | User dry-runs withdrawing from ETH/USD GM pool | L3 | `gmx-v2 --chain arbitrum --dry-run withdraw-liquidity --market-token 0x70d95587d40A2caf56bd97485aB3Eec10Bee6336 --gm-amount 1000000000000000000 --from 0xee385...` | PASS | `0xac9650d8...`; inner: `7d39aaf1`, `e6d66ac8` (sendTokens GM), `9b8eb9e7` (createWithdrawal) | withdrawalVault=0x0628D4... |
| 11 | User dry-runs closing ETH long position | L3 | `gmx-v2 --chain arbitrum --dry-run close-position --market-token 0x70d95... --collateral-token 0xaf88... --size-usd 2.0 --collateral-amount 1000000 --long --from 0xee385...` | PASS | `0xac9650d8...`; inner: `7d39aaf1`, `97aedce2` (createOrder MarketDecrease) | $2 close, orderType=3 (MarketDecrease) |
| 12 | User dry-runs canceling a pending order | L3 | `gmx-v2 --chain arbitrum --dry-run cancel-order --key 0x1234...abcd --from 0xee385...` | PASS | `0x7489ec23...` (direct cancelOrder call) | Direct call, not multicall-wrapped |
| 13 | User dry-runs claiming funding fees | L3 | `gmx-v2 --chain arbitrum --dry-run claim-funding-fees --markets 0x70d95... --tokens 0xaf88... --receiver 0xee385... --from 0xee385...` | PASS | `0xc41b1ab3...` (direct claimFundingFees call) | Dynamic array ABI-encoded correctly |
| L4 | User opens small ETH market order on-chain | L4 | — | SKIPPED | — | Execution fee 0.001 ETH (20x GUARDRAILS limit of 0.00005 ETH). No L4 tests run. |

---

## L4 Skip Justification

GMX V2 uses a **keeper model** requiring pre-payment of an execution fee in native token (ETH/AVAX). The minimum fee is hard-coded in the protocol at:

- **Arbitrum (42161):** `0.001 ETH` = 1,000,000,000,000,000 wei
- **Avalanche (43114):** `0.012 AVAX`

The GUARDRAILS test limit is **0.00005 ETH** (50,000,000,000,000 wei). The Arbitrum execution fee is **20x over the limit**. Per the tester protocol: "If execution fee > 0.00005 ETH, mark L4 as SKIPPED."

L4 on-chain tests (create-order, create-deposit, create-withdrawal) are **all SKIPPED** for this reason.

---

## Bug Fixes Applied

| # | Problem | Root Cause | Fix | File |
|---|---------|-----------|-----|------|
| 1 | `get-prices` returned `midPrice_usd: "0.000000"` for all tokens | GMX oracle prices use `price_usd * 10^(30 - token_decimals)` precision; code divided by `10^30` always producing near-zero for tokens with <30 decimals | Added `fetch_tokens()` to get token decimals from `/tokens` API endpoint; added `raw_price_to_usd(raw, decimals)` helper; updated `get_prices.rs` to use proper conversion | `src/api.rs`, `src/commands/get_prices.rs` |
| 2 | `open_position.rs` displayed wrong entry price USD | Same root cause: `mid_price_usd` computed with `/ 1e30` showing 0.000 | Updated `open_position.rs` to fetch token decimals and use `raw_price_to_usd()` for display | `src/commands/open_position.rs` |

---

## Known Issues / Non-Bugs

| # | Issue | Severity | Notes |
|---|-------|----------|-------|
| 1 | SKILL.md examples show `--long true` syntax | Minor doc mismatch | The clap CLI uses `--long` as a presence flag (no value). `--long true` fails with "unexpected argument 'true'". SKILL.md examples should use `--long` alone. Does not affect runtime correctness when used correctly. |
| 2 | `get-positions` / `get-orders` include raw `eth_call` hex in output | Minor UX | The `"raw"` field exposes undecoded contract response bytes. Could be removed from user-facing output. |
| 3 | Execution fee 0.001 ETH hardcoded | Protocol constraint | GMX V2 keeper model requires this minimum fee. Cannot be reduced for testing. |
