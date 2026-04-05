# Symbiotic Plugin — Test Cases

DApp supported chains: Ethereum (EVM only, chain ID: 1)
Test chain: Ethereum mainnet (chain 1)

## Level 1 — Compilation + Lint

| # | Test | Command | Expected |
|---|------|---------|---------|
| 1 | cargo build | `cargo build --release` | 0 errors |
| 2 | plugin-store lint | `cargo clean && plugin-store lint .` | 0 errors |

## Level 2 — Read Operations (no wallet, no gas)

| # | Scenario (user view) | Command | Expected |
|---|---------------------|---------|---------|
| 3 | List all Symbiotic vaults | `vaults --chain 1` | JSON with vaults array, TVL/APR fields |
| 4 | List wstETH vault specifically | `vaults --token wstETH --chain 1` | 1+ vaults with token_symbol=wstETH |
| 5 | Get APR rates for all vaults | `rates --chain 1` | JSON with rates array, apr fields |
| 6 | Get wstETH vault APR | `rates --token wstETH --chain 1` | wstETH vault rate |

## Level 3 — Simulation / dry-run (no gas)

| # | Scenario (user view) | Command | Expected |
|---|---------------------|---------|---------|
| 7 | Preview deposit 0.00005 wstETH into wstETH vault | `deposit --token wstETH --amount 0.00005 --chain 1 --dry-run` | dry_run=true, approve calldata starts 0x095ea7b3, deposit calldata starts 0x47e7ef24 |
| 8 | Preview withdraw 0.00005 wstETH | `withdraw --token wstETH --amount 0.00005 --chain 1 --dry-run` | dry_run=true, calldata starts 0xf3fef3a3 |
| 9 | Query positions (wallet resolution) | `positions --address 0x87fb0647faabea33113eaf1d80d67acb1c491b90 --chain 1` | JSON with positions array (may be empty) |

## Level 4 — On-chain Write Operations (requires lock, uses gas)

| # | Scenario (user view) | Command | Fund limit |
|---|---------------------|---------|-----------|
| 10 | Deposit 0.00005 wstETH into wstETH Vault | `deposit --token wstETH --amount 0.00005 --chain 1` | 0.00005 ETH gas |
| 11 | Request withdraw 0.00005 wstETH | `withdraw --token wstETH --amount 0.00005 --chain 1` | 0.00005 ETH gas |

Note: The wstETH in wallet from Mellow LRT testing will be used for L4 deposit test.
