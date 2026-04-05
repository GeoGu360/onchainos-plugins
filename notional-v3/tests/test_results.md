# Test Results Report — notional-v3

- Date: 2026-04-05
- DApp supported chains: Ethereum mainnet (chain 1) ONLY (Notional V3 legacy paused; this plugin targets Notional Exponent V4)
- EVM test chain: Ethereum mainnet (1)
- Compile: ✅
- Lint: ✅
- Overall: PASS (L2 + L3 full coverage, L4 enter-position confirmed on-chain)

## Summary

| Total | L1 Compile | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|-----------|---------|-------------|-------------|--------|---------|
| 13    | 2         | 4       | 5           | 2           | 0      | 1       |

## Detailed Results

| # | Scenario (user view) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Compile release binary | L1 | `cargo build --release` | ✅ PASS | — | 0 errors, 12 warnings (dead code) |
| 2 | Lint plugin | L1 | `cargo clean && plugin-store lint .` | ✅ PASS | — | 0 errors |
| 3 | List all Notional Exponent vaults | L2 | `get-vaults` | ✅ PASS | — | 8 vaults returned |
| 4 | Filter vaults by USDC | L2 | `get-vaults --asset USDC` | ✅ PASS | — | 6 USDC vaults |
| 5 | Filter vaults by WETH | L2 | `get-vaults --asset WETH` | ✅ PASS | — | 2 WETH vaults |
| 6 | Get positions (empty wallet) | L2 | `get-positions --wallet 0x87fb...` | ✅ PASS | — | count=0 returned correctly |
| 7 | Simulate enter-position (USDC vault) | L3 | `--dry-run enter-position --vault 0x9fb5... --amount 0.01 --asset USDC` | ✅ PASS | 0xde13c617... | selector correct |
| 8 | Simulate exit-position | L3 | `--dry-run exit-position --vault 0x9fb5... --shares 1000000` | ✅ PASS | 0x8a363181... | selector correct |
| 9 | Simulate initiate-withdraw | L3 | `--dry-run initiate-withdraw --vault 0xaf14... --shares 1000000` | ✅ PASS | 0x37753799... | selector correct |
| 10 | Simulate claim-rewards | L3 | `--dry-run claim-rewards --vault 0x9fb5...` | ✅ PASS | 0xf1e42ccd... | selector correct |
| 11 | Reject unsupported chain | L3 | `--chain 8453 get-vaults` | ✅ PASS | — | "not supported" error returned |
| 12 | Enter weETH leveraged vault with 0.00005 WETH | L4 | `enter-position --vault 0x7f72... --amount 0.00005 --asset WETH` | ✅ PASS | [0x487fb9ff...](https://etherscan.io/tx/0x487fb9ff308d3a324743311449d47bca861525e8b384feb099a2fd96eb3a2942) | 15s delay between approve+enterPosition required |
| 13 | Exit weETH leveraged vault | L4 | `exit-position --vault 0x7f72... --shares all` | ⚠️ BLOCKED | — | weETH vault requires unstaking queue (EtherFi protocol constraint); exitPosition reverts immediately |

## Fix Log

| # | Issue | Root Cause | Fix | File |
|---|-------|-----------|-----|------|
| 1 | `tx_hash: pending` on all write ops | `wallet contract-call` needs `--force` flag to broadcast immediately | Added `--force` to all `wallet_contract_call` invocations | `src/onchainos.rs` |
| 2 | `ok: true, tx_hash: pending` on revert | Error response not checked — `extract_tx_hash` returned "pending" for `{"ok": false, "error": "..."}` | Added `ok: false` check in `wallet_contract_call`; now returns `Err` | `src/onchainos.rs` |
| 3 | Enter-position approve→deposit returns `pending` on first call | 3s delay insufficient; approve needs to confirm (~12s) before deposit | Increased delay from 3s to 15s | `src/commands/enter_position.rs` |

## Notes

- **WETH wrapping**: Test wallet had ETH but not WETH. Used `onchainos wallet send --receipt 0xC02a... --readable-amount 0.00005` to wrap ETH → WETH before testing enter-position
- **No USDC in wallet**: Most Notional Exponent vaults require USDC. Tests were done on the only WETH vault available
- **weETH exit blocked**: The weETH leveraged vault requires an unstaking period before redemption. `exitPosition` reverts immediately — this is expected protocol behavior. Users should use `initiate-withdraw` first (separate cooldown period), then a final claim
- **Etherscan links**: All L4 transactions can be verified at etherscan.io
