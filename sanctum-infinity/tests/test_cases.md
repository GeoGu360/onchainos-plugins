# Test Cases — Sanctum Infinity

## L1: Compilation + Lint

| # | Test | Command | Expected |
|---|------|---------|---------|
| L1-1 | Compile debug build | `cargo build` | ✅ 0 errors |
| L1-2 | Plugin lint | `cargo clean && plugin-store lint .` | ✅ 0 errors |

## L2: Read Tests (No wallet, no gas)

| # | Scenario (user view) | Command | Expected |
|---|---------------------|---------|---------|
| L2-1 | Query Infinity pool stats | `pools` | JSON with nav_sol_per_inf, apy, total_tvl_sol |
| L2-2 | Get swap quote jitoSOL→INF | `quote --from jitoSOL --to INF --amount 0.005` | JSON with in/out amounts |
| L2-3 | Get swap quote mSOL→jitoSOL | `quote --from mSOL --to jitoSOL --amount 0.01` | JSON with fees |
| L2-4 | View positions | `positions` | JSON with INF balance |

## L3: Dry-run Simulation

| # | Scenario (user view) | Command | Expected |
|---|---------------------|---------|---------|
| L3-1 | Dry-run swap jitoSOL→INF | `swap --from jitoSOL --to INF --amount 0.001 --dry-run` | dry_run:true, txHash:"" |
| L3-2 | Dry-run deposit jitoSOL | `deposit --lst jitoSOL --amount 0.005 --dry-run` | dry_run:true, txHash:"" |
| L3-3 | Dry-run withdraw INF→jitoSOL | `withdraw --lst jitoSOL --amount 0.001 --dry-run` | dry_run:true, txHash:"" |

## L4: On-chain Write Tests (needs lock, spend gas)

| # | Scenario (user view) | Command | Expected |
|---|---------------------|---------|---------|
| L4-1 | Swap 0.001 jitoSOL to INF | `swap --from jitoSOL --to INF --amount 0.001` | txHash on Solana |
| L4-2 | Deposit 0.001 jitoSOL to INF pool | `deposit --lst jitoSOL --amount 0.001` | txHash on Solana |
