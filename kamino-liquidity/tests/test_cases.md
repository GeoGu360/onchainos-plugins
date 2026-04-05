# Test Cases — kamino-liquidity

DApp: Kamino Liquidity (KVault)
Chain: Solana (501)
Date: 2026-04-05

## Level 1 — Compilation + Lint

| # | Test | Command | Expected |
|---|------|---------|---------|
| 1.1 | Compile debug | `cargo build` | ✅ 0 errors |
| 1.2 | Compile release | `cargo build --release` | ✅ 0 errors |
| 1.3 | Lint | `cargo clean && plugin-store lint .` | ✅ 0 errors |

---

## Level 2 — Read Tests (no wallet, no gas)

| # | Scenario (user view) | Command | Expected |
|---|---------------------|---------|---------|
| 2.1 | List all Kamino vaults | `./target/release/kamino-liquidity vaults --chain 501` | JSON with 100+ vaults |
| 2.2 | List SOL vaults | `./target/release/kamino-liquidity vaults --chain 501 --token SOL` | Vaults with SOL in name |
| 2.3 | List vaults with limit | `./target/release/kamino-liquidity vaults --chain 501 --limit 5` | 5 vaults returned |
| 2.4 | Check my positions (empty) | `./target/release/kamino-liquidity positions --chain 501 --wallet DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE` | JSON with positions array |

---

## Level 3 — Simulate (dry-run)

| # | Scenario (user view) | Command | Expected |
|---|---------------------|---------|---------|
| 3.1 | Preview deposit to SOL vault | `./target/release/kamino-liquidity deposit --vault GEodMsAREMV4JdKs1yUCTKpz4EtzxKoSDeM3NZkG1RRk --amount 0.001 --chain 501 --dry-run` | `{ok:true, dry_run:true, serialized_tx: <non-empty>}` |
| 3.2 | Preview withdraw from SOL vault | `./target/release/kamino-liquidity withdraw --vault GEodMsAREMV4JdKs1yUCTKpz4EtzxKoSDeM3NZkG1RRk --amount 1 --chain 501 --dry-run` | `{ok:true, dry_run:true, serialized_tx: <non-empty>}` |

---

## Level 4 — On-chain Tests (require lock, spend SOL)

SOL limit per tx: 0.001 SOL
SOL hard reserve: 0.002 SOL
KVault for testing: GEodMsAREMV4JdKs1yUCTKpz4EtzxKoSDeM3NZkG1RRk (AL-SOL-aut-t, SOL vault)

| # | Scenario (user view) | Command | Expected |
|---|---------------------|---------|---------|
| 4.1 | Deposit 0.001 SOL into Kamino SOL vault | `./target/release/kamino-liquidity deposit --vault GEodMsAREMV4JdKs1yUCTKpz4EtzxKoSDeM3NZkG1RRk --amount 0.001 --chain 501` | `{ok:true, data:{txHash: <non-pending>}}` + solscan.io link |
| 4.2 | Verify position appears after deposit | `./target/release/kamino-liquidity positions --chain 501 --wallet DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE` | shows vault GEodMs... with shares > 0 |
| 4.3 | Withdraw partial shares | `./target/release/kamino-liquidity withdraw --vault GEodMsAREMV4JdKs1yUCTKpz4EtzxKoSDeM3NZkG1RRk --amount <shares_from_4.2> --chain 501` | `{ok:true, data:{txHash: <non-pending>}}` |
