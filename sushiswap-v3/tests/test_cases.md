# SushiSwap V3 — Test Cases

## Test Pyramid

| Level | Type | Count |
|-------|------|-------|
| L1 | Build | 1 |
| L2 | Read (no wallet) | 4 |
| L3 | Dry-run write (selector verify) | 4 |
| L4 | On-chain | 2 |

## L1 — Build

| ID | Name | Command | Expected |
|----|------|---------|----------|
| L1-1 | Release build | `cargo build --release` | Exit 0, binary created |

## L2 — Read Operations

| ID | Name | Command | Expected |
|----|------|---------|----------|
| L2-1 | List WETH/USDC pools on Base | `get-pools --token0 WETH --token1 USDC --chain 8453` | JSON with 4 deployed pools |
| L2-2 | Quote WETH→USDC all fee tiers | `quote --token-in WETH --token-out USDC --amount-in 1000000000000000 --chain 8453` | JSON ok=true, bestFee, amountOut > 0 |
| L2-3 | Quote WETH→USDC fee=500 | `quote --token-in WETH --token-out USDC --amount-in 1000000000000000 --fee 500 --chain 8453` | amountOut > 0 |
| L2-4 | Get positions (empty wallet) | `get-positions --owner 0x87fb0647faabea33113eaf1d80d67acb1c491b90 --chain 8453` | ok=true, positions=[] |

## L3 — Dry-Run Write Operations

| ID | Name | Command | Expected |
|----|------|---------|----------|
| L3-1 | Swap dry-run WETH→USDC | `swap --token-in WETH --token-out USDC --amount-in 50000000000000 --chain 8453 --dry-run` | Calldata selector 0x414bf389, txHash all zeros |
| L3-2 | Add-liquidity dry-run (negative ticks) | `add-liquidity --token0 WETH --token1 USDC --fee 500 --tick-lower -887270 --tick-upper 887270 --amount0-desired 1000000000000000 --amount1-desired 2051116 --chain 8453 --dry-run` | Calldata selector 0x88316456, tick-lower accepted |
| L3-3 | Collect-fees dry-run | `collect-fees --token-id 99999 --chain 8453 --dry-run` | ok=true, "no fees owed" |
| L3-4 | Remove-liquidity dry-run | `remove-liquidity --token-id 99999 --chain 8453 --dry-run` | ok=true, both txHashes all zeros |

## L4 — On-Chain

| ID | Name | Command | Expected |
|----|------|---------|----------|
| L4-1 | Swap native ETH→USDC via Sushi API | Direct RouteProcessor4 call with Sushi API calldata, 0.00005 ETH | txHash returned, tx confirmed |
| L4-2 | Plugin swap WETH→USDC via SwapRouter | `swap --token-in WETH --token-out USDC --amount-in 50000000000000 --chain 8453` | Auto-approve WETH, swap executed, txHash returned |
