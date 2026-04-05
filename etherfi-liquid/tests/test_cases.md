# Test Cases — etherfi-liquid

## Level 1: Compile + Lint

| # | Test | Command | Expected |
|---|------|---------|----------|
| 1 | cargo build | `cargo build` | Exit 0, no errors |
| 2 | cargo build --release | `cargo build --release` | Exit 0, no errors |
| 3 | plugin-store lint | `cargo clean && plugin-store lint .` | 0 errors |

## Level 2: Read Operations

| # | Scenario | Command | Expected |
|---|----------|---------|----------|
| 4 | List all vaults with APY | `etherfi-liquid --chain 1 vaults` | JSON with 3 vaults, APY > 0, TVL > 0 |
| 5 | Show current exchange rates | `etherfi-liquid --chain 1 rates` | JSON with 3 rates, all > 0 |
| 6 | Show positions (no holdings) | `etherfi-liquid --chain 1 positions --wallet 0x87fb...` | JSON with 3 vaults, all shares=0 |

## Level 3: Dry-run / Simulate

| # | Scenario | Command | Expected |
|---|----------|---------|----------|
| 7 | Deposit dry-run (weETH) | `etherfi-liquid --chain 1 --dry-run deposit --vault LIQUIDETH --token weETH --amount 0.00005` | dry_run=true, approve_calldata starts 0x095ea7b3, deposit_calldata starts 0x8b6099db |
| 8 | Withdraw dry-run (shares) | `etherfi-liquid --chain 1 --dry-run withdraw --vault LIQUIDETH --shares 0.00005` | dry_run=true, calldata starts 0x8432f02b |
| 9 | Deposit dry-run LIQUIDUSD | `etherfi-liquid --chain 1 --dry-run deposit --vault LIQUIDUSD --token USDC --amount 0.01` | dry_run=true, deposit_token_addr=USDC |
| 10 | Invalid vault symbol | `etherfi-liquid --chain 1 --dry-run deposit --vault INVALID --amount 0.01` | Error: Unknown vault symbol |

## Level 4: On-chain Write Operations

| # | Scenario | Command | Expected |
|---|----------|---------|----------|
| 11 | Approve weETH to Teller (L4) | Part of deposit flow | Approve tx confirmed on-chain ✅ |
| 12 | Deposit into LIQUIDETH (L4) | `etherfi-liquid --chain 1 deposit --vault LIQUIDETH --token weETH --amount 0.00004` | SKIPPED — Teller uses requiresAuth (permissioned vault) |
| 13 | Withdraw from LIQUIDETH (L4) | `etherfi-liquid --chain 1 withdraw --vault LIQUIDETH --all` | SKIPPED — No shares to withdraw (deposit failed) |
