# USDe Staking Test Cases

## Level 1: Compile + Lint

| # | Test | Command | Expected |
|---|------|---------|---------|
| L1-1 | Build | `cargo build --release` | Compiles with 0 errors |
| L1-2 | Lint | `cargo clean && plugin-store lint .` | 0 lint errors |

## Level 2: Read Tests (no gas)

| # | Scenario | Command | Expected |
|---|---------|---------|---------|
| L2-1 | Check current sUSDe APY | `get-rates` | APY > 0, exchange rate > 1.0 |
| L2-2 | View position for test wallet | `get-positions --address 0x87fb...` | Returns balances (zero is valid) |
| L2-3 | View position for known sUSDe holder | `get-positions --address 0x...` | Shows non-zero sUSDe balance |

## Level 3: Dry-Run (calldata verification)

| # | Scenario | Command | Expected Selector |
|---|---------|---------|-----------------|
| L3-1 | Stake dry-run | `stake --amount 0.01 --from 0x87fb... --dry-run` | Approve: `0x095ea7b3`, Deposit: `0x6e553f65` |
| L3-2 | Request-unstake by shares dry-run | `request-unstake --shares 0.01 --from 0x87fb... --dry-run` | `0x9343d9e1` |
| L3-3 | Request-unstake by assets dry-run | `request-unstake --assets 0.01 --from 0x87fb... --dry-run` | `0xcdac52ed` |
| L3-4 | Claim-unstake dry-run | `claim-unstake --from 0x87fb... --dry-run` | `0xf2888dbb` |

## Level 4: On-Chain Tests

| # | Scenario | Command | Policy |
|---|---------|---------|-------|
| L4-1 | Stake 0.01 USDe | `stake --amount 0.01 --from 0x87fb...` | SKIPPED — no USDe in test wallet |
| L4-2 | Request-unstake | `request-unstake --shares ...` | SKIPPED — cooldown-gated, dry-run sufficient |
| L4-3 | Claim-unstake | `claim-unstake` | SKIPPED — requires prior request-unstake |

**Note on L4 skips:**
- L4-1 skip: Test wallet has 0 USDe. GUARDRAILS rule 5: "use USDe if wallet has it, else skip L4 stake"
- L4-2/L4-3 skip: Pipeline instructions rule 10: "For unstaking: mark cooldown-gated ops as dry-run (no funds at risk during cooldown period)"
- All read ops (L2) and calldata verification (L3) pass fully
