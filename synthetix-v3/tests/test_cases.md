# Synthetix V3 Test Cases

DApp: Synthetix V3 | Chain: Base (8453) | Date: 2026-04-05

## L1 — Compile + Lint

| # | Test | Command | Expected |
|---|------|---------|----------|
| 1 | Debug build | `cargo build` | 0 errors |
| 2 | Release build | `cargo build --release` | 0 errors |
| 3 | Lint | `cargo clean && plugin-store lint .` | 0 errors (E123 expected pending monorepo push) |

## L2 — Read Tests (no gas, no wallet)

| # | Scenario | Command | Expected |
|---|----------|---------|----------|
| 4 | List all perps markets (first 20) | `./target/release/synthetix-v3 markets` | JSON with markets array, ETH/BTC present |
| 5 | Query ETH market specifically | `./target/release/synthetix-v3 markets --market-id 100` | JSON with ETH symbol, funding rate, skew |
| 6 | Query BTC market | `./target/release/synthetix-v3 markets --market-id 200` | JSON with BTC symbol |
| 7 | Query positions for known account | `./target/release/synthetix-v3 positions --account-id 1` | JSON with available_margin |
| 8 | Query collateral for known account | `./target/release/synthetix-v3 collateral --account-id 1` | JSON with collaterals array |

## L3 — Dry-run Tests (validate calldata structure)

| # | Scenario | Command | Expected |
|---|----------|---------|----------|
| 9 | Dry-run deposit 0.01 sUSDC | `./target/release/synthetix-v3 --dry-run deposit-collateral --account-id 1 --amount 0.01` | calldata starts with 0x83802968 |
| 10 | Dry-run withdraw 0.01 sUSDC | `./target/release/synthetix-v3 --dry-run withdraw-collateral --account-id 1 --amount 0.01` | calldata starts with 0x95997c51 |

## L4 — On-chain Tests (requires lock, real gas)

Note: Base ETH balance is LOW (0.0028 ETH). ETH reserve threshold = 0.001 ETH.
Available ETH for tests: ~0.0018 ETH. USDC: 0.28 USDC.

For deposit-collateral L4, we need sUSDC. Since we only have USDC (not sUSDC), L4 deposit/withdraw will be **skipped** unless sUSDC is available.

| # | Scenario | Command | Expected |
|---|----------|---------|----------|
| 11 | Check wallet balance | `onchainos wallet balance --chain 8453 --output json` | ETH balance + USDC balance |
| 12 | deposit-collateral L4 | SKIP (no sUSDC in wallet) | — |
| 13 | withdraw-collateral L4 | SKIP (no prior deposit) | — |
