# GMX V1 Test Cases

## Level 1 — Compile + Lint

| # | Test | Command | Expected |
|---|------|---------|---------|
| 1 | Build release binary | `cargo build --release` | Compiles with 0 errors |
| 2 | Lint | `cargo clean && plugin-store lint .` | 0 errors, 0 warnings |

## Level 2 — Read Tests (no wallet, no gas)

| # | Scenario | Command | Expected |
|---|---------|---------|---------|
| 1 | Get token prices on Arbitrum | `get-prices --chain 42161` | Table of token prices |
| 2 | Get token prices on Avalanche | `get-prices --chain 43114` | Table of token prices |
| 3 | Get positions (no open positions) | `get-positions --chain 42161 --account 0x87fb0647faabea33113eaf1d80d67acb1c491b90` | "No open positions" or positions list |

## Level 3 — Dry-run Tests (verify calldata selectors)

| # | Scenario | Command | Expected Selector |
|---|---------|---------|-----------------|
| 1 | Swap USDC to WETH dry-run | `swap --input-token USDC --input-amount 10000000 --output-token WETH --min-output 0 --dry-run` | `0x6023e966` |
| 2 | Buy GLP with USDC dry-run | `buy-glp --token USDC --amount 5000000 --min-usdg 0 --min-glp 0 --dry-run` | `0x364e2311` |
| 3 | Sell GLP dry-run | `sell-glp --token-out USDC --glp-amount 1e18 --min-out 0 --dry-run` | `0x0f3aa554` |
| 4 | Open long position dry-run | `open-position --collateral-token USDC --index-token WETH --amount-in 5000000 --size-usd 50.0 --is-long --acceptable-price 2100e33 --dry-run` | `0xf2ae372f` |
| 5 | Close position dry-run | `close-position --collateral-token USDC --index-token WETH --size-usd 50.0 --is-long --acceptable-price 1900e33 --dry-run` | `0x7be7d141` |
| 6 | Approve token dry-run | `approve-token --token USDC --spender Router --dry-run` | `0x095ea7b3` |

## Level 4 — On-chain Tests (Arbitrum 42161)

| # | Scenario | Command | Policy |
|---|---------|---------|--------|
| 1 | Approve USDT to Router | `approve-token --token USDT --spender Router` | Execute |
| 2 | Swap 0.01 USDT to WETH | `swap --input-token USDT --input-amount 10000 --output-token WETH` | Execute |
| 3 | Buy GLP with USDC.e | `buy-glp --token USDC.e --amount 9994` | Execute |
| 4 | Open position | `open-position ...` | SKIPPED — 0.0001 ETH fee > GUARDRAILS 0.00005 ETH limit |
| 5 | Close position | `close-position ...` | SKIPPED — 0.0001 ETH fee > GUARDRAILS 0.00005 ETH limit |
