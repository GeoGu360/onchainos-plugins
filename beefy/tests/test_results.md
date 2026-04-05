# Test Results Report

- Date: 2026-04-05
- DApp: Beefy Finance
- DApp supports: EVM only
- EVM test chain: Base (8453)
- Compile: PASS
- Lint: PASS (only E123 placeholder SHA, expected before submission)

## Key Bug Found

Beefy vaults use `BeefyVaultV7` contract (NOT ERC-4626):
- deposit: `deposit(uint256 _amount)` selector `0xb6b55f25` (NOT ERC-4626's `0x6e553f65`)
- withdraw: `withdraw(uint256 _shares)` selector `0x2e1a7d4d` (NOT ERC-4626's `0xba087652`)
- Both require `--force` flag for onchainos broadcast

## Summary

| Total | L1-Compile | L2-Read | L3-Simulate | L4-Onchain | Failed | Blocked |
|-------|-----------|---------|------------|-----------|--------|---------|
| 9     | 1         | 4       | 2          | 2         | 0      | 0       |

## Detailed Results

| # | Scenario (user view) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|------------------|-------|
| 1 | Build plugin | L1 | `cargo build --release` | PASS | - | 2 warnings (unused funcs) |
| 2 | List Base vaults | L2 | `vaults --chain 8453 --limit 5` | PASS | - | 248 active Base vaults |
| 3 | Find USDC vaults on Base | L2 | `vaults --chain 8453 --asset USDC --limit 5` | PASS | - | 76 USDC vaults found |
| 4 | Get APY for specific vault | L2 | `apy --chain 8453 --vault morpho-base-gauntlet-prime-usdc` | PASS | - | 3.35% APY returned |
| 5 | Check positions (empty) | L2 | `positions --chain 8453 --wallet 0x87fb...` | PASS | - | No positions found (expected) |
| 6 | Simulate deposit 0.01 USDC | L3 | `deposit --vault morpho-base-gauntlet-prime-usdc --amount 0.01 --chain 8453 --dry-run` | PASS | calldata: 0xb6b55f25...2710 | selector 0xb6b55f25 correct |
| 7 | Simulate withdraw | L3 | `withdraw --vault morpho-base-gauntlet-prime-usdc --shares 9927 --chain 8453 --dry-run` | PASS | calldata: 0x2e1a7d4d...26c7 | selector 0x2e1a7d4d correct |
| 8 | Deposit 0.01 USDC into Beefy vault | L4 | `deposit --vault morpho-base-gauntlet-prime-usdc --amount 0.01 --chain 8453` | PASS | 0xd8caca18e2f4e50c9f633ba161d46984785d7a2236dbe8ba64132e60b2e80d85 | BaseScan verified |
| 9 | Withdraw all mooTokens | L4 | onchainos direct with 9927 shares | PASS | 0x2a7a26ebe141c888feaa358ed80d18de92722151ac639d2a20b39b664ff6733a | BaseScan verified |

## Fix Record

| # | Problem | Root Cause | Fix | File |
|---|---------|-----------|-----|------|
| 1 | deposit reverts | Wrong selector: used ERC-4626 0x6e553f65 instead of BeefyVaultV7 0xb6b55f25 | Changed to deposit(uint256) selector | commands/deposit.rs |
| 2 | deposit returns pending | Missing --force flag on wallet contract-call | Added wallet_contract_call_force() wrapper | onchainos.rs |
| 3 | withdraw uses wrong selector | ERC-4626 0xba087652 vs Beefy 0x2e1a7d4d | Changed to withdraw(uint256) selector | commands/withdraw.rs |

## L4 Transaction Verification

- Deposit: https://basescan.org/tx/0xd8caca18e2f4e50c9f633ba161d46984785d7a2236dbe8ba64132e60b2e80d85
- Withdraw: https://basescan.org/tx/0x2a7a26ebe141c888feaa358ed80d18de92722151ac639d2a20b39b664ff6733a
