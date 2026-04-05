# Kamino Lend — Test Cases

| # | Scenario (User View) | Level | Command | Expected Result | Priority |
|---|---------------------|-------|---------|-----------------|----------|
| 1 | View Kamino markets and APYs | L2 | `markets --name main` | JSON with USDC/SOL APY, TVL | P0 |
| 2 | Check my lending positions | L2 | `positions` | JSON with obligations (or empty) | P0 |
| 3 | Check positions for specific wallet | L2 | `positions --wallet DTEq...` | JSON response | P0 |
| 4 | Simulate supplying USDC (no tx) | L3 | `supply --token USDC --amount 0.01 --dry-run` | `{"ok":true,"dry_run":true}` | P0 |
| 5 | Simulate withdrawing USDC (no tx) | L3 | `withdraw --token USDC --amount 0.01 --dry-run` | `{"ok":true,"dry_run":true}` | P0 |
| 6 | Preview borrowing SOL | L3 | `borrow --token SOL --amount 0.001 --dry-run` | `{"ok":true,"dry_run":true}` | P0 |
| 7 | Preview repaying SOL | L3 | `repay --token SOL --amount 0.001 --dry-run` | `{"ok":true,"dry_run":true}` | P0 |
| 8 | Supply SOL on-chain | L4 | `supply --token SOL --amount 0.001` | txHash non-empty, confirmed on-chain | P0 |
| 9 | Withdraw SOL on-chain | L4 | `withdraw --token SOL --amount 0.001` | txHash non-empty, confirmed on-chain | P0 |
| 10 | Invalid token symbol | L1-error | `supply --token INVALID --amount 0.01 --dry-run` | Error: Unknown token | P1 |
