# Test Cases — swell-restaking

## Level 1: Compile + Lint

| # | Test | Command | Expected |
|---|------|---------|---------|
| L1-1 | Compile | `cargo build --release` | Finished successfully |
| L1-2 | Lint | `cargo clean && plugin-store lint .` | 0 errors (E123 OK until source_commit set) |

## Level 2: Read Tests (no wallet, no gas)

| # | Scenario | Command | Expected |
|---|----------|---------|---------|
| L2-1 | Query rswETH exchange rates | `./target/release/swell-restaking get-rates --chain 1` | JSON with ETH_per_rswETH > 1.0, rswETH_per_ETH < 1.0 |
| L2-2 | Query positions for test wallet | `./target/release/swell-restaking get-positions --address 0x87fb0647faabea33113eaf1d80d67acb1c491b90 --chain 1` | JSON with rswETH balance (likely 0), no error |
| L2-3 | Query positions for rswETH whale | `./target/release/swell-restaking get-positions --address 0xBA12222222228d8Ba445958a75a0704d566BF2C8 --chain 1` | JSON with non-zero rswETH balance |

## Level 3: Dry-run Tests (calldata validation)

| # | Scenario | Command | Expected |
|---|----------|---------|---------|
| L3-1 | Stake 0.00005 ETH dry-run | `./target/release/swell-restaking stake --amount 0.00005 --chain 1 --dry-run` | calldata starts with 0xd0e30db0, dry_run: true |
| L3-2 | Stake 0.001 ETH dry-run | `./target/release/swell-restaking stake --amount 0.001 --chain 1 --dry-run` | calldata: 0xd0e30db0, amount_wei: 1000000000000000 |

## Level 4: On-chain Tests (real tx, requires lock)

| # | Scenario | Command | Expected |
|---|----------|---------|---------|
| L4-1 | Stake 0.00005 ETH for rswETH | `./target/release/swell-restaking stake --amount 0.00005 --chain 1` | txHash from Etherscan |

## Level 1-error: Error Handling

| # | Scenario | Command | Expected |
|---|----------|---------|---------|
| E1 | Zero amount | `./target/release/swell-restaking stake --amount 0 --chain 1 --dry-run` | Error: "must be greater than 0" |
| E2 | Unsupported chain warning | `./target/release/swell-restaking get-rates --chain 8453` | Warning printed, but executes |
