# Velodrome V2 Test Cases

## DApp Supported Chains
- Optimism (chain ID 10) — EVM only

## Test Pyramid

### L1 — Compile + Lint
| # | Test | Command | Expected |
|---|------|---------|---------|
| 1 | Compile | `cargo build --release` | Compiled successfully |
| 2 | Lint | `cargo clean && plugin-store lint .` | 0 errors |

### L2 — Read Tests (no wallet, no gas)
| # | Scenario | Command | Expected |
|---|----------|---------|---------|
| 3 | Query WETH/USDC volatile pool | `velodrome-v2 pools --token-a WETH --token-b USDC --stable false` | Pool address returned, reserves > 0 |
| 4 | Query WETH/USDC stable pool | `velodrome-v2 pools --token-a WETH --token-b USDC --stable true` | deployed: false or deployed: true |
| 5 | Get swap quote WETH -> USDC | `velodrome-v2 quote --token-in WETH --token-out USDC --amount-in 50000000000000` | amountOut > 0 |
| 6 | Get quote USDC -> DAI stable | `velodrome-v2 quote --token-in USDC --token-out DAI --amount-in 1000000 --stable true` | amountOut > 0 or pool not deployed |
| 7 | Query pool by address | `velodrome-v2 pools --pool 0xf4f2657ae744354baca871e56775e5083f7276ab` | token0, token1, reserves returned |

### L3 — Dry-run Simulate (verify calldata)
| # | Scenario | Command | Expected |
|---|----------|---------|---------|
| 8 | Dry-run swap WETH -> USDC | `velodrome-v2 swap --token-in WETH --token-out USDC --amount-in 50000000000000 --dry-run` | calldata starts with 0xcac88ea9 |
| 9 | Dry-run add-liquidity | `velodrome-v2 add-liquidity --token-a WETH --token-b USDC --stable false --amount-a-desired 50000000000000 --dry-run` | calldata starts with 0x5a47ddc3 |
| 10 | Dry-run remove-liquidity | `velodrome-v2 remove-liquidity --token-a WETH --token-b USDC --stable false --dry-run` | calldata starts with 0x0dede6c4 |
| 11 | Dry-run claim-rewards | `velodrome-v2 claim-rewards --gauge 0x6c2614e94b085a4b1fdcb0e3d7f09c2b5c2e3d53 --dry-run` | dry_run: true |

### L4 — On-chain Write Tests (need lock, spend gas)
| # | Scenario | Command | Expected |
|---|----------|---------|---------|
| 12 | Swap USDC -> WETH (min amount) | `velodrome-v2 swap --token-in USDC --token-out WETH --amount-in 10000 --slippage 1.0` | txHash returned, verified on optimistic.etherscan.io |
| 13 | Positions check (read only, resolve wallet) | `velodrome-v2 positions` | ok: true, positions array |
