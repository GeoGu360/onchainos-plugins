# Test Results — swell-restaking

- Date: 2026-04-05
- DApp supported chains: EVM only (Ethereum mainnet, chain 1)
- EVM test chain: Ethereum mainnet (1)
- Compile: ✅
- Lint: ✅ (E123 only — source_commit placeholder, fixed after monorepo push)

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|-----------|---------|-------------|-------------|--------|---------|
| 8     | 2         | 4       | 2           | 1           | 0      | 0       |

## Detailed Results

| # | Scenario (user view) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Build plugin binary | L1 | `cargo build --release` | ✅ PASS | — | Clean compile, 0 errors |
| 2 | Lint plugin | L1 | `cargo clean && plugin-store lint .` | ✅ PASS | — | 0 errors (E123 expected pre-submit) |
| 3 | Check rswETH exchange rates | L2 | `get-rates --chain 1` (via --chain before subcmd) | ✅ PASS | — | ETH/rswETH=1.069, rswETH/ETH=0.935 |
| 4 | Check my rswETH balance (empty wallet) | L2 | `get-positions --address 0x87fb...` | ✅ PASS | — | Returns 0.000046 rswETH with ETH value |
| 5 | Check rswETH balance for Balancer vault | L2 | `get-positions --address 0xBA12...` | ✅ PASS | — | Non-zero balance returned |
| 6 | Error: zero stake amount | L2 | `stake --amount 0 --dry-run` | ✅ PASS | — | Error: "must be greater than 0" |
| 7 | Stake 0.00005 ETH dry-run | L3 | `stake --amount 0.00005 --dry-run` | ✅ PASS | calldata: 0xd0e30db0 | Selector correct (deposit()) |
| 8 | Stake 0.00005 ETH for rswETH | L4 | `stake --amount 0.00005 --from 0x87fb...` | ✅ PASS | 0x8b65253fe7e0d303eb29366f376366d5dcd40bfaf368620c53757dff98928188 | [Etherscan](https://etherscan.io/tx/0x8b65253fe7e0d303eb29366f376366d5dcd40bfaf368620c53757dff98928188) |

## Fix Log

No bugs encountered. Clean pass on first run.

## Notes

- CLI structure: `swell-restaking --chain <id> <subcommand>` (global flag before subcommand)
- Test wallet already had 0.000046 rswETH from previous swell-staking restake test
- L4 stake confirmed on Ethereum mainnet etherscan
- The `--output json` flag is not supported on chain 1 (known limitation), so wallet balance uses plain JSON parsing
