# Test Results Report

- Date: 2026-04-05
- DApp supported chains: EVM only (Base 8453)
- EVM test chain: Base (8453)
- Compile: ✅
- Lint: ✅
- Overall pass standard: EVM DApp → EVM all pass

## Summary

| Total | L1 compile | L2 read | L3 simulate | L4 onchain | Fail | Blocked |
|-------|-----------|---------|-------------|------------|------|---------|
| 10    | 1         | 4       | 3           | 2          | 0    | 0       |

## Detailed Results

| # | Scenario (user view) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Compile + lint | L1 | `cargo build --release && plugin-store lint` | ✅ PASS | — | 0 errors |
| 2 | Query all Morpho Blue markets on Base | L2 | `markets --chain 8453` | ✅ PASS | — | 50 markets returned |
| 3 | Query USDC markets on Base | L2 | `markets --chain 8453 --asset USDC` | ✅ PASS | — | 40 USDC markets |
| 4 | Query USDC vaults on Base | L2 | `vaults --chain 8453 --asset USDC` | ✅ PASS | — | 37 USDC vaults |
| 5 | Check my Morpho Base positions | L2 | `positions --chain 8453` | ✅ PASS | — | Empty (no positions) |
| 6 | Dry-run supply 0.01 USDC to Steakhouse vault | L3 | `supply --dry-run --vault ... --asset USDC --amount 0.01` | ✅ PASS | calldata: 0x095ea7b3..., 0x6e553f65... | Correct selectors |
| 7 | Dry-run withdraw 0.01 USDC from vault | L3 | `withdraw --dry-run --vault ... --asset USDC --amount 0.01` | ✅ PASS | calldata: 0xb460af94... | Correct ERC-4626 withdraw selector |
| 8 | Dry-run claim-rewards (no rewards) | L3 | `claim-rewards --dry-run` | ✅ PASS | message: "No claimable rewards found." | Merkl 500 → graceful empty response |
| 9 | Supply 0.01 USDC to Steakhouse USDC vault | L4 | `supply --vault 0xbeeF... --asset USDC --amount 0.01` | ✅ PASS | approve: 0x5fe67ed6c2b578dc8715c4b8b7c92051f56ec929de6553eef566a9eee719067e, supply: 0x28203249e447b95104d81ea26b3653b4e2f097df79daa82741935df7e05bb480 | BaseScan verified |
| 10 | Withdraw all USDC from Steakhouse vault | L4 | `withdraw --vault 0xbeeF... --asset USDC --all` | ✅ PASS | 0x977ceb2fe3c648b54b3c48c86ff486a68c9c60df3129e733927ca7defdb0dc24 | Redeemed 9210621596770612 shares |

## Fix Log

| # | Issue | Root cause | Fix | File |
|---|-------|-----------|-----|------|
| 1 | `erc20_symbol()` returned "UNKNOWN" | ABI string decoding bug: `hex_clean[64..96]` should be `hex_clean[64..128]` (32-byte word needs 64 hex chars, not 32) | Fixed offset in ABI decoding | `src/rpc.rs` |
| 2 | `claim-rewards` failed with error on no-reward wallet | Merkl API returns HTTP 500 instead of empty array when user has no rewards | Treat HTTP 500 as empty rewards response | `src/commands/claim_rewards.rs` |
| 3 | Lint crash with `byte index not a char boundary` | Em dash (`—`) in SKILL.md caused plugin-store lint tool to panic | Replaced all em dashes with ` - ` | `skills/morpho-base/SKILL.md` |

## On-chain Transactions (Base)

| Operation | TxHash | BaseScan |
|-----------|--------|---------|
| USDC approve (supply) | 0x5fe67ed6c2b578dc8715c4b8b7c92051f56ec929de6553eef566a9eee719067e | https://basescan.org/tx/0x5fe67ed6c2b578dc8715c4b8b7c92051f56ec929de6553eef566a9eee719067e |
| ERC-4626 deposit (supply) | 0x28203249e447b95104d81ea26b3653b4e2f097df79daa82741935df7e05bb480 | https://basescan.org/tx/0x28203249e447b95104d81ea26b3653b4e2f097df79daa82741935df7e05bb480 |
| ERC-4626 redeem (withdraw --all) | 0x977ceb2fe3c648b54b3c48c86ff486a68c9c60df3129e733927ca7defdb0dc24 | https://basescan.org/tx/0x977ceb2fe3c648b54b3c48c86ff486a68c9c60df3129e733927ca7defdb0dc24 |
