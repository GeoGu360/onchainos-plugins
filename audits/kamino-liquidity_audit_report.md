# Audit Report: kamino-liquidity

**Date:** 2026-04-06
**Auditor:** skill-auditor (Claude Sonnet 4.6)
**Plugin path:** /tmp/onchainos-plugins/kamino-liquidity
**Monorepo:** GeoGu360/onchainos-plugins
**Commit after fixes:** 8f75b84

---

## Test Wallet

| Chain  | Address                                      |
|--------|----------------------------------------------|
| Solana | `DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE` |

---

## Step 3: Build

| Result | Details |
|--------|---------|
| PASS | `cargo build --release` succeeded in ~96s. One warning: unused constant `SOLANA_CHAIN_ID` (fixed). Post-fix rebuild: 1.54s, zero warnings. |

---

## Step 4: Install

| Result | Details |
|--------|---------|
| PASS | `npx skills add` installed to `~/.agents/skills/kamino-liquidity` (45 agents, symlinked to Claude Code). |

---

## Step 5: Command Test Results

| # | Command | Type | Status | Tx Hash | Slot | Err | Notes |
|---|---------|------|--------|---------|------|-----|-------|
| 1 | `vaults --limit 5` | Read | PASS | — | — | — | Returned 115 total vaults, 5 shown; data structure correct |
| 2 | `positions --wallet DTEqFX...` | Read | PASS | — | — | — | Returned 1 position: vault GEodMs..., 0.454958 shares |
| 3 | `deposit --vault GEodMs... --amount 0.001 --dry-run` | Read | PASS | — | — | — | API accepted params, returned base64 serialized tx |
| 4 | `withdraw --vault GEodMs... --amount 0.1 --dry-run` | Read | PASS | — | — | — | API accepted params, returned base64 serialized tx |
| 5 | `deposit --vault GEodMs... --amount 0.001` | Write | PASS | `57McdLLt38FAqMsRPMBwY1f2WWfxSime7UP7Rht15XhW9aEEGMxbAFEsNPFLBQSe6RwLeE4E1XisSZY9Vx61StYE` | 411347499 | null | Confirmed on-chain via Solana mainnet RPC |
| 6 | `withdraw --vault GEodMs... --amount 0.1` | Write | PASS | `25D3xuR5FjF6YqZTBWwvARdtD9xLJjpBjMNeCZdzhMztXw4RgmYiU4J6o9HBMK8SPsFN99ZU44WZXzxc9HFbT1cd` | 411348083 | null | Confirmed on-chain via Solana mainnet RPC |

All 6 commands passed. Vault used for write tests: `GEodMsAREMV4JdKs1yUCTKpz4EtzxKoSDeM3NZkG1RRk` (AL-SOL-aut-t).

---

## Step 6: Static Code Review + Fixes

### Checklist Results

| # | Check | Before | After |
|---|-------|--------|-------|
| 1 | SKILL.md description ASCII-only | FAIL — contained Chinese characters: `Kamino流动性, Kamino保险库, 存入Kamino, Kamino赚取收益` | FIXED — replaced with ASCII transliterations |
| 2 | SKILL.md has "Do NOT use for" section | FAIL — missing | FIXED — added section listing 4 out-of-scope use cases |
| 3 | onchainos call checks exit code + ok field | FAIL — `wallet_contract_call_solana` did not check `output.status` or `result["ok"]` | FIXED — added exit-code check and ok-field guard |
| 4 | `extract_tx_hash` returns `Result<String>`, refuses "pending" | FAIL — returned `String`, silently returned `"pending"` on missing hash | FIXED — now returns `anyhow::Result<String>`, errors on empty/pending |
| 5 | `plugin.yaml` source_repo correct | PASS — `GeoGu360/onchainos-plugins` |  |
| 6 | Solana serializedData base58 (no extra conversion) | PASS — `base64_to_base58` called once in `wallet_contract_call_solana`, not duplicated |  |

### Issues Fixed (all merged in commit `8f75b84`)

**Issue 1 — `extract_tx_hash` silently returns `"pending"`**
- File: `src/onchainos.rs`
- Risk: A transaction that failed to broadcast would produce a valid-looking JSON output with `"txHash": "pending"` and a broken explorer link, with no error propagation.
- Fix: Changed return type to `anyhow::Result<String>`. Now bails if `ok != true`, if hash is empty, or if hash equals `"pending"`. Both `deposit.rs` and `withdraw.rs` updated to propagate with `?`.

**Issue 2 — `wallet_contract_call_solana` ignores process exit code and ok field**
- File: `src/onchainos.rs`
- Risk: If `onchainos` crashes or returns `{"ok": false}`, the function would still return `Ok(value)` and the caller would attempt to extract a txHash from an error response.
- Fix: Added `output.status.success()` check (bails with stderr on non-zero exit), and added `result["ok"] == true` guard after JSON parse.

**Issue 3 — SKILL.md description contains non-ASCII (Chinese) characters**
- File: `skills/kamino-liquidity/SKILL.md`
- Fix: Replaced Chinese characters in the `description` field with ASCII-safe transliterations.

**Issue 4 — SKILL.md missing "Do NOT use for" section**
- File: `skills/kamino-liquidity/SKILL.md`
- Fix: Added section immediately after Overview listing: swapping, Kamino Lend/borrow, non-Solana chains, CL range management.

**Issue 5 — Unused constant `SOLANA_CHAIN_ID` (dead code warning)**
- File: `src/config.rs`
- Fix: Removed the unused constant. Build now produces zero warnings.

---

## Step 7: Uninstall

Completed successfully via `npx skills remove kamino-liquidity --yes --global`.

---

## Additional Observations and Suggestions

1. **`resolve_wallet_solana` does not check exit code** — `onchainos wallet balance` exit code is not checked; only JSON parse failure would surface an error. Consider adding `output.status.success()` guard here as well (same pattern as the write path).

2. **API error field assumption** — `build_deposit_tx` and `build_withdraw_tx` fall back to `data["message"]` for error text, but the actual Kamino API may use different error keys. Logging `data.to_string()` as a fallback is correct and safe.

3. **Token filter in `vaults` only matches vault name/mint, not symbol** — The `--token` flag description says "filter by token symbol" but the implementation does a substring match on `name` and `token_mint`. A lookup from known mint → symbol would improve UX but is not a correctness bug.

4. **Withdraw `--amount` unit ambiguity** — The SKILL.md says "shares in UI units" but does not clarify the relationship between shares and underlying tokens. Consider adding a note that `positions` output shows `total_shares` in the same units used by `withdraw --amount`.
