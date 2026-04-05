# Test Results Report

- Date: 2026-04-05
- DApp supported chains: EVM only
- EVM test chain: Base (8453)
- Compilation: OK
- Lint: OK (1 expected error E123 placeholder SHA — resolved after monorepo push)
- Overall pass standard: EVM DApp - EVM all pass

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|-----------|---------|-------------|-------------|--------|---------|
| 9     | 1         | 5       | 2           | 1           | 0      | 0       |

## Detailed Results

| # | Scenario (User View) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Compile and lint plugin | L1 | `cargo build --release && cargo clean && plugin-store lint` | PASS | - | 0 errors (E123 expected, resolved after SHA fill) |
| 2 | List all supported EVM chains | L2 | `get-chains` | PASS | - | 62 mainnet chains returned |
| 3 | Find USDC on Base | L2 | `get-tokens --chains 8453 --symbol USDC` | PASS | - | address 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913 |
| 4 | Get available bridges and DEXes | L2 | `get-tools` | PASS | - | 31 bridges, 32 exchanges |
| 5 | Quote bridge 5 USDC from Base to Arbitrum | L2 | `get-quote --from-chain 8453 --to-chain 42161 --from-token USDC --to-token USDC --amount 5000000` | PASS | - | Tool: eco, receive 4.9875 USDC |
| 6 | Check completed transfer status | L2 | `get-status --tx-hash 0xd3ae... --from-chain 8453` | PASS | - | Status: DONE, substatus: COMPLETED |
| 7 | Preview bridge without broadcasting | L3 | `--dry-run swap --to-chain 42161 --from-token USDC --to-token USDC --amount 5000000` | PASS | calldata_selector: 0x9e75aa95 | Correct LiFiDiamond target |
| 8 | Preview same-chain swap | L3 | `--dry-run swap --to-chain 8453 --from-token USDC --to-token ETH --amount 10000` | PASS | calldata_selector: 0x2c57e884 | ERC-20 approve preview included |
| 9 | Swap 0.01 USDC to ETH on Base | L4 | `--chain 8453 swap --to-chain 8453 --from-token USDC --to-token ETH --amount 10000` | PASS | 0xd3ae5aed5d5eb0405fc28f0ef05f830aa0a976d5fbd1878f530563ea79a4e4be | BaseScan: confirmed, tool: Nordstern Finance |

## Fix Log

None - all tests passed on first run.

## L4 Transaction Details

- Approve tx: 0x0eceddaeee4b7a6074fd4247abe94e25381d9429e4726ad0cb0f740a5d2fc025 (Base, USDC approve to LiFiDiamond)
- Swap tx: 0xd3ae5aed5d5eb0405fc28f0ef05f830aa0a976d5fbd1878f530563ea79a4e4be (Base, 0.01 USDC to ETH via Nordstern Finance)
- BaseScan: https://basescan.org/tx/0xd3ae5aed5d5eb0405fc28f0ef05f830aa0a976d5fbd1878f530563ea79a4e4be
- LI.FI Explorer: https://scan.li.fi/tx/0xd3ae5aed5d5eb0405fc28f0ef05f830aa0a976d5fbd1878f530563ea79a4e4be
