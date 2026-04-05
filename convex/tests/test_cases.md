# Test Cases — Convex Finance Plugin

## L1: Compilation + Lint

| ID | Test | Command | Expected |
|----|------|---------|---------|
| L1-1 | Build | `cargo build` | Compiles with 0 errors |
| L1-2 | Release build | `cargo build --release` | Produces binary |
| L1-3 | Lint | `cargo clean && plugin-store lint .` | 1 error (placeholder SHA only) |

## L2: Read Tests (no wallet needed)

| ID | Scenario | Command | Expected |
|----|---------|---------|---------|
| L2-1 | List top Convex pools | `convex get-pools --limit 5` | Returns pool list with TVL |
| L2-2 | Get factory pools | `convex get-pools --registry factory --limit 3` | Factory pool list |
| L2-3 | Query positions (explicit address) | `convex get-positions --address 0x87fb... --chain 1` | Position data with balances |

## L3: Dry-Run / Simulate

| ID | Scenario | Command | Expected |
|----|---------|---------|---------|
| L3-1 | Stake cvxCRV dry-run | `convex --dry-run stake-cvxcrv --amount 1.0 --chain 1` | approve+stake calldata, selectors 0x095ea7b3 + 0xa694fc3a |
| L3-2 | Unstake cvxCRV dry-run | `convex --dry-run unstake-cvxcrv --amount 1.0 --chain 1` | withdraw calldata, selector 0x00ebf5dd |
| L3-3 | Lock CVX dry-run | `convex --dry-run lock-cvx --amount 1.0 --chain 1` | approve+lock calldata, selector 0x1338736f |
| L3-4 | Unlock CVX dry-run | `convex --dry-run unlock-cvx --chain 1` | processExpiredLocks calldata, selector 0x312ff839 |
| L3-5 | Claim rewards dry-run | `convex --dry-run claim-rewards --chain 1` | getReward calldata selectors 0x7050ccd9 + 0x3d18b912 |

## L4: On-Chain Write (requires lock)

| ID | Scenario | Command | Expected |
|----|---------|---------|---------|
| L4-1 | Claim rewards (no positions) | `convex claim-rewards --chain 1` | Graceful skip, wallet resolved |
| L4-2 | Stake cvxCRV | `convex stake-cvxcrv --amount 0.001 --chain 1` | BLOCKED — test wallet has 0 cvxCRV |
| L4-3 | Lock CVX | `convex lock-cvx --amount 0.001 --chain 1` | BLOCKED — test wallet has 0 CVX |
