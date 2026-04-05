# Test Cases — StakeStone

## L1 — Compile + Lint

| # | Test | Command | Expected |
|---|------|---------|----------|
| 1 | Build release | `cargo build --release` | Exits 0, binary created |
| 2 | Lint | `cargo clean && plugin-store lint .` | 0 errors (E123 OK with placeholder SHA) |

## L2 — Read Tests (no wallet, no gas)

| # | Scenario | Command | Expected |
|---|---------|---------|---------|
| 3 | Get current STONE rate | `stakestone get-rate` | STONE price ~1.063 ETH, round >274 |
| 4 | Get position for zero-balance wallet | `stakestone get-position --address 0x87fb0647...` | 0 STONE, no pending withdrawal |

## L3 — Dry-Run Simulation

| # | Scenario | Command | Expected Calldata |
|---|---------|---------|------------------|
| 5 | Stake dry-run | `stakestone stake --amount 0.00005 --from <ADDR> --dry-run` | `0xd0e30db0` |
| 6 | Request-withdraw dry-run | `stakestone request-withdraw --amount 0.001 --from <ADDR> --dry-run` | `0x745400c9` + 32-byte shares |
| 7 | Cancel-withdraw dry-run | `stakestone cancel-withdraw --amount 0.001 --from <ADDR> --dry-run` | `0x9f01f7ba` + 32-byte shares |

## L4 — On-Chain (requires lock, uses real gas)

| # | Scenario | Command | Limit |
|---|---------|---------|-------|
| 8 | Stake 0.00005 ETH | `stakestone stake --amount 0.00005 --from <WALLET>` | 0.00005 ETH |
