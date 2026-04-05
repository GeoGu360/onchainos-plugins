# Balancer V2 Test Cases

## Level 1 — Compile + Lint

| # | Test | Command | Expected |
|---|------|---------|---------|
| L1-1 | Build debug | `cargo build` | 0 errors, 1 warning (unused fn) |
| L1-2 | Build release | `cargo build --release` | 0 errors |
| L1-3 | Lint: E002 api_calls | Manual check plugin.yaml | Pure string list |
| L1-4 | Lint: E106 confirmations | Manual check SKILL.md | "Ask user to confirm" near every wallet contract-call |
| L1-5 | Lint: E080 gitignore | Check .gitignore | Contains `/target/` |

## Level 2 — Read Tests

| # | Scenario | Command | Expected |
|---|---------|---------|---------|
| L2-1 | Get pool info for WBTC/WETH/USDC.e | `pool-info --pool 0x6454...0002 --chain 42161` | JSON with 3 tokens, swap_fee, total_supply |
| L2-2 | Get pool info for stable pool | `pool-info --pool 0x1533...0016 --chain 42161` | JSON with 3 tokens, no weights |
| L2-3 | List top Arbitrum pools | `pools --chain 42161 --limit 5` | JSON array with 5 pools |
| L2-4 | Quote WETH → USDC.e | `quote --from WETH --to USDC --amount 0.001 --pool ... --chain 42161` | amountOut > 0 |
| L2-5 | Quote USDT → USDC.e (stable) | `quote --from 0xfd0... --to 0xff9... --amount 1.0 --pool 0x1533... --chain 42161` | ~0.999 USDC.e |

## Level 3 — Simulate Tests (Dry-run)

| # | Scenario | Command | Expected |
|---|---------|---------|---------|
| L3-1 | Swap dry-run | `swap --from WETH --to USDC --amount 0.001 --pool ... --dry-run --chain 42161` | dry_run: true, tx_hash: 0x000..., selector 0x52bbbe29 |
| L3-2 | Join dry-run | `join --pool ... --amounts "0,0,1.0" --dry-run --chain 42161` | dry_run: true, 3 tokens |
| L3-3 | Exit dry-run | `exit --pool ... --bpt-amount 0.001 --dry-run --chain 42161` | dry_run: true, selector 0x8bdb3913 |

## Level 4 — On-chain Tests

| # | Scenario | Command | Expected |
|---|---------|---------|---------|
| L4-1 | Swap 0.01 USDT → USDC.e via stable pool | `swap --from 0xfd0... --to 0xff9... --amount 0.01 --pool 0x1533... --chain 42161` | txHash broadcast, verified on Arbiscan |
