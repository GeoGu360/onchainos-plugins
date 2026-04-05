# Test Results Report — Frax Ether Plugin

- **Date:** 2026-04-05
- **DApp supported chains:** Ethereum mainnet (chain ID: 1) — EVM only
- **Test chain:** Ethereum mainnet (1)
- **Binary:** `frax-ether v0.1.0`
- **Compile:** ✅ PASS
- **Lint:** ✅ PASS (manual — plugin-store not installed locally)
- **Overall:** ✅ ALL PASS

---

## Summary

| Total | L1 Build | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|----------|---------|-------------|-------------|--------|---------|
| 9     | 2        | 2       | 3           | 2           | 0      | 0       |

---

## Detailed Results

| # | Scenario (User View) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Build plugin (debug) | L1 | `cargo build` | ✅ PASS | — | 3 unused const warnings, 0 errors |
| 2 | Build plugin (release) | L1 | `cargo build --release` | ✅ PASS | — | 0 errors |
| 3 | Query sfrxETH APR and exchange rate | L2 | `frax-ether rates` | ✅ PASS | — | APR: 2.85%, 1 sfrxETH = 1.1547 frxETH |
| 4 | Query wallet frxETH/sfrxETH positions | L2 | `frax-ether positions --address 0x87fb...` | ✅ PASS | — | Returns balances and USD values |
| 5 | Simulate staking ETH to frxETH | L3 | `stake --amount 0.00005 --dry-run` | ✅ PASS | calldata: `0x5bcb2fc6` | submit() selector correct |
| 6 | Simulate staking frxETH to sfrxETH | L3 | `stake-frx --amount 0.00005 --dry-run` | ✅ PASS | approve: `0x095ea7b3...`, deposit: `0x6e553f65...` | ERC-4626 selectors correct |
| 7 | Simulate redeeming sfrxETH to frxETH | L3 | `unstake --amount 0.00005 --dry-run` | ✅ PASS | calldata: `0xba087652...` | redeem selector correct |
| 8 | Stake 0.00005 ETH to get frxETH | L4 | `frax-ether stake --amount 0.00005 --chain 1` | ✅ PASS | [0x22427d69...](https://etherscan.io/tx/0x22427d6913c8b671bfaaae0b7eeabb19f222cac94fc57ef11bca34053e835e18) | Wallet received 0.00005 frxETH |
| 9 | Stake frxETH to earn yield in sfrxETH | L4 | `frax-ether stake-frx --amount 0.00005 --chain 1` | ✅ PASS | Approve: [0xa4985e96...](https://etherscan.io/tx/0xa4985e96eeb21455effdcd34bedecf17c7ebc6ebb0fa9279c2fe064091122d8e) Deposit: [0xfd1be0bd...](https://etherscan.io/tx/0xfd1be0bd34b54134206a0dd1edaba039234c0811e168c96754346e8fd79dcc86) | frxETH → sfrxETH success |

---

## Fix Log

| # | Issue | Root Cause | Fix | File |
|---|-------|-----------|-----|------|
| 1 | `stake` L4 tx reverted: `submit(address)` | `submit(address)` (0xa1903eab) reverts when called with our wallet; possibly referral whitelist restriction | Changed to `submit()` (no args, 0x5bcb2fc6) which works correctly | `src/config.rs`, `src/commands/stake.rs` |

---

## Fund Usage

| Op | Amount | TxHash |
|----|--------|--------|
| stake ETH → frxETH | 0.00005 ETH | 0x22427d69... |
| approve frxETH | 0 ETH (gas only) | 0xa4985e96... |
| deposit frxETH → sfrxETH | 0.00005 frxETH (gas only) | 0xfd1be0bd... |
| **Remaining ETH** | ~0.005199 ETH | — |
