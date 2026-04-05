# Test Results Report

- Date: 2026-04-05
- DApp: Venus Core Pool
- DApp supported chains: BSC only (chain 56)
- EVM test chain: BSC (56)
- Compile: OK
- Lint: OK

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|-----------|---------|------------|-------------|--------|---------|
| 12    | 3         | 3       | 7          | 0           | 0      | 2       |

## Detailed Results

| # | Scenario (user view) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Compile debug build | L1 | `cargo build` | PASS | - | No errors |
| 2 | Compile release build | L1 | `cargo build --release` | PASS | - | Binary at target/release/venus (4.35MB) |
| 3 | plugin-store lint | L1 | `cargo clean && plugin-store lint .` | PASS | - | "Plugin 'venus' passed all checks!" |
| 4 | List Venus markets on BSC | L2 | `venus --chain 56 get-markets` | PASS | - | 48 markets returned; vUSDT supply APY 0.2548%, borrow APY 0.5273% |
| 5 | Check my Venus positions | L2 | `venus --chain 56 get-positions --wallet 0x87fb...` | PASS | - | Empty positions (no funds on BSC) |
| 6 | Invalid chain rejected | L2 | `venus --chain 1 get-markets` | PASS | - | Error: "Unsupported chain ID: 1. Venus Core Pool is only on BSC (56)." |
| 7 | Simulate supply 0.01 USDT | L3 | `venus --chain 56 --dry-run supply --asset USDT --amount 0.01` | PASS | calldata: `0xa0712d68000000000000000000000000000000000000000000000000002386f26fc10000` | Selector 0xa0712d68 = mint(uint256) CORRECT |
| 8 | Simulate supply 0.001 BNB | L3 | `venus --chain 56 --dry-run supply --asset BNB --amount 0.001` | PASS | calldata: `0x1249c58b` | Selector 0x1249c58b = mint() CORRECT; amt=1000000000000000 wei |
| 9 | Simulate withdraw USDT | L3 | `venus --chain 56 --dry-run withdraw --asset USDT --amount 0.01` | PASS | calldata: `0x852a12e3...` | Selector 0x852a12e3 = redeemUnderlying(uint256) CORRECT |
| 10 | Simulate borrow USDT | L3 | `venus --chain 56 --dry-run borrow --asset USDT --amount 0.01` | PASS | calldata: `0xc5ebeaec...` | Selector 0xc5ebeaec = borrow(uint256) CORRECT |
| 11 | Simulate repay USDT | L3 | `venus --chain 56 --dry-run repay --asset USDT --amount 0.01` | PASS | calldata: `0x0e752702...` | Selector 0x0e752702 = repayBorrow(uint256) CORRECT |
| 12 | Simulate enter-market USDT | L3 | `venus --chain 56 --dry-run enter-market --asset USDT` | PASS | calldata: `0xc2998238...` | Selector 0xc2998238 = enterMarkets(address[]) CORRECT |
| 13 | Simulate claim XVS rewards | L3 | `venus --chain 56 --dry-run claim-rewards` | PASS | - | Selector 0xadcd5fb9 = claimVenus(address) CORRECT |
| 14 | Supply 0.01 USDT on-chain | L4 | `venus --chain 56 supply --asset USDT --amount 0.01` | SKIPPED | - | No BNB/USDT on BSC test wallet (0.0 balance) |
| 15 | Enter market USDT | L4 | `venus --chain 56 enter-market --asset USDT` | SKIPPED | - | No gas on BSC |

## Bug Fixes

| # | Issue | Root Cause | Fix | File |
|---|-------|-----------|-----|------|
| 1 | BNB supply calldata was 0x32807ec0 | alloy sol! macro named function mintBNB() instead of mint() | Replaced sol! macro with hardcoded `VBNB_MINT_CALLDATA = "0x1249c58b"` | src/commands/supply.rs |

## L4 Status

L4 on-chain tests are SKIPPED because the test wallet (0x87fb0647faabea33113eaf1d80d67acb1c491b90) has 0 BNB and 0 USDT on BSC (chain 56). Lark notification sent requesting BSC top-up.

All L1/L2/L3 tests PASS. All selectors verified correct by cast sig. Plugin is ready for submission.
