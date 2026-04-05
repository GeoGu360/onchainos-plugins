# Jito Plugin — Test Results

Date: 2026-04-05
Wallet: DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE
Binary: jito v0.1.0 (release build)

## L1: Unit Tests

```
$ cargo test
running 1 test
test commands::tests::test_withdraw_authority_pda ... ok
test result: ok. 1 passed; 0 failed; 0 ignored
```

| ID | Result | Notes |
|----|--------|-------|
| U1 | PASS | PDA = `6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS` ✓ |

## L2: Read-Only Integration Tests

| ID | Result | Output Snapshot |
|----|--------|-----------------|
| R1 | PASS | `sol_per_jitosol: "1.27127624"`, `estimated_apy_pct: "5.89"`, `total_staked_sol: "11227425.6369"` |
| R2 | PASS | `ok:false`, `"Jito only supports Solana (chain 501)"` |

## L3: Wallet-Dependent Read Tests

| ID | Result | Output Snapshot |
|----|--------|-----------------|
| W1 | PASS | `jitosol_balance: "0.008659678"`, `wallet: "DTEqFXy..."`, token account `59y8wu...` resolved via `getTokenAccountsByOwner` fallback |
| W2 | PASS | `dry_run:true`, `expected_jitosol: "0.000786611"`, `withdraw_authority: "6iQKfEy..."` |
| W3 | PASS | `dry_run:true`, `expected_sol: "0.006356381"`, delay note included |

## L4: Write (On-Chain) Tests

| ID | Result | txHash |
|----|--------|--------|
| S1 | PASS | `67msKUTGBVMbYrKoWTKrcWTpkq8q4anmDg74HsvDzz3Ja9u6W5jmD6MSuUyr9XRGw7MKZ2zz72fcjcU1FRw8FCYW` |

Solscan: https://solscan.io/tx/67msKUTGBVMbYrKoWTKrcWTpkq8q4anmDg74HsvDzz3Ja9u6W5jmD6MSuUyr9XRGw7MKZ2zz72fcjcU1FRw8FCYW

On-chain verification: JitoSOL balance increased from 0.007873067 → 0.008659678 (+0.000786611 JitoSOL for 0.001 SOL, matching expected at rate 1.27127624).

## Key Fixes Applied During Testing

1. **v0 versioned transaction format**: Added `0x80` version prefix byte and trailing `0x00` (empty address table lookups) to `SolanaMessage::serialize()`. Onchainos rejects legacy-format transactions with "Service temporarily unavailable".
2. **Existing token account**: Removed CreateATA instruction (ATA program simulation fails). Used `getTokenAccountsByOwner` to resolve existing JitoSOL token account `59y8wu...` as deposit destination.
3. **PDA correctness**: Python3 subprocess for Ed25519 on-curve check ensures correct withdraw authority derivation (earlier heuristic approach gave wrong PDA).
