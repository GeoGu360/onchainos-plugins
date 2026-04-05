# Test Results Report — Symbiotic

- Date: 2026-04-05
- DApp supported chains: Ethereum (EVM only, chain ID: 1)
- EVM test chain: Ethereum Mainnet (1)
- Compilation: ✅
- Lint: ✅
- Overall pass standard: EVM DApp → EVM all pass

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|-----------|---------|------------|------------|-------|---------|
| 11    | 2          | 4       | 3           | 2          | 0     | 0       |

## Detailed Results

| # | Scenario (User View) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Compile plugin | L1 | `cargo build --release` | ✅ PASS | — | 16 warnings only |
| 2 | Lint plugin | L1 | `cargo clean && plugin-store lint .` | ✅ PASS | — | 0 errors |
| 3 | List all Symbiotic vaults | L2 | `vaults --chain 1` | ✅ PASS | — | 20 vaults returned with TVL/APR |
| 4 | Filter wstETH vaults | L2 | `vaults --token wstETH --chain 1` | ✅ PASS | — | 4 wstETH vaults found |
| 5 | Get all vault APR rates | L2 | `rates --chain 1` | ✅ PASS | — | Sorted by APR desc, HYPER 7.09% top |
| 6 | Query wallet positions | L2 | `positions --address 0x87fb... --chain 1` | ✅ PASS | — | Returns empty (no prior positions) |
| 7 | Preview deposit 0.00003 wstETH | L3 | `deposit --token wstETH --amount 0.00003 --chain 1 --dry-run` | ✅ PASS | approve: `0x095ea7b3...`, deposit: `0x47e7ef24...` | Selectors correct |
| 8 | Preview withdraw 0.00003 wstETH | L3 | `withdraw --token wstETH --amount 0.00003 --chain 1 --dry-run` | ✅ PASS | calldata: `0xf3fef3a3...` | Correct selector |
| 9 | Simulate positions query | L3 | `positions --chain 1` (uses wallet resolve) | ✅ PASS | — | wallet resolved via onchainos wallet addresses |
| 10 | Deposit 0.00003 wstETH into Stakestone vault | L4 | `deposit --token wstETH --amount 0.00003 --vault 0xF40... --chain 1` | ✅ PASS | approve: `0x950893964111789fd90fdceeb231ac7b5932bd216dc16b39586dbfd50cf6a61b` deposit: `0x8d01cf6735381bc3aafe83c1bd5c3a9e81c9d531ba9cad7c83b373ca9328eac7` | Etherscan: block 0x17a9cd6 |
| 11 | Request withdraw 0.00003 wstETH from Stakestone vault | L4 | `withdraw --token wstETH --amount 0.00003 --vault 0xF40... --chain 1` | ✅ PASS | `0x4679ce02bfc7939cbefc8f9dfb074295eed1e0c890f1356bd0acd9c81686866b` | Epoch 25→26, ~7 day wait. Block 0x17a9cd9 |

## Fix Log

| # | Issue | Root Cause | Fix | File |
|---|-------|-----------|-----|------|
| 1 | `resolve_wallet` failed on chain 1 | `wallet balance --chain 1 --output json` not supported on Ethereum | Use `wallet addresses` which works on all chains | `src/onchainos.rs` |
| 2 | deposit tx returned "pending" on first attempt | 2-tx flow (approve + deposit) submitted too fast; approve not confirmed before deposit | Added 15s delay between approve and deposit calls | `src/commands/deposit.rs` |
| 3 | Legacy wstETH vault (0xC329...) reverts on `activeBalanceOf` | `DefaultCollateral` interface, not `IVault` | `find_vault_by_token` prefers non-legacy vaults | `src/api.rs` |
