# Segment Finance — Test Cases

## Level 1: Compile + Lint
| # | Test | Command | Expected |
|---|------|---------|---------|
| L1-1 | Compile | `cargo build --release` | 0 errors |
| L1-2 | Lint | `cargo clean && plugin-store lint .` | 0 errors |

## Level 2: Read Tests (no wallet, no gas)

| # | Scenario | Command | Expected |
|---|----------|---------|---------|
| L2-1 | List Segment Finance markets on BSC | `get-markets --chain 56` | ok=true, market_count>=5, supply_apy_pct set |
| L2-2 | Get positions (no wallet specified) | `get-positions --chain 56 --wallet 0x87fb0647faabea33113eaf1d80d67acb1c491b90` | ok=true, positions array, health_status |
| L2-3 | Invalid chain | `get-markets --chain 1` | ok=false, error message |
| L2-4 | Invalid asset | `supply --asset XYZ --amount 1 --chain 56 --dry-run` | ok=false, unsupported asset |

## Level 3: Dry-Run / Simulate Tests

| # | Scenario | Command | Expected Calldata |
|---|----------|---------|-----------------|
| L3-1 | Dry-run supply USDT | `supply --asset USDT --amount 0.01 --chain 56 --dry-run` | calldata starts with 0xa0712d68 |
| L3-2 | Dry-run supply BNB | `supply --asset BNB --amount 0.001 --chain 56 --dry-run` | calldata = 0x1249c58b |
| L3-3 | Dry-run withdraw USDT | `withdraw --asset USDT --amount 0.01 --chain 56 --dry-run` | calldata starts with 0x852a12e3 |
| L3-4 | Dry-run borrow USDT | `borrow --asset USDT --amount 0.01 --chain 56 --dry-run` | calldata starts with 0xc5ebeaec |
| L3-5 | Dry-run repay USDT | `repay --asset USDT --amount 0.01 --chain 56 --dry-run` | calldata starts with 0x0e752702 |
| L3-6 | Dry-run enter-market USDT | `enter-market --asset USDT --chain 56 --dry-run` | calldata starts with 0xc2998238 |

## Level 4: On-chain Write Tests

L4 SKIPPED — wallet has 0 BNB and 0 tokens on BSC (chain 56). Test wallet only has funds on ETH mainnet and Base (chain 1, 8453).

Per GUARDRAILS rule 8: "If wallet has no funds on target chain, mark L4 as SKIPPED."
