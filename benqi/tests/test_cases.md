# Test Cases — Benqi Lending

## Level 1: Compilation + Lint

| # | Test | Command | Expected |
|---|------|---------|---------|
| 1 | Compile release | `cargo build --release` | Exit 0 |
| 2 | Lint | `cargo clean && plugin-store lint .` | 0 errors (E123 on placeholder SHA is expected pre-submission) |

## Level 2: Read Tests (no wallet, no gas)

| # | Scenario | Command | Expected |
|---|---------|---------|---------|
| 1 | List all Benqi markets | `benqi --chain 43114 markets` | JSON with 8 markets, supply/borrow APR > 0 |
| 2 | View positions (no position) | `benqi --chain 43114 positions --wallet 0x87fb0647faabea33113eaf1d80d67acb1c491b90` | Empty positions array, account_liquidity_usd = "0.0000" |
| 3 | Wrong chain error | `benqi --chain 1 markets` | ok:false, error contains "chain 43114" |

## Level 3: Dry-Run Tests (simulate calldata, no broadcast)

| # | Scenario | Command | Expected Selector |
|---|---------|---------|-----------------|
| 1 | Dry-run supply USDC | `benqi --chain 43114 --dry-run supply --asset USDC --amount 0.01` | approve: `0x095ea7b3`, mint: `0xa0712d68` |
| 2 | Dry-run supply AVAX | `benqi --chain 43114 --dry-run supply --asset AVAX --amount 0.001` | `0x1249c58b` |
| 3 | Dry-run redeem USDC | `benqi --chain 43114 --dry-run redeem --asset USDC --amount 0.01` | `0x852a12e3` |
| 4 | Borrow USDC (always dry-run) | `benqi --chain 43114 borrow --asset USDC --amount 1.0` | `0xc5ebeaec`, dry_run:true |
| 5 | Repay USDC (always dry-run) | `benqi --chain 43114 repay --asset USDC --amount 1.0` | approve: `0x095ea7b3`, repay: `0x0e752702` |
| 6 | Dry-run claim QI rewards | `benqi --chain 43114 --dry-run claim-rewards --reward-type 0` | `0x0952c563` |
| 7 | Dry-run claim AVAX rewards | `benqi --chain 43114 --dry-run claim-rewards --reward-type 1` | `0x0952c563` |

## Level 4: On-Chain Tests

**SKIPPED** — Test wallet (0x87fb0647faabea33113eaf1d80d67acb1c491b90) has 0 AVAX balance on Avalanche C-Chain (chain 43114) at time of testing. All L2 and L3 tests pass.

To run L4 tests when funded:
1. Supply small AVAX: `benqi --chain 43114 supply --asset AVAX --amount 0.001`
2. Claim rewards: `benqi --chain 43114 claim-rewards --reward-type 0`
