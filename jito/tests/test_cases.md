# Jito Plugin — Test Cases

## L1: Unit Tests (cargo test)

| ID | Test | Description | Expected |
|----|------|-------------|----------|
| U1 | `test_withdraw_authority_pda` | Verify PDA derivation for Jito stake pool withdraw authority | `6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS` |

## L2: Read-Only Integration Tests (mainnet RPC, no wallet)

| ID | Command | Input | Expected |
|----|---------|-------|----------|
| R1 | `jito rates --chain 501` | — | `ok:true`, `sol_per_jitosol` > 1.0, `estimated_apy_pct` > 0 |
| R2 | `jito rates --chain 999` | invalid chain | `ok:false`, error message |

## L3: Wallet-Dependent Read Tests

| ID | Command | Input | Expected |
|----|---------|-------|----------|
| W1 | `jito positions --chain 501` | wallet logged in | `ok:true`, `jitosol_balance` ≥ 0, `wallet` non-empty |
| W2 | `jito stake --amount 0.001 --chain 501 --dry-run` | wallet logged in | `ok:true`, `dry_run:true`, `expected_jitosol` > 0 |
| W3 | `jito unstake --amount 0.005 --chain 501 --dry-run` | wallet logged in | `ok:true`, `dry_run:true`, `expected_sol` > 0 |

## L4: Write (On-Chain) Tests

| ID | Command | Input | Expected |
|----|---------|-------|----------|
| S1 | `jito stake --amount 0.001 --chain 501` | 0.001 SOL | `ok:true`, `txHash` is a valid 88-char base58 string |

Note: unstake live test skipped — requires epoch-delayed stake account deactivation, managed via Jito webapp.
