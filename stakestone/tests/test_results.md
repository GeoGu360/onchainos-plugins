# Test Results — StakeStone

- Date: 2026-04-05
- DApp supported chains: EVM only (Ethereum mainnet, chain 1)
- EVM test chain: Ethereum mainnet (1)
- Compile: PASS
- Lint: PASS (E123 placeholder SHA — expected before monorepo push)

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|-----------|---------|-------------|-------------|--------|---------|
| 8     | 1         | 2       | 3           | 2           | 0      | 0       |

## Detailed Results

| # | Scenario (user view) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Build plugin binary | L1 | `cargo build --release` | PASS | - | ~55s cached |
| 2 | Lint check | L1 | `cargo clean && plugin-store lint .` | PASS | - | Only E123 placeholder SHA (expected) |
| 3 | Query current STONE exchange rate | L2 | `stakestone get-rate` | PASS | - | STONE=1.063076 ETH, TVL=10051 ETH |
| 4 | Check position for zero-balance wallet | L2 | `stakestone get-position --address 0x87fb...` | PASS | - | 0 STONE, no pending withdrawal |
| 5 | Simulate staking 0.00005 ETH | L3 | `stakestone stake --amount 0.00005 --from <addr> --dry-run` | PASS | calldata: `0xd0e30db0` | selector correct |
| 6 | Simulate requesting withdrawal of 0.001 STONE | L3 | `stakestone request-withdraw --amount 0.001 --from <addr> --dry-run` | PASS | calldata: `0x745400c9` + 32-byte shares | selector correct |
| 7 | Simulate cancelling withdrawal of 0.001 STONE | L3 | `stakestone cancel-withdraw --amount 0.001 --from <addr> --dry-run` | PASS | calldata: `0x9f01f7ba` + 32-byte shares | selector correct |
| 8 | Stake 0.00005 ETH on Ethereum mainnet | L4 | `stakestone stake --amount 0.00005 --from 0x87fb...` | PASS | `0x07feb6f35d295d81760c1452343c7836684c0ade30162720239599985a54b1b6` | etherscan.io confirmed; received 0.000047 STONE |
| 9 | Verify position shows STONE after stake | L2 | `stakestone get-position --address 0x87fb...` | PASS | - | 0.000047 STONE, 0.000050 ETH value |

## L4 On-chain Evidence

**Stake 0.00005 ETH → STONE:**
- TxHash: `0x07feb6f35d295d81760c1452343c7836684c0ade30162720239599985a54b1b6`
- Chain: Ethereum mainnet (1)
- Explorer: https://etherscan.io/tx/0x07feb6f35d295d81760c1452343c7836684c0ade30162720239599985a54b1b6
- STONE received: 0.000047 STONE
- ETH spent: 0.00005 ETH (50000000000000 wei)

## Fix Record

None — zero bugs encountered.
