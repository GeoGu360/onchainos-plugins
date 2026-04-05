# Test Results Report

- Date: 2026-04-05
- DApp supported chains: EVM (Ethereum 1, BSC 56) + Solana (read-only queries)
- EVM test chain: Ethereum mainnet (1) for dry-run; BSC (56) for dry-run
- Solana test chain: mainnet (501) - address validation only
- Compile: PASS
- Lint: PASS
- Overall pass criteria: EVM dry-run coverage + API read coverage

## Summary

| Total | L1-Compile | L2-Read | L3-Simulate | L4-OnChain | Failed | Blocked |
|-------|-----------|---------|-------------|-----------|--------|---------|
| 8 | 2 | 4 | 3 | 0 | 0 | 0 |

Note: L4 on-chain bridge tests skipped per pipeline instructions ("actual cross-chain bridge tx is expensive so dry-run only").

## Detailed Results

| # | Scenario (user view) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Build binary from source | L1 | `cargo build --release` | PASS | - | 0 errors, 0 warnings |
| 2 | Lint plugin structure | L1 | `cargo clean && plugin-store lint .` | PASS | - | 0 errors |
| 3 | List supported tokens and bridge fees | L2 | `get-tokens` | PASS | - | ETH: USDT/USDC; SOL: aeUSDT/aeUSDC; BSC: ABR/BNB; AVA: AVAX/ABR |
| 4 | Validate Solana recipient address | L2 | `check-address --chain SOL --address DTEq...` | PASS | - | `{"result": true, "status": "OK"}` |
| 5 | Validate Ethereum recipient address | L2 | `check-address --chain ETH --address 0x87fb...` | PASS | - | `{"result": true, "status": "OK"}` |
| 6 | Check status of unknown bridge tx | L2 | `get-tx-status --lock-id 12345` | PASS (expected error) | - | Returns "not yet confirmed" as expected |
| 7 | Dry-run bridge 0.01 USDT ETH -> BSC | L3 | `bridge --chain 1 --token USDT --amount 0.01 --dest-chain BSC --dry-run` | PASS | lockCalldata: `0x7bacc91e...42534300...2710` | Selector 0x7bacc91e correct; BSC dest 42534300 correct; amount 0x2710=10000 (0.01 USDT with 6 dec) |
| 8 | Dry-run bridge 0.01 USDT ETH -> SOL | L3 | `bridge --chain 1 --token USDT --amount 0.01 --dest-chain SOL --dry-run` | PASS | lockCalldata: `0x7bacc91e...534f4c00...2710` | SOL dest 534f4c00 correct; Solana address decoded to bytes32 correctly |
| 9 | Dry-run bridge 1 ABR BSC -> ETH | L3 | `bridge --chain 56 --token ABR --amount 1.0 --dest-chain ETH --dry-run` | PASS | lockCalldata: `0x7bacc91e...45544800...de0b6b3a7640000` | ETH dest 45544800 correct; amount 1e18 wei correct for 18-decimal token |

## API Accessibility Note

The Allbridge Classic API (`https://allbridgeapi.net`) is accessible via Rust reqwest (TLS fingerprinting bypasses Cloudflare), but blocked by curl and Python urllib. All API calls in the plugin use reqwest and work correctly.

API endpoints tested:
- `GET /token-info` - PASS (200, returns chain->tokens map)
- `GET /check/{chain}/address/{addr}` - PASS (200, validates SOL and ETH addresses)
- `GET /sign/{lockId}` - PASS (returns correct error for non-existent lock ID)

## Protocol Note

Allbridge Classic is deprecated and scheduled for shutdown in mid-2026. Token support is limited:
- ETH: USDT, USDC (6 decimals)
- BSC: ABR, BNB (18 decimals); no USDT/USDC remaining
- POL: 0 tokens remaining (deprecated)
- SOL: aeUSDT, aeUSDC (wrapped, 9 decimals)
- AVA: AVAX, ABR

SKILL.md updated to reflect limited token availability and deprecation status.

## Selector Verification

| Function | Signature | Selector | Verified |
|----------|-----------|---------|---------|
| lock | `lock(uint128,address,bytes32,bytes4,uint256)` | `0x7bacc91e` | cast sig ✅ |
| approve | `approve(address,uint256)` | `0x095ea7b3` | cast sig ✅ |
