# Test Results Report

- Date: 2026-04-05
- DApp supported chains: EVM (Ethereum mainnet, chain 1)
- EVM test chain: Ethereum mainnet (1)
- Compile: PASS
- Lint: PASS
- Overall pass standard: EVM DApp - all EVM read/simulate ops pass

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|-----------|---------|-------------|-------------|--------|---------|
| 8     | 2         | 2       | 4           | 0           | 0      | 3 (skipped per policy) |

## Detailed Results

| # | Scenario (user view) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Build plugin | L1 | `cargo build --release` | PASS | - | 0 errors, 5 warnings (unused consts) |
| 2 | Lint plugin | L1 | `cargo clean && plugin-store lint .` | PASS | - | "Plugin 'usde-staking' passed all checks!" |
| 3 | Check current sUSDe APY and exchange rate | L2 | `get-rates` | PASS | - | APY 3.49%, 1 sUSDe = 1.226168 USDe, TVL $3.52B |
| 4 | View my sUSDe position | L2 | `get-positions --address 0x87fb...` | PASS | - | 0 balances, no pending unstake |
| 5 | Simulate staking 0.01 USDe | L3 | `stake --amount 0.01 --dry-run` | PASS | Approve: `0x095ea7b3...`, Deposit: `0x6e553f65...` | Selectors correct, expected 0.008155 sUSDe |
| 6 | Simulate request-unstake by shares | L3 | `request-unstake --shares 0.01 --dry-run` | PASS | `0x9343d9e1...` | cooldownShares selector correct |
| 7 | Simulate request-unstake by assets | L3 | `request-unstake --assets 0.01 --dry-run` | PASS | `0xcdac52ed...` | cooldownAssets selector correct |
| 8 | Simulate claim-unstake | L3 | `claim-unstake --dry-run` | PASS | `0xf2888dbb...` | unstake selector correct |
| 9 | Stake 0.01 USDe on-chain | L4 | `stake --amount 0.01` | SKIPPED | - | No USDe in test wallet; GUARDRAILS rule 5 |
| 10 | Request-unstake on-chain | L4 | `request-unstake --shares ...` | SKIPPED | - | Cooldown-gated; pipeline rule 10 |
| 11 | Claim-unstake on-chain | L4 | `claim-unstake` | SKIPPED | - | Requires prior request-unstake; cooldown-gated |

## Selector Verification

| Operation | Function | Selector | Verified |
|-----------|---------|---------|---------|
| approve | `approve(address,uint256)` | `0x095ea7b3` | cast sig ✅ |
| stake (deposit) | `deposit(uint256,address)` | `0x6e553f65` | cast sig ✅ |
| request-unstake shares | `cooldownShares(uint256)` | `0x9343d9e1` | cast sig ✅ |
| request-unstake assets | `cooldownAssets(uint256)` | `0xcdac52ed` | cast sig ✅ |
| claim-unstake | `unstake(address)` | `0xf2888dbb` | cast sig ✅ |
| query cooldowns | `cooldowns(address)` | `0x01320fe2` | cast sig + on-chain ✅ |

## Fix Log

No fixes required. First-pass all tests pass.

## L4 Skip Rationale

**Stake skipped (L4-1):**
- Test wallet `0x87fb...` has 0 USDe on Ethereum mainnet
- GUARDRAILS.md rule: "use USDe if wallet has it, else skip L4 stake"
- All calldata has been verified via L3 dry-run

**Unstake/claim skipped (L4-2, L4-3):**
- Pipeline instruction rule 10: "For unstaking: mark cooldown-gated ops as dry-run (no funds at risk during cooldown period)"
- The 7-day cooldown would lock test funds; acceptable L3 verification
- Note: cooldown is currently 1 day (86400s) per contract, not 7 days as originally documented
