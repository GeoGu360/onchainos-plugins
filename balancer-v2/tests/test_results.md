# Test Results Report — Balancer V2 Plugin

- **Date:** 2026-04-05
- **DApp supported chains:** EVM (Arbitrum 42161, Ethereum 1)
- **EVM test chain:** Arbitrum (42161)
- **Binary:** `balancer-v2 v0.1.0`
- **Compile:** ✅ PASS
- **Lint:** ✅ PASS (manual — plugin-store CLI not installed in environment)
- **Overall pass standard:** EVM DApp → EVM operations all pass ✅

---

## Summary

| Total | L1 Build | L2 Read | L3 Simulate | L4 On-chain | Failed | Blocked |
|-------|----------|---------|-------------|-------------|--------|---------|
| 11    | 2        | 5       | 3           | 1           | 0      | 0       |

---

## Detailed Results

| # | Scenario (User View) | Level | Command | Result | TxHash / Calldata | Notes |
|---|---------------------|-------|---------|--------|-------------------|-------|
| 1 | Build plugin debug | L1 | `cargo build` | ✅ PASS | — | 1 warning: unused `serialize_u128_as_string` (harmless) |
| 2 | Build plugin release | L1 | `cargo build --release` | ✅ PASS | — | Clean release binary |
| 3 | Get WBTC/WETH/USDC.e pool info | L2 | `pool-info --pool 0x64541216...0002 --chain 42161` | ✅ PASS | — | 3 tokens, fee=0.25%, total_supply=512 BPT, weights=33.33% each |
| 4 | Get DAI/USDT/USDC.e stable pool info | L2 | `pool-info --pool 0x1533...0016 --chain 42161` | ✅ PASS | — | GENERAL specialization, fee=0.03%, no weights (stable pool) |
| 5 | List top 5 Arbitrum pools | L2 | `pools --chain 42161 --limit 5` | ✅ PASS | — | 5 pools returned including waArbUSDCn/waArbUSDT/waArbGHO stable pool (TVL $6.8M) |
| 6 | Quote 0.001 WETH → USDC.e | L2 | `quote --from WETH --to USDC --amount 0.001 --pool 0x6454...0002 --chain 42161` | ✅ PASS | amount_out: 2031120 (2.03 USDC.e) | querySwap via BalancerQueries on-chain |
| 7 | Quote 1.0 USDT → USDC.e (stable) | L2 | `quote --from 0xfd0... --to 0xff9... --amount 1.0 --pool 0x1533...0016 --chain 42161` | ✅ PASS | amount_out: 999493 (0.999 USDC.e) | ~1:1 stable swap as expected |
| 8 | Simulate WETH→USDC swap | L3 | `swap --from WETH --to USDC --amount 0.001 --dry-run --chain 42161` | ✅ PASS | calldata selector: `0x52bbbe29` (swap) | dry_run: true, min_amount_out: 2020964 |
| 9 | Simulate join pool | L3 | `join --pool 0x6454... --amounts "0,0,1.0" --dry-run --chain 42161` | ✅ PASS | calldata selector: `0xb95cac28` (joinPool) | dry_run: true, 3 token amounts |
| 10 | Simulate exit pool | L3 | `exit --pool 0x6454... --bpt-amount 0.001 --dry-run --chain 42161` | ✅ PASS | calldata selector: `0x8bdb3913` (exitPool) | dry_run: true, bpt_amount_in: 1000000000000000 |
| 11 | Swap 0.01 USDT → USDC.e on Arbitrum | L4 | `swap --from 0xfd0... --to 0xff9... --amount 0.01 --pool 0x1533...0016 --chain 42161` | ✅ PASS | Approve: `0xf4057c6587c8372cb5165878ed0e0a8030ff22c1c3f56c6619be377b2b47a7fd` Swap: `0x3ee6a2fe5a035bf490d80aa94a0af9c1c9a695aea52c8703479f5559b1b8041b` | [Verified on Arbiscan](https://arbiscan.io/tx/0x3ee6a2fe5a035bf490d80aa94a0af9c1c9a695aea52c8703479f5559b1b8041b): block 449238437, status SUCCESS, 177,979 gas, received 0.009994 USDC.e |

---

## Bug Fixes During Testing

| # | Issue | Root Cause | Fix | File |
|---|-------|-----------|-----|------|
| 1 | `querySwap` reverts on all calls | ABI encoding bug: `bytes userData` offset within SingleSwap tuple was `5*32=0xa0` but must be `6*32=0xc0` | Changed `single_swap_bytes_offset` calculation | `src/rpc.rs` |
| 2 | `pools` command fails | Balancer Subgraph TheGraph URL not accessible in sandbox | Switched to `api-v3.balancer.fi/graphql` Balancer V3 API with `poolGetPools` query | `src/commands/pools.rs`, `src/config.rs` |
| 3 | `wallet balance --output json` flag not supported | EVM chain wallet balance does not accept `--output json` flag (returns JSON natively) | Removed `--output json` from wallet balance call | `src/onchainos.rs` |

---

## Funds Spent

| Operation | Amount | Token | TxHash |
|-----------|--------|-------|--------|
| ERC-20 approve | 0 | USDT | 0xf4057c65... (gas only) |
| Swap USDT→USDC.e | 0.01 | USDT | 0x3ee6a2fe... |

**ETH spent:** ~0.000005 ETH gas (two transactions)
**USDT spent:** 0.01 USDT
**USDC.e received:** 0.009994 USDC.e
