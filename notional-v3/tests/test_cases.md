# Test Cases — notional-v3

Protocol: Notional Exponent (Ethereum mainnet, chain 1 only)
Test wallet: 0x87fb0647faabea33113eaf1d80d67acb1c491b90

## L1 — Compile + Lint

| # | Test | Command | Pass Criteria |
|---|------|---------|--------------|
| 1 | Compile release binary | `cargo build --release` | Exit 0, no errors |
| 2 | Lint plugin | `cargo clean && plugin-store lint .` | 0 errors |

## L2 — Read Tests (no wallet, no gas)

| # | User Intent | Command | Pass Criteria |
|---|------------|---------|--------------|
| 3 | List all Notional Exponent vaults | `get-vaults` | ok=true, count>0, vaults array non-empty |
| 4 | List USDC vaults only | `get-vaults --asset USDC` | ok=true, all vaults have asset_symbol=USDC |
| 5 | List WETH vaults only | `get-vaults --asset WETH` | ok=true, all vaults have asset_symbol=WETH |
| 6 | Get positions for test wallet | `get-positions --wallet 0x87fb...` | ok=true, JSON parseable |

## L3 — Dry-run / Simulate Tests

| # | User Intent | Command | Pass Criteria |
|---|------------|---------|--------------|
| 7 | Simulate entering USDC vault | `--dry-run enter-position --vault 0x9fb5... --amount 0.01 --asset USDC` | calldata starts with 0xde13c617 |
| 8 | Simulate exiting vault | `--dry-run exit-position --vault 0x9fb5... --shares 1000000` | calldata starts with 0x8a363181 |
| 9 | Simulate initiate-withdraw (staking vault) | `--dry-run initiate-withdraw --vault 0xaf14... --shares 1000000` | calldata starts with 0x37753799 |
| 10 | Simulate claim-rewards | `--dry-run claim-rewards --vault 0x9fb5...` | calldata starts with 0xf1e42ccd |
| 11 | Error: wrong chain ID | `--chain 8453 get-vaults` | non-zero exit, "not supported" message |

## L4 — On-chain Write Tests (require lock, real gas)

| # | User Intent | Command | Pass Criteria |
|---|------------|---------|--------------|
| 12 | Enter liUSD-4w vault with 0.01 USDC | `enter-position --vault 0x9fb57943926749b49a644f237a28b491c9b465e0 --amount 0.01 --asset USDC` | ok=true, tx_hash non-null, etherscan URL |
| 13 | Exit liUSD-4w vault (all shares) | `exit-position --vault 0x9fb57943926749b49a644f237a28b491c9b465e0 --shares all` | ok=true, tx_hash non-null |
