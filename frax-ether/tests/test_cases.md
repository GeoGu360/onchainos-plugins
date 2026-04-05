# Test Cases — frax-ether

## L1 — Build + Lint

| # | Description | Command | Expected |
|---|-------------|---------|---------|
| 1.1 | Compile (debug) | `cargo build` | ✅ 0 errors |
| 1.2 | Compile (release) | `cargo build --release` | ✅ 0 errors |
| 1.3 | Lint | `cargo clean && plugin-store lint .` | ✅ 0 errors (plugin-store not installed locally) |

## L2 — Read Tests (no wallet, no gas)

| # | Description | Command | Expected |
|---|-------------|---------|---------|
| 2.1 | Query sfrxETH APR and rates | `frax-ether rates` | JSON with apr, exchange rate, total_assets |
| 2.2 | Query positions for wallet | `frax-ether positions --address 0x87fb...` | JSON with frxETH + sfrxETH balances |

## L3 — Dry-run (simulate, no broadcast)

| # | Description | Command | Expected |
|---|-------------|---------|---------|
| 3.1 | Stake ETH dry-run | `frax-ether stake --amount 0.00005 --dry-run` | calldata `0x5bcb2fc6`, dry_run:true |
| 3.2 | Stake frxETH dry-run | `frax-ether stake-frx --amount 0.00005 --dry-run` | approve `0x095ea7b3...`, deposit `0x6e553f65...` |
| 3.3 | Unstake sfrxETH dry-run | `frax-ether unstake --amount 0.00005 --dry-run` | redeem calldata `0xba087652...` |

## L4 — On-chain Write Operations

| # | Description | Command | Expected |
|---|-------------|---------|---------|
| 4.1 | Stake 0.00005 ETH → frxETH | `frax-ether stake --amount 0.00005 --chain 1` | txHash on etherscan.io |
| 4.2 | Stake 0.00005 frxETH → sfrxETH | `frax-ether stake-frx --amount 0.00005 --chain 1` | approve + deposit txHash |
