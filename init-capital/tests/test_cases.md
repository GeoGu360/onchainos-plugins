# INIT Capital Test Cases

- Date: 2026-04-05
- Chain: Blast (81457)
- Note: Mantle (5000) not supported by onchainos; using Blast deployment
- Note: L4 SKIPPED — wallet has no funds on Blast (81457)

## L1 — Compile + Lint

| # | Test | Expected |
|---|------|----------|
| 1.1 | `cargo build --release` | Compiles successfully |
| 1.2 | `cargo clean && plugin-store lint .` | Only E123 (placeholder SHA), no other errors |

## L2 — Read Tests (no wallet, no gas)

| # | Test | Command | Expected |
|---|------|---------|----------|
| 2.1 | Query WETH pool supply rate | `./target/release/init-capital pools --chain 81457` | Returns JSON with 2 pools, supply_rate_e18 non-zero |
| 2.2 | Query positions (empty wallet) | `./target/release/init-capital positions --chain 81457 --wallet 0x0000000000000000000000000000000000000001` | Returns ok:true, position_count:0 |
| 2.3 | Health factor for pos 1 | `./target/release/init-capital health-factor --pos-id 1 --chain 81457` | Returns health_e18 value |
| 2.4 | Wrong chain ID | `./target/release/init-capital pools --chain 1` | Returns error about wrong chain |

## L3 — Dry-run Tests (calldata verification)

| # | Test | Command | Expected Selector |
|---|------|---------|------------------|
| 3.1 | Supply WETH dry-run | `./target/release/init-capital supply --asset WETH --amount 0.01 --chain 81457 --dry-run` | step 2 calldata starts with 0x247d4981 |
| 3.2 | Withdraw WETH dry-run | `./target/release/init-capital withdraw --asset WETH --amount 0.01 --pos-id 1 --chain 81457 --dry-run` | calldata starts with 0x247d4981 |
| 3.3 | Borrow USDB dry-run | `./target/release/init-capital borrow --asset USDB --amount 1.0 --pos-id 1 --chain 81457 --dry-run` | calldata starts with 0x247d4981 |
| 3.4 | Repay USDB dry-run | `./target/release/init-capital repay --asset USDB --amount 1.0 --pos-id 1 --chain 81457 --dry-run` | step 2 calldata starts with 0x247d4981 |
| 3.5 | Supply approve dry-run | `./target/release/init-capital supply --asset WETH --amount 0.01 --chain 81457 --dry-run` | step 1 calldata starts with 0x095ea7b3 |

## L4 — On-chain Tests

**SKIPPED** — Wallet `0x87fb0647faabea33113eaf1d80d67acb1c491b90` has $0.00 balance on Blast (81457).

Reason: INIT Capital's supported chains are Mantle (5000, not supported by onchainos) and Blast (81457, supported but no funds).

All L2 and L3 tests will be executed to validate plugin functionality.
