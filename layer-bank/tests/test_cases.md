# LayerBank Test Cases

## Level 1 — Compile + Lint

| # | Test | Expected |
|---|------|---------|
| 1.1 | `cargo build` | Compiles with 0 errors |
| 1.2 | `cargo clean && plugin-store lint .` | 0 errors, 0 warnings |

## Level 2 — Read Tests (no wallet, no gas)

| # | User Scenario | Command | Expected |
|---|--------------|---------|---------|
| 2.1 | View LayerBank markets on Scroll | `layer-bank markets` | JSON with ETH, USDC, USDT, wstETH, WBTC markets |
| 2.2 | View positions for empty wallet | `layer-bank positions --wallet 0x87fb...` | JSON with empty supplied/borrowed, ∞ health |
| 2.3 | Verify ETH market TVL > 0 | `layer-bank markets` | tvl_usd > 0 for ETH market |

## Level 3 — Dry-Run Simulation (verify calldata)

| # | User Scenario | Command | Expected selector |
|---|--------------|---------|------------------|
| 3.1 | Supply 0.001 ETH dry-run | `supply --asset ETH --amount 0.001 --dry-run` | `0xf2b9fdb8` |
| 3.2 | Supply 0.01 USDC dry-run | `supply --asset USDC --amount 0.01 --dry-run` | approve:`0x095ea7b3` + supply:`0xf2b9fdb8` |
| 3.3 | Withdraw 0.01 USDC dry-run | `withdraw --asset USDC --amount 0.01 --dry-run` | `0x96294178` |
| 3.4 | Borrow 0.01 USDC dry-run | `borrow --asset USDC --amount 0.01 --dry-run` | `0x4b8a3529` |
| 3.5 | Repay 0.01 USDC dry-run | `repay --asset USDC --amount 0.01 --dry-run` | approve:`0x095ea7b3` + repay:`0xabdb5ea8` |
| 3.6 | Wrong chain rejected | `markets --chain 8453` → but chain is global flag | Error: "LayerBank is deployed on Scroll" |

## Level 4 — On-Chain Tests

**Status: SKIPPED — 0 ETH/USDC on Scroll (chain 534352)**

LayerBank is NOT deployed on Base (chain 8453). The test wallet has 0 funds on Scroll.
Funds on Base (ETH=0.0028, USDC=0.27) cannot be used since LayerBank has no Base deployment.

To run L4 tests: fund the wallet with ≥ 0.001 ETH on Scroll (chain 534352).

Expected L4 tests once funded:
| # | Scenario | Command | Expected |
|---|---------|---------|---------|
| 4.1 | Supply 0.0001 ETH | `supply --asset ETH --amount 0.0001` | txHash on scrollscan |
| 4.2 | View position | `positions` | lETH balance > 0 |
| 4.3 | Withdraw 0.0001 ETH | `withdraw --asset ETH --amount 0.0001` | txHash, position cleared |
