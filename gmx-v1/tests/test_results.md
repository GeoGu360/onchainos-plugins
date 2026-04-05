# Test Results Report

- Date: 2026-04-05
- DApp supported chains: EVM (Arbitrum 42161, Avalanche 43114)
- Test chain: Arbitrum (42161) — EVM only DApp
- Compile: PASS
- Lint: PASS

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|-----------|---------|-------------|-------------|--------|---------|
| 13    | 2          | 3       | 6           | 2           | 0      | 4       |

## Detailed Results

| # | Scenario (user view) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|------------------|-------|
| 1 | Build GMX V1 plugin | L1 | `cargo build --release` | PASS | - | 0 errors |
| 2 | Lint plugin | L1 | `cargo clean && plugin-store lint .` | PASS | - | 0 errors, 0 warnings |
| 3 | View GMX V1 token prices on Arbitrum | L2 | `get-prices --chain 42161` | PASS | - | 28+ tokens returned with correct price data |
| 4 | View GMX V1 token prices on Avalanche | L2 | `get-prices --chain 43114` | PASS | - | Token prices returned |
| 5 | Check wallet positions | L2 | `get-positions --chain 42161 --account 0x87fb...` | PASS | - | Returns "No open positions" correctly |
| 6 | Simulate swap USDC to WETH | L3 | `swap --dry-run ...` | PASS | `0x6023e966...` | Selector 0x6023e966 confirmed |
| 7 | Simulate buy GLP with USDC | L3 | `buy-glp --dry-run ...` | PASS | `0x364e2311...` | Selector 0x364e2311 confirmed |
| 8 | Simulate sell GLP | L3 | `sell-glp --dry-run ...` | PASS | `0x0f3aa554...` | Selector 0x0f3aa554 confirmed |
| 9 | Simulate open long position | L3 | `open-position --is-long --dry-run ...` | PASS | `0xf2ae372f...` | Selector 0xf2ae372f confirmed |
| 10 | Simulate close position | L3 | `close-position --dry-run ...` | PASS | `0x7be7d141...` | Selector 0x7be7d141 confirmed |
| 11 | Simulate approve token | L3 | `approve-token --dry-run ...` | PASS | `0x095ea7b3...` | Selector 0x095ea7b3 confirmed |
| 12 | Approve USDT to Router | L4 | `approve-token --token USDT --spender Router` | PASS | [0x3cf357907c41a1ae8cae94463e5d6862755a6f2811deb56c44ef499d5adfcb4a](https://arbiscan.io/tx/0x3cf357907c41a1ae8cae94463e5d6862755a6f2811deb56c44ef499d5adfcb4a) | |
| 13 | Swap 0.01 USDT to WETH | L4 | `swap --input-token USDT --input-amount 10000 --output-token WETH` | PASS | [0x183741f17ebe58f1724d045f7ea46f3797378595e487976d5c5eac2ae84305e6](https://arbiscan.io/tx/0x183741f17ebe58f1724d045f7ea46f3797378595e487976d5c5eac2ae84305e6) | 0.01 USDT -> WETH successful |

## Skipped Tests

| # | Scenario | Reason |
|---|---------|--------|
| 14 | Buy GLP on-chain | GMX V1 GLP pool at max USDG capacity — all tokens (USDC, USDC.e, USDT) have reached maxUsdgAmount. GMX V1 liquidity is deprecated since V2 launch. Dry-run L3 test verifies calldata is correct. |
| 15 | Sell GLP on-chain | Cannot acquire GLP tokens (buy-glp blocked); dry-run verified. |
| 16 | Open position on-chain | 0.0001 ETH (100,000,000,000,000 wei) execution fee exceeds GUARDRAILS L4 limit of 0.00005 ETH. Dry-run L3 test verifies calldata selector 0xf2ae372f. |
| 17 | Close position on-chain | 0.0001 ETH execution fee exceeds GUARDRAILS L4 limit. Dry-run L3 verified. |

## Fix Record

| # | Issue | Root Cause | Fix | File |
|---|-------|-----------|-----|------|
| 1 | buy-glp "GlpManager: forbidden" | RewardRouter V1 (0xA906...) not a handler of GlpManager | Switched to RewardRouter V2 (0xB95DB5...) and GlpManager V2 (0x3963...) | src/config.rs |
| 2 | "Vault: max USDG exceeded" | GMX V1 GLP pool at maximum capacity for stablecoins | Protocol limitation — noted as SKIPPED in test results | - |

## Selector Verification

| Function | Selector | Source | Status |
|---------|---------|--------|--------|
| `swap(address[],uint256,uint256,address)` | `0x6023e966` | cast sig | ✅ |
| `swapETHToTokens(address[],uint256,address)` | `0xabe68eaa` | cast sig | ✅ |
| `swapTokensToETH(address[],uint256,uint256,address payable)` | `0x2d4ba6a7` | cast sig | ✅ |
| `mintAndStakeGlp(address,uint256,uint256,uint256)` | `0x364e2311` | cast sig | ✅ |
| `unstakeAndRedeemGlp(address,uint256,uint256,address)` | `0x0f3aa554` | cast sig | ✅ |
| `createIncreasePosition(address[],address,uint256,uint256,uint256,bool,uint256,uint256,bytes32,address)` | `0xf2ae372f` | cast sig | ✅ |
| `createDecreasePosition(address[],address,uint256,uint256,bool,address,uint256,uint256,uint256,bool,address)` | `0x7be7d141` | cast sig | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | cast sig | ✅ |
