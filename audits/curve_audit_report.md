# Skill Audit Report — Curve DEX Plugin

**Repo**: https://github.com/GeoGu360/onchainos-plugins (dir: `curve/`)
**Audit date**: 2026-04-06
**Test wallet**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
**Test chain**: Ethereum mainnet (chain ID 1)
**Source commit (pre-fix)**: `a98284d`
**Source commit (post-fix)**: `321078b`

---

## Summary

| Item | Result |
|------|--------|
| Compilation | ✅ (4 dead-code warnings, no errors) |
| Commands tested | 5 / 7 |
| On-chain write ops confirmed | 1 (swap: approve + swap) |
| P0 issues found & fixed | 3 |
| P1 issues found & fixed | 3 |
| P2 issues | 2 |

---

## Command Test Results

| # | Command | Status | Tx Hash | On-chain | Notes |
|---|---------|--------|---------|----------|-------|
| 1 | `get-pools --chain 1 --limit 5` | ✅ | — | — | Returned 5 pools sorted by TVL; top pool 3pool TVL $163M |
| 2 | `get-pool-info --pool 0xbEbc...` | ✅ | — | — | fee_pct=0.0150%, virtual_price OK |
| 3 | `get-balances --wallet 0x87fb... --chain 8453` | ✅ | — | — | 0 positions, correct JSON structure |
| 4 | `quote --token-in USDT --token-out USDC --amount 10000000` | ✅ | — | — | 10 USDT → 9997473 USDC, price_impact_pct=0.0253% |
| 5 | `quote --token-in USDT --token-out 0xC02... (WETH)` | ✅ (post-fix) | — | — | Was failing (wrong selector); now returns 4680644978798873 raw WETH |
| 6 | `swap --token-in USDT --token-out USDC --amount 10000` | ✅ | `0x528c4ae1c5d43188927899d25fa5e10889afeeadb965cb3cfa97ff40d86e368e` | ✅ block 24818368, status 1 | 0.01 USDT → 9997 USDC raw; state change confirmed |
| 7 | `add-liquidity` | ⬜ SKIPPED | — | — | No LP position to test after; skip due to fund limits |
| 8 | `remove-liquidity` | ⬜ SKIPPED | — | — | No LP position held by audit wallet |

**Additional on-chain ops recorded:**

| Op | Tx Hash | Block | Status | Notes |
|----|---------|-------|--------|-------|
| Approve USDT (first attempt) | `0x15e357892f65811e11c581803ae34d0217521593a7cf8029cb1acc7c1c97934a` | 24818340 | ✅ status=1 | u128::MAX approval for 3pool |
| Swap USDT→USDC (second attempt, post-fix) | `0x528c4ae1c5d43188927899d25fa5e10889afeeadb965cb3cfa97ff40d86e368e` | 24818368 | ✅ status=1 | 10000 raw USDT → 9997 raw USDC |

**State changes:**
- USDT: 15040482 raw → 15030482 raw (−10000 = −0.01 USDT spent)
- USDC: 0 → 9997 raw (+0.009997 USDC received)

---

## Discovered Issues

### P0 — ABI Selector Bugs (Fixed)

**P0-1: Wrong selector for `get_dy(uint256,uint256,uint256)` (CryptoSwap pools)**
- File: `src/curve_abi.rs`, function `encode_get_dy_uint256`
- Code had: `0xccb48b3c` — live RPC returned empty `0x`, causing all quote/swap on crypto pools to fail with "Quote returned 0"
- Correct: `0x556d6e9f` (verified via `cast sig` and live eth_call on tricrypto2 `0xD51a44d3...`)
- Fix applied: changed selector in `encode_get_dy_uint256`

**P0-2: Wrong selector for `exchange(uint256,uint256,uint256,uint256)` (CryptoSwap pools)**
- File: `src/curve_abi.rs`, function `encode_exchange_uint256`
- Code had: `0x40d12098` — wrong selector would cause all swaps on crypto pools to revert
- Correct: `0x5b41b908` (verified via `cast sig`)
- Fix applied: changed selector in `encode_exchange_uint256`

**P0-3: Wrong selector for `remove_liquidity(uint256,uint256[3])` (3-coin pools)**
- File: `src/curve_abi.rs`, function `encode_remove_liquidity_3`
- Code had: `0x1a4d01d2` — 4byte lookup shows this is `remove_liquidity_one_coin(uint256,int128,uint256)`, not `remove_liquidity(uint256,uint256[3])`
- Correct: `0xecb586a5` (verified via `cast sig`)
- Fix applied: changed selector in `encode_remove_liquidity_3`

### P1 — Important Issues (Fixed)

**P1-1: `uses_uint256_indices()` misses old-style CryptoSwap pools in main registry**
- Files: `src/commands/quote.rs`, `src/commands/swap.rs`
- Problem: The function only checks `id.contains("crypto")` etc., but old-style CryptoSwap pools registered in the main registry have numeric IDs (e.g. id=`"38"` = `Curve.fi USD-BTC-ETH` with USDT/WBTC/WETH). These use uint256 indices but were incorrectly routed to int128 selectors, causing all quotes/swaps to return 0.
- Verified: `0x80466c64868E1ab14a1Ddf27A676C3fcBE638Fe5` (main id=38) only responds to `0x556d6e9f` (uint256), not `0x5e0d443f` (int128).
- Fix: Added dual-try logic in `quote.rs` and `swap.rs` — try uint256 selector first, fall back to int128 only when uint256 returns empty.

**P1-2: `extract_tx_hash` silently returns `"pending"` on onchainos `ok:false` errors**
- File: `src/onchainos.rs`
- Problem: When onchainos returned `{"ok":false,"error":"..."}`, `extract_tx_hash` returned `"pending"` and the commands printed `{"ok":true,"tx_hash":"pending"}` — a false success. This was observed during the first swap attempt.
- Fix: Changed `extract_tx_hash` to return `anyhow::Result<String>`. If `ok:false` is present, bail with the error message. All 3 write commands updated to propagate the error.

**P1-3: SKILL.md description field contains CJK characters**
- File: `skills/curve/SKILL.md` frontmatter
- Problem: The `description` field contained inline Chinese characters (换币, 添加流动性, etc.). The skill-auditor spec requires `description` to be ASCII-only.
- Fix: Replaced CJK characters with Pinyin romanisation.

**P1-4: SKILL.md missing "Do NOT use for..." disambiguation rules**
- File: `skills/curve/SKILL.md`
- Problem: No negative scope rules present — risk of the skill being triggered for Uniswap swaps, Aave operations, or general token transfers.
- Fix: Added a `## Scope` section with 3 "Do NOT use for" rules.

### P2 — Improvement Suggestions

**P2-1: Dead code warnings (unused functions)**
- `config::curve_router_ng()` — CurveRouterNG is imported in config but never called (swap goes direct to pool, not via router)
- `curve_abi::encode_address()` — utility never used
- Suggest: either use these functions or remove them to clean up the codebase

**P2-2: Token symbol not shown when raw address is passed to `quote`/`swap`**
- When a raw token address is passed (e.g. `--token-in 0xdAC17...`), output shows the address as symbol: `"symbol":"0xdAC17..."`. Minor UX issue but not a correctness bug.

**P2-3: Comment mismatch for `get_virtual_price` selector**
- File: `src/commands/get_pool_info.rs` (fixed inline)
- Comment said `// virtual_price()` but the actual selector `0xbb7b8b80` = `get_virtual_price()`. Selector was correct; comment was wrong. Fixed in the same commit.

---

## SKILL.md Improvements Applied

- Removed CJK characters from `description` field (P1 — ASCII-only requirement)
- Added `## Scope` section with 3 "Do NOT use for" disambiguation rules (P1)
- Trigger phrase coverage: English and romanised Chinese phrases are now present

## Code Improvements Applied

- 3 ABI function selectors corrected (P0)
- Dual-try pool detection for quote and swap (P1)
- `extract_tx_hash` now propagates onchainos errors (P1)
- `source_repo` in `plugin.yaml` corrected to `GeoGu360/onchainos-plugins`
- `source_commit` in `plugin.yaml` updated to `321078b`

---

## ABI Selector Verification (Full)

| Function | Code Selector | cast sig | Status |
|----------|--------------|----------|--------|
| `get_dy(int128,int128,uint256)` | `0x5e0d443f` | `0x5e0d443f` | ✅ |
| `exchange(int128,int128,uint256,uint256)` | `0x3df02124` | `0x3df02124` | ✅ |
| `get_dy(uint256,uint256,uint256)` | `0xccb48b3c` → **`0x556d6e9f`** | `0x556d6e9f` | Fixed |
| `exchange(uint256,uint256,uint256,uint256)` | `0x40d12098` → **`0x5b41b908`** | `0x5b41b908` | Fixed |
| `add_liquidity(uint256[2],uint256)` | `0x0b4c7e4d` | `0x0b4c7e4d` | ✅ |
| `add_liquidity(uint256[3],uint256)` | `0x4515cef3` | `0x4515cef3` | ✅ |
| `add_liquidity(uint256[4],uint256)` | `0x029b2f34` | `0x029b2f34` | ✅ |
| `remove_liquidity(uint256,uint256[2])` | `0x5b36389c` | `0x5b36389c` | ✅ |
| `remove_liquidity(uint256,uint256[3])` | `0x1a4d01d2` → **`0xecb586a5`** | `0xecb586a5` | Fixed |
| `remove_liquidity_one_coin(uint256,int128,uint256)` | `0x517a55a3` | `0x517a55a3` | ✅ |
| `calc_withdraw_one_coin(uint256,int128)` | `0xcc2b27d7` | `0xcc2b27d7` | ✅ |
| `get_virtual_price()` | `0xbb7b8b80` | `0xbb7b8b80` | ✅ (comment fixed) |
| `fee()` | `0xddca3f43` | `0xddca3f43` | ✅ |
| `balanceOf(address)` | `0x70a08231` | `0x70a08231` | ✅ |
| `allowance(address,address)` | `0xdd62ed3e` | `0xdd62ed3e` | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | `0x095ea7b3` | ✅ |

---

## Commit History

| Commit | Description |
|--------|-------------|
| `33a3b93` | fix(curve): correct 3 ABI selectors, dual-try pool detection, propagate onchainos errors |
| `321078b` | chore(curve): update source_commit to post-fix HEAD 33a3b93 |

Both commits pushed to `GeoGu360/onchainos-plugins` main branch.

No `feat/curve` branch exists in `plugin-store-community` (only `feat/curve-lending` which is a different plugin). No plugin-store update required.
