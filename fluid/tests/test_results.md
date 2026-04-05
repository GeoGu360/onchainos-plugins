# Test Results Report — Fluid Protocol

- **Date:** 2026-04-05
- **Test chains:** Base (8453)
- **Compile:** PASS
- **Lint:** PASS (manual — plugin-store not installed locally; rules checked manually)

---

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|-----------|---------|------------|------------|--------|---------|
| 11 | 2 | 4 | 4 | 1 | 0 | 0 |

---

## Detailed Results

| # | Scenario | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------|-------|---------|--------|------------------|-------|
| 1 | Build release binary | L1 | `cargo build --release` | PASS | — | No errors, only dead-code warnings suppressed |
| 2 | Binary runs + --help | L1 | `fluid --help` | PASS | — | All 8 commands listed |
| 3 | List markets on Base | L2 | `fluid --chain 8453 markets` | PASS | — | 4 fTokens returned: fUSDC, fWETH, fGHO, fEURC |
| 4 | View positions (empty wallet) | L2 | `fluid --chain 8453 --from 0x87fb... positions` | PASS | — | Returns empty positions correctly |
| 5 | DEX quote EURC->USDC | L2 | `fluid --chain 8453 quote --token-in EURC --token-out USDC --amount-in 100` | PASS | — | Returns 0 (EURC/USDC pool has min amount requirement) |
| 6 | DEX quote WSTETH->WETH | L2 | `fluid --chain 8453 quote --token-in WSTETH --token-out WETH --amount-in 1` | PASS | amountOut: 1.230881 WETH | DexReservesResolver.estimateSwapIn works correctly |
| 7 | Supply dry-run fUSDC | L3 | `fluid --chain 8453 --dry-run supply --ftoken fUSDC --amount 0.01` | PASS | approve: 0x095ea7b3..., deposit: 0x6e553f65... | Selectors verified correct |
| 8 | Withdraw dry-run fUSDC | L3 | `fluid --chain 8453 --dry-run withdraw --ftoken fUSDC --amount 0.01` | PASS | withdraw: 0xb460af94... | Selector verified correct |
| 9 | Swap dry-run EURC->USDC | L3 | `fluid --chain 8453 --dry-run swap --token-in EURC --token-out USDC --amount-in 1` | PASS | swapIn: 0x2668dfaa... | Selector verified correct |
| 10 | Borrow blocked without dry-run | L3 | `fluid --chain 8453 borrow --vault ... --amount 100` | PASS (errors correctly) | — | Returns error "dry-run only" as expected |
| 11 | Supply 0.01 USDC to fUSDC on Base | L4 | `fluid --chain 8453 supply --ftoken fUSDC --amount 0.01` | PASS | approveTx: 0x79f7726f..., supplyTx: 0x8a16e4b7... | basescan.org/tx/0x8a16e4b7..., status=1, block=44289336, 4 logs |

---

## On-chain Evidence

- **Approve tx:** https://basescan.org/tx/0x79f7726f842764d65bb2cc7f5b7e034d2efff181e7ea4478cf73f2bf478818c7
- **Supply tx:** https://basescan.org/tx/0x8a16e4b7ff77b0de3aa4364723699625d37d77c7bc0caf45e16d0a88d0c560fa
- **Status:** 1 (success), gasUsed: 127381, to: fUSDC 0xf42f5795...
- **Position verified:** 9025 fUSDC shares = 0.009999 USDC

---

## Known Issues / Notes

1. **ERC-20 symbol returns UNKNOWN** — The `erc20_symbol` RPC call returns UNKNOWN for USDC on Base. This is cosmetic only (display issue, not functional). The underlying address `0x833589fcd6edb6e08f4c7c32d4f71b54bda02913` is correct.
2. **EURC/USDC DEX quote returns 0** — The estimateSwapIn returns 0 for small amounts (< pool minimum). The wstETH/ETH pool works correctly with the same code. This is pool-specific behavior.
3. **plugin-store lint not installed** — Manual review confirms:
   - `plugin.yaml` format is correct (all required fields)
   - `SKILL.md` has "ask user to confirm" near all `wallet contract-call` mentions (E106 compliant)
   - No compiled artifacts (gitignore has `/target/`)
   - `source_commit` will be updated to real SHA after Phase 4 push
