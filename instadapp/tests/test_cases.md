# Test Cases — Instadapp Plugin

## Level 1: Compile + Lint

| # | Test | Command | Expected |
|---|------|---------|---------|
| 1 | Build release | `cargo build --release` | Compiles with 0 errors |
| 2 | Lint | `cargo clean && plugin-store lint .` | 0 errors (after source_commit set) |

## Level 2: Read Tests (no wallet, no gas)

| # | Scenario | Command | Expected |
|---|----------|---------|---------|
| 2.1 | List vaults | `instadapp --chain 1 vaults` | 2 vaults returned, exchange_price_eth > 1.0 |
| 2.2 | Get rates | `instadapp --chain 1 rates` | Both vaults with cumulative_yield_pct > 0 |
| 2.3 | Check positions (no position) | `instadapp --chain 1 positions --wallet 0x87fb...` | position_count: 0 |

## Level 3: Dry-run Tests (calldata verification)

| # | Scenario | Command | Expected |
|---|----------|---------|---------|
| 3.1 | Dry-run deposit v1 | `instadapp --chain 1 --dry-run deposit --vault v1 --amount 0.00005` | selector: 0x87ee9312, amount_wei: 50000000000000 |
| 3.2 | Dry-run deposit v2 | `instadapp --chain 1 --dry-run deposit --vault v2 --amount 0.001` | steps[0].selector: 0x095ea7b3, steps[1].selector: 0x6e553f65 |
| 3.3 | Dry-run withdraw v1 | `instadapp --chain 1 --dry-run withdraw --vault v1 --shares 0.001` | selector: 0x00f714ce |
| 3.4 | Dry-run withdraw v2 | `instadapp --chain 1 --dry-run withdraw --vault v2 --shares 0.001` | selector: 0xba087652 |

## Level 4: On-chain Tests (requires lock)

| # | Scenario | Command | Result |
|---|----------|---------|--------|
| 4.1 | Deposit 0.00005 ETH into iETH v1 | `instadapp --chain 1 deposit --vault v1 --amount 0.00005` | txHash: 0xe972a01b... |
| 4.2 | Check positions after deposit | `instadapp --chain 1 positions` | position_count: 1, shares > 0 |
| 4.3 | Withdraw all iETH shares from v1 | `instadapp --chain 1 withdraw --vault v1` | txHash: 0xc45a409a... |
