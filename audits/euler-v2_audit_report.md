# Skill Audit Report -- euler-v2

**Repo**: https://github.com/GeoGu360/onchainos-plugins/tree/main/euler-v2
**Audit Date**: 2026-04-06
**Auditor**: Claude Sonnet 4.6
**Test Wallet**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
**Test Chain**: Base (8453) for write ops; Ethereum (1) for RPC fix verification

---

## Summary

| Item | Result |
|------|--------|
| Compilation | PASS |
| Commands Tested | 7 / 7 |
| On-chain Write Ops | 2 successful (supply + withdraw) |
| Issues Found | 4 P1, 2 P2 |
| P0 Issues | 0 |
| Auto-fixed | All P1 and P2 issues |
| Fix Commits | onchainos-plugins: 3851e81, a4a6123 |
| plugin-store-community | feat/euler-v2: a8e6004 |

---

## Step 0: Environment

- Wallet: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
- ETH balance at start: 0.002720 ETH (above 0.001 stop threshold)
- USDC balance at start: 0.257886 USDC
- Pre-existing USDC position: 0.009999 USDC supplied (9516 shares)

---

## Step 2: Test Plan

| # | Command | Type | Key Params | Test Input |
|---|---------|------|-----------|-----------|
| 1 | `markets` | Query | --chain, --asset | chain 8453, filter USDC |
| 2 | `markets` | Query | --chain | chain 1 (Ethereum) |
| 3 | `positions` | Query | --from | test wallet |
| 4 | `supply` | Write | --vault, --amount | USDC, 0.01 |
| 5 | `withdraw` | Write | --vault, --amount | USDC, 0.01 |
| 6 | `borrow` | Dry-run only | --vault, --amount | USDC, 1 |
| 7 | `repay` | Dry-run only | --vault, --amount | USDC, 0.5 |

---

## Step 3: Compilation

```
cargo build --release
Finished `release` profile [optimized] target(s) in 2m 48s
```

**Result: PASS**

Binary: `/tmp/onchainos-plugins/euler-v2/target/release/euler-v2`

---

## Step 5: Command Test Results

| # | Command | Status | Tx Hash | On-chain Confirm | Notes |
|---|---------|--------|---------|-----------------|-------|
| 1 | `markets --chain 8453` | PASS | - | - | 20 vaults, totalVaults: 257, borrowAPR returned |
| 2 | `markets --chain 8453 --asset USDC` | PASS | - | - | 2 USDC vaults returned |
| 3 | `markets --chain 1 --asset USDC` | PASS (post-fix) | - | - | FAILED before RPC fix; 3 vaults after fix |
| 4 | `positions --chain 8453` | PASS | - | - | Returned 1 active USDC position |
| 5 | `supply --vault USDC --amount 0.01` | PASS | Approve: `0xdfc13db4b26717c1247c2736db961cd6261a7b7c010f5962bd4dc1abdf5ba861` Deposit: `0x5534b51743e2c541b8fe787525dc2a51d874d4075ad04b37820a558116110ef9` | Approve: status 1 block 44330699; Deposit: status 1 block 44330702 | Shares: 9516 -> 19032; supplied: 0.009999 -> 0.019999 USDC |
| 6 | `withdraw --vault USDC --amount 0.01` | PASS | `0x77a44562f6efdb8e2a79f0212e82063dea4d89c201f53315fd17207f18f672c4` | status 1 block 44330920 | Shares: 19032 -> 9515; supplied: 0.019999 -> 0.009998 USDC |
| 7 | `borrow --vault USDC --amount 1` (no --dry-run) | PASS | - | - | Correctly rejected: "borrow is dry-run only" |
| 8 | `--dry-run borrow --vault USDC --amount 1` | PASS | - | - | Returns simulated calldata with EVC steps |
| 9 | `--dry-run repay --vault USDC --amount 0.5` | PASS | - | - | Returns simulated approve+repay calldata |

### On-chain TX Details

**Supply approve TX**
- Hash: `0xdfc13db4b26717c1247c2736db961cd6261a7b7c010f5962bd4dc1abdf5ba861`
- Chain: Base (8453)
- Status: 1 (success)
- Block: 44330699

**Supply deposit TX**
- Hash: `0x5534b51743e2c541b8fe787525dc2a51d874d4075ad04b37820a558116110ef9`
- Chain: Base (8453)
- Status: 1 (success)
- Block: 44330702

**Withdraw TX**
- Hash: `0x77a44562f6efdb8e2a79f0212e82063dea4d89c201f53315fd17207f18f672c4`
- Chain: Base (8453)
- Status: 1 (success)
- Block: 44330920

### Error Handling Test

- Invalid vault `FAKE`: Returns friendly JSON error with suggested fix
- Insufficient balance `9999999 USDC`: Returns friendly "Insufficient USDC balance. Have: 0.249362, Need: 9999999"
- `borrow` without `--dry-run`: Returns friendly "borrow is dry-run only" error

---

## Step 6: Static Code Review

### 6a. SKILL.md Quality (Pre-fix)

- [x] FAIL: No YAML frontmatter -- missing `name`, `description`, trigger phrases **[P1 - FIXED]**
- [x] FAIL: Em-dash (U+2014) in section headers -- non-ASCII **[P2 - FIXED]**
- [x] FAIL: No "Do NOT use for" disambiguation rule **[P1 - FIXED]**
- [x] FAIL: No Chinese trigger phrases **[P1 - FIXED]**
- [x] PASS: All commands documented with examples
- [x] PASS: Parameters clearly explained (--vault accepts address or symbol, --amount is human-readable)

### 6b. Code Quality

- [x] PASS: Amount precision conversion correct (`parse_amount`/`format_amount` handle decimals properly)
- [x] PASS: onchainos `contract-call` used for approve and deposit (correct approach)
- [x] PASS: `--force` flag correctly added for on-chain writes
- [x] PASS: Error messages user-friendly (no raw panics, no bare RPC error codes)
- [x] PASS: Dry-run mode properly gates all write operations
- [x] FAIL: CBBTC vault address wrong in config.rs -- `0x7b181d65...` is wstETH, not cbBTC **[P1 - FIXED]**
- [x] FAIL: Ethereum RPC `eth.llamarpc.com` returns gzip-compressed responses; reqwest missing `gzip` feature causes "RPC response parse failed" **[P1 - FIXED]**

### 6c. ABI/Selector Verification (EVM)

All core selectors verified against keccak256:

| Function | Code Selector | Verified |
|----------|--------------|---------|
| `deposit(uint256,address)` | 0x6e553f65 | PASS |
| `withdraw(uint256,address,address)` | 0xb460af94 | PASS |
| `redeem(uint256,address,address)` | 0xba087652 | PASS |
| `totalAssets()` | 0x01e1d114 | PASS |
| `convertToAssets(uint256)` | 0x07a2d13a | PASS |
| `balanceOf(address)` | 0x70a08231 | PASS |
| `decimals()` | 0x313ce567 | PASS |
| `symbol()` | 0x95d89b41 | PASS |
| `approve(address,uint256)` | 0x095ea7b3 | PASS |
| `debtOf(address)` | 0xd283e75f | PASS |
| `interestRate()` | 0x7c3a00fd | PASS |
| `asset()` | 0x38d52e0f | PASS |
| `borrow(uint256,address)` | 0x4b3fd148 | PASS |
| `repay(uint256,address)` | 0xacb70815 | PASS |
| `getProxyListSlice(uint256,uint256)` | 0xc0e96df6 | PASS |
| `getProxyListLength()` | 0x0a68b7ba | PASS |
| `enableCollateral(address,address)` (EVC dry-run) | 0xb9b2aa44 (WRONG) | FAIL -- Fixed to 0xd44fee5a **[P2 - FIXED]** |
| `enableController(address,address)` (EVC dry-run) | 0x04e5d38d (WRONG) | FAIL -- Fixed to 0xc368516c **[P2 - FIXED]** |

---

## Issues Found and Fixed

### P1 -- Important Issues (All Fixed)

#### P1-1: CBBTC Vault Address Wrong
- **File**: `src/config.rs:99`, `skills/euler-v2/SKILL.md`
- **Problem**: `get_known_vault("CBBTC", 8453)` returned `0x7b181d6509deabfbd1a23af1e65fd46e89572609`, which is actually the **wstETH** vault on Base. Any user supplying to `--vault CBBTC` would deposit into the wstETH vault, not cbBTC.
- **Fix**: Updated to correct cbBTC vault `0x882018411bc4a020a879cee183441fc9fa5d7f8b` (underlying `0xcbB7C0000aB88B473b1f5aFd9ef808440eed33Bf`). Also updated SKILL.md Known Vault Symbols table.

#### P1-2: Ethereum RPC Returns Compressed Response
- **File**: `src/config.rs:17`, `Cargo.toml`
- **Problem**: `eth.llamarpc.com` returns gzip-compressed HTTP responses. `reqwest` without the `gzip` feature cannot decompress these, causing "RPC response parse failed" on every Ethereum chain call.
- **Evidence**: `euler-v2 --chain 1 markets` always failed; direct curl to same endpoint succeeded.
- **Fix**: Changed Ethereum `rpc_url` to `https://ethereum-rpc.publicnode.com` (consistent with other chains using publicnode), and added `gzip` feature to reqwest in `Cargo.toml` as defense-in-depth.

#### P1-3: SKILL.md Missing YAML Frontmatter
- **File**: `skills/euler-v2/SKILL.md`
- **Problem**: SKILL.md had no `---` YAML frontmatter block. Missing `name`, `description`, trigger phrases, and "Do NOT use for" disambiguation rules. Without triggers, the AI assistant cannot reliably route user intents to this skill.
- **Fix**: Added complete YAML frontmatter with `name: euler-v2`, `description` containing English and Chinese trigger phrases, and a "Do NOT use for" clause excluding Aave/Compound/Euler V1.

#### P1-4: Non-ASCII Em-Dash in plugin.yaml
- **File**: `plugin.yaml:5`
- **Problem**: U+2014 em-dash `—` in the `description` field. Plugin store parsers expecting ASCII-only metadata fields may reject or garble the entry.
- **Fix**: Replaced with ASCII hyphen `-`.

### P2 -- Improvement Items (All Fixed)

#### P2-1: EVC enableCollateral/enableController Selectors Wrong in Dry-Run
- **File**: `src/commands/borrow.rs:52-61`
- **Problem**: The dry-run borrow output included simulated EVC calldata using `0xb9b2aa44` for `enableCollateral` and `0x04e5d38d` for `enableController`. Correct keccak256 selectors are `0xd44fee5a` and `0xc368516c` respectively. Since borrow is always dry-run only, this does not cause fund loss, but the calldata guide is misleading.
- **Fix**: Updated both selectors to correct values.

#### P2-2: Non-ASCII Em-Dashes in SKILL.md
- **File**: `skills/euler-v2/SKILL.md`
- **Problem**: Multiple U+2014 em-dashes in section headers and inline text.
- **Fix**: Replaced all em-dashes with ASCII `--` double-hyphen.

---

## Commits

### GeoGu360/onchainos-plugins (main)

| Commit | Message |
|--------|---------|
| `3851e81` | fix(euler-v2): P1 CBBTC vault address, Ethereum RPC, SKILL.md frontmatter, non-ASCII chars |
| `a4a6123` | chore(euler-v2): update source_commit to post-fix HEAD 3851e81 |

### GeoGu360/plugin-store-community (feat/euler-v2)

| Commit | Message |
|--------|---------|
| `a8e6004` | fix(euler-v2): P1 fixes - CBBTC vault, Ethereum RPC, SKILL.md frontmatter, non-ASCII |

---

## Final Score

| Category | Pre-fix | Post-fix |
|----------|---------|---------|
| Compilation | PASS | PASS |
| Base chain operations | 5/5 | 5/5 |
| Ethereum chain operations | 0/1 | 1/1 |
| SKILL.md quality | FAIL | PASS |
| Code correctness | 1 critical bug | PASS |
| Error handling | PASS | PASS |
| ABI selectors (core) | PASS | PASS |
| ABI selectors (dry-run EVC) | FAIL | PASS |
| **Overall** | **CONDITIONAL** | **PASS** |
