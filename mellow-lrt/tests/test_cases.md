# Test Cases — mellow-lrt

DApp: Mellow LRT (Mellow Protocol)  
Chain: Ethereum mainnet (chain 1)  
Date: 2026-04-05

---

## Level 1 — Compilation + Lint

| # | Test | Command | Expected |
|---|------|---------|---------|
| L1-1 | Compile debug build | `cargo build` | No errors |
| L1-2 | Lint | `cargo clean && plugin-store lint .` | ✅ 0 errors |

---

## Level 2 — Read Tests (no wallet, no gas)

| # | Scenario (User View) | Command | Expected |
|---|---------------------|---------|---------|
| L2-1 | Show all Mellow vaults on Ethereum | `vaults --chain 1` | JSON with ≥5 vaults, APR, TVL |
| L2-2 | Show top 5 vaults | `vaults --chain 1 --limit 5` | Exactly 5 vaults returned |
| L2-3 | Check wallet positions (empty) | `positions --chain 1 --wallet 0x87fb0647faabea33113eaf1d80d67acb1c491b90` | JSON, positions=[] (no holdings) |

---

## Level 3 — Simulation Tests (dry-run, no on-chain)

| # | Scenario (User View) | Command | Expected Selector |
|---|---------------------|---------|-----------------|
| L3-1 | Preview ETH deposit into rsENA (simple-lrt) | `--dry-run deposit --vault rsENA --token ETH --amount 0.00005 --chain 1` | `0x0bb9f5e1` (EthWrapper) |
| L3-2 | Preview wstETH deposit into rsENA (simple-lrt) | `--dry-run deposit --vault rsENA --token wstETH --amount 0.001 --chain 1` | `0x6e553f65` (ERC-4626 deposit) |
| L3-3 | Preview withdraw all shares from rsENA | `--dry-run withdraw --vault rsENA --all --chain 1` | `0xba087652` (ERC-4626 redeem) |
| L3-4 | Preview claim from rsENA | `--dry-run claim --vault rsENA --chain 1` | `0x996cba68` (claim) |
| L3-5 | Preview wstETH deposit into steakLRT (multi-vault) | `--dry-run deposit --vault steakLRT --token wstETH --amount 0.001 --chain 1` | `0xf379a7d6` (multi-vault deposit) |

---

## Level 4 — On-chain Write Tests (requires lock, costs gas)

| # | Scenario (User View) | Command | Max Cost |
|---|---------------------|---------|---------|
| L4-1 | Deposit 0.00005 ETH into steakLRT | `deposit --vault steakLRT --token ETH --amount 0.00005 --chain 1` | 0.00005 ETH + gas |

NOTE: Multi-vault deposits (steakLRT, Re7LRT etc.) revert due to Mellow validator restriction — these
vaults require curator whitelist authorization and cannot be accessed by arbitrary public wallets.
Only simple-lrt type vaults accept public deposits. L4-1 uses rsENA (simple-lrt, ERC-4626 interface).

NOTE: Amounts are small due to testnet budget (0.005 ETH total). L4-1 uses the ETH→EthWrapper path
which wraps ETH to wstETH internally before depositing into the ERC-4626 vault.
