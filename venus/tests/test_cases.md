# Venus Core Pool — Test Cases

## Level 1: Compilation + Lint

| # | Test | Command | Expected |
|---|------|---------|---------|
| 1.1 | Compile debug | `cargo build` | Finished without errors |
| 1.2 | Compile release | `cargo build --release` | Binary at target/release/venus |
| 1.3 | Lint | `cargo clean && plugin-store lint .` | "Plugin 'venus' passed all checks!" |

## Level 2: Read Tests (No wallet, no gas)

| # | Test | Command | Expected |
|---|------|---------|---------|
| 2.1 | Get all markets | `venus --chain 56 get-markets` | 48 markets with APY data |
| 2.2 | Get positions (test wallet, empty) | `venus --chain 56 get-positions --wallet 0x87fb...` | Empty positions, 0 liquidity |
| 2.3 | Invalid chain | `venus --chain 1 get-markets` | Error: "Unsupported chain ID: 1" |

## Level 3: Dry-run Simulation (verify calldata)

| # | Test | Command | Expected Selector |
|---|------|---------|-----------------|
| 3.1 | Supply USDT dry-run | `venus --chain 56 --dry-run supply --asset USDT --amount 0.01` | `0xa0712d68` (mint(uint256)) |
| 3.2 | Supply BNB dry-run | `venus --chain 56 --dry-run supply --asset BNB --amount 0.001` | `0x1249c58b` (mint()) |
| 3.3 | Withdraw USDT dry-run | `venus --chain 56 --dry-run withdraw --asset USDT --amount 0.01` | `0x852a12e3` (redeemUnderlying(uint256)) |
| 3.4 | Borrow USDT dry-run | `venus --chain 56 --dry-run borrow --asset USDT --amount 0.01` | `0xc5ebeaec` (borrow(uint256)) |
| 3.5 | Repay USDT dry-run | `venus --chain 56 --dry-run repay --asset USDT --amount 0.01` | `0x0e752702` (repayBorrow(uint256)) |
| 3.6 | Enter market USDT | `venus --chain 56 --dry-run enter-market --asset USDT` | `0xc2998238` (enterMarkets(address[])) |
| 3.7 | Claim rewards | `venus --chain 56 --dry-run claim-rewards` | `0xadcd5fb9` (claimVenus(address)) |

## Level 4: On-chain Write Tests (BSC, require lock + funds)

| # | Test | Command | Status |
|---|------|---------|--------|
| 4.1 | Supply USDT on-chain (0.01 USDT) | `venus --chain 56 supply --asset USDT --amount 0.01` | SKIPPED - no BSC funds |
| 4.2 | Enter market USDT | `venus --chain 56 enter-market --asset USDT` | SKIPPED - no BSC funds |

> L4 blocked: test wallet has 0 BNB and 0 USDT on BSC (chain 56). Lark notification sent.
> Dry-run (L3) tests verify all calldata is correct.
