# Allbridge Classic — Test Cases

## Level 1: Compile + Lint

| # | Test | Command | Expected |
|---|------|---------|---------|
| L1-1 | Compile | `cargo build --release` | 0 errors |
| L1-2 | Lint | `cargo clean && plugin-store lint .` | 0 errors |

## Level 2: Read Tests (no wallet, no gas)

| # | Scenario (user view) | Command | Expected |
|---|---------------------|---------|---------|
| L2-1 | "Show me what tokens I can bridge" | `get-tokens` | JSON with ETH/BSC/POL/SOL chains and tokens |
| L2-2 | "Is this Solana address valid for bridging?" | `check-address --chain SOL --address DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE` | Valid address response |
| L2-3 | "Is this ETH address valid for bridging?" | `check-address --chain ETH --address 0x87fb0647faabea33113eaf1d80d67acb1c491b90` | Valid address response |
| L2-4 | "Check tx status for a known lock ID" | `get-tx-status --lock-id <known_id>` | Should return 404-style error or confirmed |

## Level 3: Dry-run / Simulate Tests

| # | Scenario (user view) | Command | Expected calldata prefix |
|---|---------------------|---------|---------|
| L3-1 | "Dry-run bridge 0.01 USDT from ETH to BSC" | `bridge --chain 1 --token USDT --amount 0.01 --dest-chain BSC --recipient 0x87fb0647faabea33113eaf1d80d67acb1c491b90 --dry-run` | lockCalldata starts with 0x7bacc91e |
| L3-2 | "Dry-run bridge 0.01 USDT from ETH to SOL" | `bridge --chain 1 --token USDT --amount 0.01 --dest-chain SOL --recipient DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE --dry-run` | lockCalldata starts with 0x7bacc91e, dry_run:true |
| L3-3 | "Dry-run bridge 0.01 USDC from BSC to POL" | `bridge --chain 56 --token USDC --amount 0.01 --dest-chain POL --recipient 0x87fb0647faabea33113eaf1d80d67acb1c491b90 --dry-run` | lockCalldata correct |

## Level 4: On-chain Tests (requires lock + gas)

Per GUARDRAILS: bridge tests use minimum amount (0.01 USDT).
Due to GUARDRAILS bridge policy: actual cross-chain bridge tx is expensive (lock + wait + unlock).
L4 is restricted to read ops and dry-run only for this DApp.

| # | Scenario (user view) | Command | Expected |
|---|---------------------|---------|---------|
| L4-1 | "Get token list live" | `get-tokens` | Real API response with tokens |
| L4-2 | "Check address live" | `check-address --chain SOL --address ...` | Real API response |

Note: Actual bridge L4 (lock on-chain) is skipped per pipeline instructions: "actual cross-chain bridge tx is expensive so dry-run only"
