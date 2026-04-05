# Test Results — Benqi Lending

- Date: 2026-04-05
- DApp supported chains: Avalanche C-Chain only (EVM, chain 43114)
- EVM test chain: Avalanche C-Chain (43114)
- Compile: ✅
- Lint: ✅ (0 errors, E123 on placeholder SHA expected pre-submission)
- Overall pass standard: EVM DApp — EVM L1-L3 all pass

## Summary

| Total | L1 compile | L2 read | L3 simulate | L4 on-chain | Failed | Blocked |
|-------|-----------|---------|------------|------------|--------|---------|
| 12    | 2         | 3       | 7          | 0 (SKIPPED) | 0    | 0       |

## Detailed Results

| # | Scenario (user perspective) | Level | Command | Result | Calldata / TxHash | Notes |
|---|---------------------------|-------|---------|--------|-------------------|-------|
| 1 | Compile benqi binary | L1 | `cargo build --release` | ✅ PASS | - | 4 unused warnings, no errors |
| 2 | Lint plugin | L1 | `cargo clean && plugin-store lint .` | ✅ PASS | - | E123 placeholder SHA only |
| 3 | View all Benqi lending markets on Avalanche | L2 | `benqi --chain 43114 markets` | ✅ PASS | - | 8 markets returned with APR/exchange rate |
| 4 | Check my Benqi positions | L2 | `benqi --chain 43114 positions --wallet 0x87fb...` | ✅ PASS | - | Empty positions, account liquidity $0 |
| 5 | Wrong chain error | L2 | `benqi --chain 1 markets` | ✅ PASS | - | Returns error: chain 43114 required |
| 6 | Preview supplying 0.01 USDC to Benqi | L3 | `benqi --chain 43114 --dry-run supply --asset USDC --amount 0.01` | ✅ PASS | approve: `0x095ea7b3...`, mint: `0xa0712d68...` | Correct 2-step ERC20 flow |
| 7 | Preview supplying 0.001 AVAX to Benqi | L3 | `benqi --chain 43114 --dry-run supply --asset AVAX --amount 0.001` | ✅ PASS | `0x1249c58b` | Correct payable mint() |
| 8 | Preview redeeming 0.01 USDC from Benqi | L3 | `benqi --chain 43114 --dry-run redeem --asset USDC --amount 0.01` | ✅ PASS | `0x852a12e3...` | redeemUnderlying selector correct |
| 9 | Preview borrowing 1 USDC (dry-run only) | L3 | `benqi --chain 43114 borrow --asset USDC --amount 1.0` | ✅ PASS | `0xc5ebeaec...` | Always dry-run, never broadcasts |
| 10 | Preview repaying 1 USDC (dry-run only) | L3 | `benqi --chain 43114 repay --asset USDC --amount 1.0` | ✅ PASS | approve: `0x095ea7b3...`, repay: `0x0e752702...` | Always dry-run |
| 11 | Preview claiming QI rewards | L3 | `benqi --chain 43114 --dry-run claim-rewards --reward-type 0` | ✅ PASS | `0x0952c563...` | Comptroller.claimReward(0, addr) |
| 12 | Preview claiming AVAX rewards | L3 | `benqi --chain 43114 --dry-run claim-rewards --reward-type 1` | ✅ PASS | `0x0952c563...` | Comptroller.claimReward(1, addr) |
| L4 | Supply AVAX on-chain | L4 | SKIPPED | SKIPPED | - | No AVAX on test wallet (chain 43114) |
| L4 | Claim rewards on-chain | L4 | SKIPPED | SKIPPED | - | No AVAX on test wallet (chain 43114) |

## Fix Record

No fixes required — first compilation passed.

## Notes

- Benqi uses per-timestamp interest rates (`supplyRatePerTimestamp`/`borrowRatePerTimestamp`) unlike Compound V2's per-block rates
- L4 skipped: test wallet has 0 balance on Avalanche C-Chain. All L1-L3 tests pass.
- All selectors verified via `cast sig` and empirically confirmed via eth_call on Avalanche mainnet
