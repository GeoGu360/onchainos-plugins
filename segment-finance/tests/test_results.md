# Segment Finance — Test Results

- Date: 2026-04-05
- DApp supported chains: BSC (chain 56)
- EVM test chain: BSC (56)
- Compile: ✅
- Lint: ✅
- Overall pass standard: EVM-only DApp, BSC operations must pass

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|-----------|---------|------------|------------|--------|---------|
| 11 | 2 | 4 | 6 | 0 (SKIPPED) | 0 | 0 |

## Detailed Results

| # | Scenario (User Perspective) | Level | Command | Result | TxHash / Calldata | Notes |
|---|-----------------------------|-------|---------|--------|-------------------|-------|
| 1 | Compile plugin | L1 | `cargo build --release` | ✅ PASS | - | 4 dead_code warnings, 0 errors |
| 2 | Lint plugin | L1 | `cargo clean && plugin-store lint .` | ✅ PASS | - | "passed all checks!" |
| 3 | List Segment Finance markets on BSC | L2 | `get-markets --chain 56` | ✅ PASS | - | 8 markets, BNB $594, USDT $0.9997 |
| 4 | Check positions for test wallet | L2 | `get-positions --wallet 0x87fb...` | ✅ PASS | - | 0 positions, NO_POSITION (as expected) |
| 5 | Invalid chain rejected | L2 | `--chain 1 get-markets` | ✅ PASS | - | Correct error: "only on BSC (56)" |
| 6 | Invalid asset rejected | L2 | `supply --asset XYZ --dry-run` | ✅ PASS | - | Correct error: "Unsupported asset: XYZ" |
| 7 | Simulate supply USDT | L3 | `supply --asset USDT --amount 0.01 --dry-run` | ✅ PASS | calldata: `0xa0712d68...` | selector 0xa0712d68 = mint(uint256) ✅ |
| 8 | Simulate supply BNB | L3 | `supply --asset BNB --amount 0.001 --dry-run` | ✅ PASS | calldata: `0x1249c58b` | selector 0x1249c58b = mint() payable ✅ |
| 9 | Simulate withdraw USDT | L3 | `withdraw --asset USDT --amount 0.01 --dry-run` | ✅ PASS | calldata: `0x852a12e3...` | selector = redeemUnderlying(uint256) ✅ |
| 10 | Simulate borrow USDT | L3 | `borrow --asset USDT --amount 0.01 --dry-run` | ✅ PASS | calldata: `0xc5ebeaec...` | selector = borrow(uint256) ✅ |
| 11 | Simulate repay USDT | L3 | `repay --asset USDT --amount 0.01 --dry-run` | ✅ PASS | calldata: `0x0e752702...` | selector = repayBorrow(uint256) ✅ |
| 12 | Simulate enter-market USDT | L3 | `enter-market --asset USDT --dry-run` | ✅ PASS | calldata: `0xc2998238...` | selector = enterMarkets(address[]) ✅ |

## L4 On-chain Tests

⚠️ L4 SKIPPED — Test wallet has 0 BNB and 0 tokens on BSC (chain 56).

Per GUARDRAILS rule 8: "If wallet has no funds on target chain, mark L4 as SKIPPED."

Wallet balance on BSC: `totalValueUsd: "0.00"`, `tokenAssets: []`

L2 and L3 tests confirm:
- All RPC reads work correctly (live market data, prices, rates)
- All calldata encoding is correct (selectors verified)
- Protocol is active: seUSDT getCash = 1292 USDT, BNB price = $594

## Fix Record

No fixes needed — clean first pass.
