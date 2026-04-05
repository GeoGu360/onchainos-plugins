# GMX V2 Plugin — Test Cases

**DApp:** GMX V2  
**Chain:** Arbitrum (42161)  
**Date:** 2026-04-05  
**Binary:** `gmx-v2`

---

## Test Pyramid Overview

```
        L4: On-chain writes    → SKIPPED (execution fee 0.001 ETH > 0.00005 ETH limit)
        L3: Dry-run simulations → 7 tests
        L2: Read queries        → 4 tests
        L1: Compile + Lint      → 1 test
```

---

## L1 — Compile + Lint

| # | Test | Command | Expected |
|---|------|---------|----------|
| 1 | Compile release binary | `cargo build --release` | Exit 0, binary created |
| 2 | Lint plugin store | `cargo clean && plugin-store lint .` | "passed all checks!" |

---

## L2 — Read Tests (no wallet, no gas)

All L2 tests use Arbitrum (chain 42161), no wallet required.

| # | User Scenario | Command | Expected |
|---|--------------|---------|----------|
| L2-1 | User asks "what GMX markets are available?" | `gmx-v2 --chain arbitrum list-markets` | JSON with `count > 0`, markets array with name/liquidity/OI fields |
| L2-2 | User asks "what is the ETH price on GMX?" | `gmx-v2 --chain arbitrum get-prices --symbol ETH` | JSON with ETH price ~$2000, `midPrice_usd` non-zero |
| L2-3 | User asks "show my open positions on GMX" | `gmx-v2 --chain arbitrum get-positions --address 0xee385ac7ac70b5e7f12aa49bf879a441bed0bae9` | JSON `ok:true`, positions array (empty ok for test wallet) |
| L2-4 | User asks "show my pending orders on GMX" | `gmx-v2 --chain arbitrum get-orders --address 0xee385ac7ac70b5e7f12aa49bf879a441bed0bae9` | JSON `ok:true`, orders array (empty ok for test wallet) |

---

## L3 — Dry-Run Simulation Tests (calldata verification)

All L3 tests verify multicall calldata. Outer selector is always `ac9650d8` (multicall) except for `claim-funding-fees` which uses direct call `c41b1ab3`.

| # | User Scenario | Command | Expected Calldata |
|---|--------------|---------|------------------|
| L3-1 | User wants to open ETH long position (dry-run) | `gmx-v2 --chain arbitrum --dry-run open-position --market "ETH/USD" --collateral-token 0xaf88... --collateral-amount 1000000 --size-usd 2.0 --long --from 0xee385...` | `0xac9650d8...` outer; inner: `7d39aaf1` (sendWnt), `e6d66ac8` (sendTokens), `97aedce2` (createOrder) |
| L3-2 | User wants to open ETH short position (dry-run) | Same without `--long` flag | Same structure, createOrder with `isLong=false` |
| L3-3 | User wants to deposit USDC into ETH/USD GM pool (dry-run) | `gmx-v2 --chain arbitrum --dry-run deposit-liquidity --market "ETH/USD" --short-amount 1000000 --from 0xee385...` | `0xac9650d8...` outer; inner: `7d39aaf1`, `e6d66ac8`, `adc567e6` (createDeposit) |
| L3-4 | User wants to withdraw from ETH/USD GM pool (dry-run) | `gmx-v2 --chain arbitrum --dry-run withdraw-liquidity --market-token 0x70d95... --gm-amount 1000000000000000000 --from 0xee385...` | `0xac9650d8...` outer; inner: `7d39aaf1`, `e6d66ac8`, `9b8eb9e7` (createWithdrawal) |
| L3-5 | User wants to close an ETH long position (dry-run) | `gmx-v2 --chain arbitrum --dry-run close-position --market-token 0x70d95... --collateral-token 0xaf88... --size-usd 2.0 --collateral-amount 1000000 --long --from 0xee385...` | `0xac9650d8...` outer; inner: `7d39aaf1`, `97aedce2` (createOrder MarketDecrease) |
| L3-6 | User wants to cancel a pending order (dry-run) | `gmx-v2 --chain arbitrum --dry-run cancel-order --key 0x1234...abcd --from 0xee385...` | Direct call: `0x7489ec23...` (cancelOrder) |
| L3-7 | User wants to claim funding fees (dry-run) | `gmx-v2 --chain arbitrum --dry-run claim-funding-fees --markets 0x70d95... --tokens 0xaf88... --receiver 0xee385... --from 0xee385...` | Direct call: `0xc41b1ab3...` (claimFundingFees) |

---

## L4 — On-Chain Write Tests

> **SKIPPED — Execution fee exceeds GUARDRAILS limit**
>
> GMX V2 Arbitrum requires a minimum execution fee of **0.001 ETH** (1,000,000,000,000,000 wei) per transaction.  
> GUARDRAILS limit: **0.00005 ETH** (50,000,000,000,000 wei).  
> 0.001 ETH is 20x the allowed limit. L4 on-chain tests are not run.
>
> Note: GMX V2 also operates on Avalanche where the fee is 0.012 AVAX — also exceeds the limit.

---

## GMX V2 Specific Notes

- **Execution fee**: Keeper model requires ETH prepayment. Arbitrum fee = 0.001 ETH, Avalanche fee = 0.012 AVAX. Both exceed test guardrails.
- **--long flag**: The `--long` arg is a clap boolean flag (presence = long, absence = short). SKILL.md examples show `--long true` syntax which does not work with clap. Use `--long` for long positions, omit for short.
- **Price format**: GMX oracle prices are in `price_usd * 10^(30 - token_decimals)` format. Correct division requires fetching token decimals from `/tokens` endpoint.
- **Multicall wrapping**: All order/deposit/withdrawal create-ops are wrapped in `multicall(bytes[])` (selector `0xac9650d8`). Inner call selectors: sendWnt (`7d39aaf1`), sendTokens (`e6d66ac8`), createOrder (`97aedce2`), createDeposit (`adc567e6`), createWithdrawal (`9b8eb9e7`).
