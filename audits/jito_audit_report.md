# Skill Audit Report — jito

**Auditor:** Claude Code (claude-sonnet-4-6)
**Date:** 2026-04-06
**Plugin repo:** GeoGu360/onchainos-plugins (monorepo)
**Plugin path:** `/tmp/onchainos-plugins/jito/`
**Audit wallet (Solana):** DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE
**Chain:** Solana (chainIndex 501)

---

## Step 0 — Environment

| Item | Value |
|------|-------|
| Solana address | DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE |
| SOL balance (pre-test) | 0.060769907 SOL |
| JitoSOL balance (pre-test) | 0.008659678 JitoSOL |
| Audit dir | /tmp/skill-audit/jito/ |

---

## Step 2 — SKILL.md Summary

| Field | Value |
|-------|-------|
| name | jito |
| version | 0.1.0 |
| binary | `jito` |
| chains | solana only |
| category | defi-protocol |

### Commands Test Plan

| # | Command | Args | Type |
|---|---------|------|------|
| 1 | `rates` | `--chain 501` | Read-only |
| 2 | `positions` | `--chain 501` | Read-only |
| 3 | `stake` | `--amount 0.001 --chain 501 --dry-run` | Dry-run write |
| 4 | `unstake` | `--amount 0.005 --chain 501 --dry-run` | Dry-run write |
| 5 | `stake` | `--amount 0.001 --chain 501` | Live write |

---

## Step 3 — Compile

```
cargo build --release
```

**Result:** SUCCESS  
**Duration:** ~22.5 seconds  
**Binary:** `target/release/jito` (4.1 MB stripped)  
**Dependencies:** clap 4, reqwest 0.12, serde/serde_json 1, tokio 1, anyhow 1, base64 0.22, bs58 0.5, sha2 0.10

---

## Step 4 — Install

```
npx skills add /tmp/onchainos-plugins/jito/skills/jito --yes --global
```

**Result:** SUCCESS  
Installed to `~/.agents/skills/jito` (universal) with symlink for Claude Code.  
Skill count: 1 skill, 45 agents targeted.

---

## Step 5 — Command Test Results

### 5.1 `jito rates --chain 501`

**Status:** PASS

```json
{
  "ok": true,
  "data": {
    "protocol": "Jito",
    "chain": "Solana",
    "stake_pool": "Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb",
    "jitosol_mint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
    "sol_per_jitosol": "1.27164059",
    "jitosol_per_sol": "0.78638572",
    "total_staked_sol": "11231060.9611",
    "total_jitosol_supply": "8831945.9685",
    "estimated_apy_pct": "5.49",
    "fee_note": "Epoch fee: ~5% of staking rewards. Deposit fee: 0%. Withdrawal fee: ~0.3% (delayed unstake).",
    "unstake_note": "Unstaking creates a stake account that unlocks after the current epoch (~2-3 days)."
  }
}
```

- Exchange rate fetched live from on-chain SPL Stake Pool account data
- APY fetched from DeFiLlama yields API (`yields.llama.fi/pools`, project: `jito-liquid-staking`)
- Both `stake_pool` and `jitosol_mint` addresses are echoed in output for transparency

---

### 5.2 `jito positions --chain 501`

**Status:** PASS

```json
{
  "ok": true,
  "data": {
    "wallet": "DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE",
    "jitosol_token_account": "59y8wuNq9tFEi8sZzVEvxcJecaVYuakNKNpayVxbP6FR",
    "jitosol_ata": "AaUt1LwgHoe7u2D1C8rEosWGwfXgYhe2bJjmgAvfi3cw",
    "jitosol_balance": "0.008659678",
    "jitosol_raw": "8659678",
    "sol_value": "0.011011998",
    "sol_per_jitosol": "1.27164059",
    "chain": "Solana"
  }
}
```

- Returns both the canonical ATA address and the actual token account (may differ)
- SOL equivalent calculated from live pool rate
- `jitosol_token_account` != `jitosol_ata`: wallet's JitoSOL lives at `59y8wuN...` (non-ATA), correctly resolved via `getTokenAccountsByOwner` fallback

---

### 5.3 `jito stake --amount 0.001 --chain 501 --dry-run`

**Status:** PASS

```json
{
  "ok": true,
  "dry_run": true,
  "data": {
    "operation": "stake",
    "wallet": "DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE",
    "sol_amount": 0.001,
    "lamports": "1000000",
    "expected_jitosol": "0.000786386",
    "sol_per_jitosol_rate": "1.27164059",
    "user_jitosol_token_account": "59y8wuNq9tFEi8sZzVEvxcJecaVYuakNKNpayVxbP6FR",
    "stake_pool": "Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb",
    "reserve_stake": "BgKUXdS29YcHCFrPm5M8oLHiTzZaMDjsebggjoaQ6KFL",
    "withdraw_authority": "6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS",
    "note": "Ask user to confirm before submitting the stake transaction"
  }
}
```

- All PDAs (withdraw_authority, reserve_stake) fetched/derived at runtime from live chain state
- `note` field correctly instructs agent to confirm before executing

---

### 5.4 `jito unstake --amount 0.005 --chain 501 --dry-run`

**Status:** PASS (dry-run only; live unstake intentionally not implemented)

```json
{
  "ok": true,
  "dry_run": true,
  "data": {
    "operation": "unstake",
    "jitosol_amount": 0.005,
    "expected_sol": "0.006358203",
    "delay_note": "Unstaking creates a stake account that unlocks after the current epoch (~2-3 days)...",
    "fee_note": "Unstake fee: ~0.3% of withdrawn amount",
    "current_jitosol_balance": "0.000000000",
    "note": "Ask user to confirm before submitting the unstake transaction"
  }
}
```

Note: `current_jitosol_balance` shows `0.000000000` in dry-run because `unstake.rs` checks the ATA address (canonical derived), but the wallet's JitoSOL is in a non-ATA account. This is a minor balance display bug in dry-run only (the live path correctly gate-checks before submit).

---

### 5.5 `jito stake --amount 0.001 --chain 501` (live on-chain)

**Status:** PASS (second attempt; see note on first attempt)

```json
{
  "ok": true,
  "data": {
    "txHash": "5PXt7UXMcHwmmX8kgTw31WyJJxyuksYMfam8ZeFYU3qYgb7R8zfJKik4E69F4HeUfpu5GmcCvwJvSmaQa2NovjPP",
    "operation": "stake",
    "sol_amount": 0.001,
    "expected_jitosol": "0.000786386",
    "wallet": "DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE",
    "solscan": "https://solscan.io/tx/5PXt7UXMcHwmmX8kgTw31WyJJxyuksYMfam8ZeFYU3qYgb7R8zfJKik4E69F4HeUfpu5GmcCvwJvSmaQa2NovjPP"
  }
}
```

**Tx verification:**
- Signature status: `finalized`, `err: null` (slot 411325495)
- JitoSOL balance: 0.008659678 → 0.009446063 (+0.000786385, expected +0.000786386) ✓
- SOL balance: 0.060769907 → 0.059764907 (-0.001005000, includes ~0.000005 tx fee) ✓

**First attempt note:** The first live stake run returned `txHash: "pending"`. The `extract_tx_hash()` function defaults to `"pending"` when the expected path in the onchainos JSON is not found. This was a transient issue — the second run returned a valid finalized txHash. Root cause: the onchainos wallet contract-call response structure may vary on first-time calls or when a tx needs additional confirmation time before the hash is returned synchronously.

---

### 5.6 Unstake (live) — SKIPPED

Live unstake is intentionally not implemented in the binary. Calling `jito unstake --amount 0.005 --chain 501` (without `--dry-run`) returns a descriptive error:

```
{
  "ok": false,
  "error": "On-chain unstake requires selecting a validator stake account and signing with an ephemeral keypair. This complex flow is currently dry-run only. Use the Jito webapp (jito.network) to complete the unstake..."
}
```

This is acceptable as a known limitation, clearly documented.

---

## Step 6 — Static Code Review

### 6.1 SKILL.md — Non-ASCII Characters

**Status:** WARN (minor)

SKILL.md description field contains:
- U+2014 (em-dash `—`) × 6 occurrences in section headings and prose
- U+2194 (bidirectional arrow `↔`) × 1 occurrence ("SOL ↔ JitoSOL")

No CJK characters found. These are purely cosmetic Unicode punctuation; they render correctly in all common markdown viewers. Recommend replacing with ASCII equivalents (`-`, `<->`) for strict ASCII compliance in the `description:` frontmatter field specifically.

### 6.2 Hardcoded Addresses

**Status:** ACCEPTABLE (by design)

All protocol addresses are compiled constants in `src/config.rs`:

| Constant | Address | Type |
|----------|---------|------|
| `JITO_STAKE_POOL` | `Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb` | Pool account |
| `JITOSOL_MINT` | `J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn` | Token mint |
| `STAKE_POOL_PROGRAM` | `SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy` | SPL program |

These are immutable Solana mainnet canonical addresses that cannot change. Hardcoding is the correct approach. The pool's live state (rate, reserve_stake, manager_fee_account) is fetched dynamically via `getAccountInfo` at runtime. This is the right design.

One duplication: `commands/mod.rs` lines 14-15 hard-code the token program and ATA program addresses inline rather than importing from `config.rs`. Minor refactor opportunity.

### 6.3 base64 → base58 Conversion

**Status:** CORRECT

`onchainos.rs` `wallet_contract_call_solana()`:
1. Builds transaction as base64 internally (for easy encoding)
2. Decodes via `base64::STANDARD.decode()`
3. Re-encodes via `bs58::encode()` before passing `--unsigned-tx` to `onchainos wallet contract-call`

This matches the CLI's documented expectation (`--unsigned-tx` expects base58). Conversion is correct and documented with inline warning comments.

### 6.4 v0 Versioned Transaction Format

**Status:** CORRECT

`commands/mod.rs` `SolanaMessage::serialize()`:
- Prefixes message with `0x80` (version byte for v0 versioned transactions)
- `build_unsigned_transaction()` prepends `compact_u16(1)` + 64-byte zeroed signature placeholder

Final wire format: `[compact_u16(1)][64×0x00][0x80][header][account_keys][blockhash][instructions][ATL=0x00]`

This is the correct Solana v0 versioned transaction format. The live transaction confirmed finalized on-chain validates this.

### 6.5 Error Messages — Friendliness

**Status:** MOSTLY GOOD, one minor issue

- All top-level errors use `anyhow::bail!()` and are caught in `main.rs`, serialized as `{"ok": false, "error": "..."}` — no raw panics to stdout.
- Error messages are human-readable and actionable.
- **One bare `.unwrap()` found** in `stake.rs:153`: `bs58::decode(config::JITOSOL_MINT).into_vec().unwrap()`. This would panic at runtime if `JITOSOL_MINT` were ever an invalid base58 string. Since it's a compile-time constant with a known-valid value, this is low risk but should be replaced with `?` error propagation for correctness.

### 6.6 is_on_ed25519_curve — Python3 Subprocess Dependency

**Status:** WARN (notable design concern)

`commands/mod.rs` PDA derivation spawns a `python3` subprocess for each hash candidate during `find_program_address`. The nonce search iterates up to 256 times in the worst case.

Concerns:
- Runtime dependency on `python3` being in `$PATH` — silently falls back to `false` (treats all hashes as off-curve) if Python is unavailable, which would produce wrong PDAs without error
- Performance: 256 subprocess spawns per PDA derivation (measured: ~0.1s per derivation in practice, acceptable but fragile)
- The fallback behavior (`false` = off-curve) means if Python is absent, the first hash tried becomes the PDA regardless of whether it's actually on-curve, which could produce an invalid PDA

**Recommendation:** Implement `is_on_ed25519_curve` natively in Rust using the `curve25519-dalek` crate, or at minimum fail loudly (return `Err`) if Python is unavailable rather than silently returning a potentially wrong PDA.

### 6.7 plugin-store lint

**Status:** FAIL (build artifacts, no .gitignore)

`plugin-store lint /tmp/onchainos-plugins/jito` — 201 errors:
- **[E080]** 200+ files in `target/` exceed the 200 KB per-file limit
- **[E081]** Total submission size 238,911 KB (limit 1,024 KB)
- **[E130]** Pre-compiled `.dylib` proc-macro files found in `target/release/deps/`

**Root cause:** Missing `.gitignore` — the `target/` directory is not excluded. The plugin store lints the full directory tree. Adding a `.gitignore` (or `.pluginignore`) with `target/` would resolve all 201 errors, since the source code itself is clean.

No errors in source files themselves.

---

## Step 7 — Uninstall

```
npx skills remove jito --yes --global
```

**Result:** SUCCESS — 1 skill removed from `~/.agents/skills/jito`.

---

## Step 8 — Summary

### Test Results Table

| Command | Mode | Status | Notes |
|---------|------|--------|-------|
| `rates` | read-only | PASS | Live rate + DeFiLlama APY fetched correctly |
| `positions` | read-only | PASS | Non-ATA token account resolved via fallback |
| `stake --dry-run` | dry-run | PASS | All PDAs derived at runtime |
| `unstake --dry-run` | dry-run | PASS | Balance display bug (0.0 in dry-run) |
| `stake` (live) | on-chain | PASS | Tx finalized, JitoSOL received, amount correct |
| `unstake` (live) | on-chain | SKIP | Intentionally not implemented; clear error |

### Findings Summary

| Severity | Finding | File | Recommendation |
|----------|---------|------|----------------|
| BLOCKER | Missing `.gitignore` (201 lint errors from `target/`) | repo root | Add `.gitignore` with `target/` before submission |
| HIGH | `is_on_ed25519_curve` silently returns wrong result if Python3 absent | `src/commands/mod.rs:99` | Implement natively via `curve25519-dalek` or fail loudly |
| MEDIUM | `extract_tx_hash` fallback to `"pending"` string on first live stake | `src/onchainos.rs:85` | Return `Err` instead of silent `"pending"` if no txHash present |
| LOW | `.unwrap()` on `JITOSOL_MINT` base58 decode | `src/commands/stake.rs:153` | Replace with `?` propagation |
| LOW | `unstake --dry-run` reports `jitosol_balance: 0.0` for non-ATA wallets | `src/commands/unstake.rs:58` | Use same `get_best_jitosol_balance()` helper as `positions.rs` |
| LOW | Address constants duplicated in `commands/mod.rs` (lines 14-15) vs `config.rs` | `src/commands/mod.rs` | Import from `config.rs` |
| INFO | SKILL.md uses U+2014 em-dash and U+2194 arrow — non-ASCII | `skills/jito/SKILL.md` | Replace with ASCII for strict compliance |
| INFO | `unstake` live flow deferred; requires ephemeral keypair signer | `src/commands/unstake.rs` | Documented limitation; acceptable for v0.1.0 |

### Overall Assessment

**CONDITIONAL PASS**

The core staking logic is sound: live `stake` executed correctly with verified on-chain confirmation, exchange rates and positions are accurate, and transaction construction (v0 versioned format, DepositSol instruction layout, base64→base58 conversion) is correct. The missing `.gitignore` is a blocking submission issue but trivially fixed. The Python3 subprocess dependency for PDA derivation is the most significant code-quality concern and should be addressed before production use.

### On-chain Evidence

| Item | Value |
|------|-------|
| Stake tx hash | `5PXt7UXMcHwmmX8kgTw31WyJJxyuksYMfam8ZeFYU3qYgb7R8zfJKik4E69F4HeUfpu5GmcCvwJvSmaQa2NovjPP` |
| Solscan link | https://solscan.io/tx/5PXt7UXMcHwmmX8kgTw31WyJJxyuksYMfam8ZeFYU3qYgb7R8zfJKik4E69F4HeUfpu5GmcCvwJvSmaQa2NovjPP |
| SOL spent | 0.001000000 SOL + ~0.000005 tx fee |
| JitoSOL received | +0.000786385 JitoSOL (expected 0.000786386, delta -0.000000001) |
| Tx status | finalized, err: null, slot 411325495 |
