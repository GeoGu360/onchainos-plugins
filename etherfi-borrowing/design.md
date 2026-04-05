# Plugin Design: EtherFi Borrowing (Cash / BNPL)

**Plugin name:** `etherfi-borrowing`
**DApp:** EtherFi Borrowing (ether.fi Cash)
**Category:** defi-protocol
**Tags:** borrowing, lending, collateral, weeth, usdc, etherfi, scroll
**Author:** Plugin Dev Pipeline — Phase 1 Researcher Agent
**Date:** 2026-04-05
**Status:** Complete

---

## §0 Plugin Meta

| Field | Value |
|---|---|
| plugin_name | `etherfi-borrowing` |
| dapp_name | EtherFi Borrowing (ether.fi Cash) |
| target_chains | EVM — Scroll (chain ID 534352) |
| target_protocols | EtherFi Cash — L2DebtManager, UserSafe architecture |
| note | Despite task spec saying chain 1, EtherFi Borrowing (Cash) is deployed on Scroll (534352). Ethereum mainnet only has bridge/top-up contracts. |

---

## §1 Feasibility Table

| Dimension | Assessment |
|---|---|
| Official Rust/WASM SDK | None |
| REST API (off-chain) | No public REST API for rates; direct eth_call only |
| Official Plugin / Skill | None published |
| Community Skill or MCP server | None found |
| Integration path | Direct EVM contract calls via `onchainos wallet contract-call`; reads via direct `eth_call` on Scroll |
| onchainos broadcast needed | Yes — for supply-liquidity, withdraw-liquidity, repay |
| Primary chain | Scroll (chain ID 534352) |
| Architecture | UserSafe model: users have personal smart contract wallets (UserSafe). Borrow() is onlyUserSafe. EOAs can: supply USDC liquidity, withdraw liquidity, repay on behalf of UserSafe |

**Chosen integration path:** Direct EVM calls to DebtManager (supply/withdraw/repay), reads via eth_call on Scroll.

**Architecture note:**
- `DebtManagerProxy` (`0x8f9d2Cd33551CE06dD0564Ba147513F715c2F4a0`) — main entry point
- `CashDataProvider` (`0xb1F5bBc3e4DE0c767ace41EAb8A28b837fBA966F`) — registry
- `UserSafeLens` (`0x333321a783f765bFd4c22FBBC5B2D02b97efB44c`) — position reader
- `UserSafeFactory` (`0x18Fa07dF94b4E9F09844e1128483801B24Fe8a27`) — deploy user safes
- Borrow tokens: USDC (`0x06eFdBFf2a14a7c8E15944D1F4A48F9F95F663A4`)
- Collateral tokens: weETH (`0x01f0a31698C4d065659b9bdC21B3610292a1c506`), USDC, SCR (`0xd29687c813D741E2F938F4aC377128810E217b1b`)

---

## §2 Interface Mapping

### Contract Addresses — Scroll (chain ID 534352)

| Contract | Address |
|---|---|
| DebtManagerProxy (main) | `0x8f9d2Cd33551CE06dD0564Ba147513F715c2F4a0` |
| CashDataProvider | `0xb1F5bBc3e4DE0c767ace41EAb8A28b837fBA966F` |
| UserSafeLens | `0x333321a783f765bFd4c22FBBC5B2D02b97efB44c` |
| UserSafeFactory | `0x18Fa07dF94b4E9F09844e1128483801B24Fe8a27` |
| PriceProvider | `0x8B4C8c403fc015C46061A8702799490FD616E3bf` |
| USDC (Scroll) | `0x06eFdBFf2a14a7c8E15944D1F4A48F9F95F663A4` |
| weETH (Scroll) | `0x01f0a31698C4d065659b9bdC21B3610292a1c506` |
| SCR (Scroll native) | `0xd29687c813D741E2F938F4aC377128810E217b1b` |

### Operations

#### Read Operations (eth_call, no gas)

| Operation | Contract | Function Signature | Selector (verified ✅) |
|---|---|---|---|
| get-markets | DebtManagerProxy | `getBorrowTokens()` | `0x5a52477a` ✅ |
| get-markets | DebtManagerProxy | `getCollateralTokens()` | `0xb58eb63f` ✅ |
| get-rates | DebtManagerProxy | `borrowApyPerSecond(address)` | `0x944e2f5e` ✅ |
| get-rates | DebtManagerProxy | `totalSupplies(address)` | `0x9782e821` ✅ |
| get-rates | DebtManagerProxy | `totalBorrowingAmount(address)` | `0xc94f8d42` ✅ |
| get-rates | DebtManagerProxy | `collateralTokenConfig(address)` | `0xf0ba097e` ✅ |
| get-position | DebtManagerProxy | `collateralOf(address)` | `0x1aefb107` ✅ |
| get-position | DebtManagerProxy | `borrowingOf(address,address)` | `0x4142152e` ✅ |
| get-position | DebtManagerProxy | `remainingBorrowingCapacityInUSD(address)` | `0xf6513bfe` ✅ |
| get-position | DebtManagerProxy | `liquidatable(address)` | `0xffec70af` ✅ |
| get-position | DebtManagerProxy | `supplierBalance(address,address)` | `0x58061652` ✅ |

#### Write Operations (via onchainos wallet contract-call)

| Operation | Contract | Function Signature | Selector (verified ✅) | ABI Params |
|---|---|---|---|---|
| supply-liquidity | DebtManagerProxy | `supply(address,address,uint256)` | `0x0c0a769b` ✅ | user(=wallet), borrowToken(=USDC), amount |
| withdraw-liquidity | DebtManagerProxy | `withdrawBorrowToken(address,uint256)` | `0xa56c8ff7` ✅ | borrowToken(=USDC), amount |
| repay | DebtManagerProxy | `repay(address,address,uint256)` | `0x1da649cf` ✅ | user(=userSafe), token(=USDC), amount |

**Note on repay:** The `repay()` function is public — any EOA can repay on behalf of a UserSafe. Requires prior ERC-20 approve of USDC to DebtManagerProxy. `supply()` requires prior ERC-20 approve of USDC to DebtManagerProxy.

**Note on borrow:** `borrow(address,uint256)` has `onlyUserSafe` modifier — cannot be called by EOA directly. This operation is dry-run only (shows calldata) with a note to users that they need a UserSafe.

---

## §3 User Scenarios

### Scenario 1: Check EtherFi Cash borrowing rates
User: "What are the current borrowing rates on EtherFi Cash?"
1. [Read] `getBorrowTokens()` → [USDC]
2. [Read] `borrowApyPerSecond(USDC)` → rate
3. [Read] `totalSupplies(USDC)` → available liquidity
4. [Read] `totalBorrowingAmount(USDC)` → total borrowed
5. [Read] `getCollateralTokens()` → [weETH, USDC, SCR]
6. [Read] `collateralTokenConfig(weETH)` → LTV=50%, threshold=75%
7. Output: table with borrow APY, available liquidity, utilization

### Scenario 2: Supply USDC liquidity to earn yield
User: "Supply 0.01 USDC to EtherFi Cash to earn yield"
1. [Read] Resolve wallet address from onchainos
2. [Read] Check USDC balance on Scroll
3. [Read] Check USDC allowance for DebtManagerProxy
4. If insufficient allowance: [Write] approve(DebtManagerProxy, amount) on USDC
5. [Write] supply(wallet, USDC, 0.01e6) on DebtManagerProxy
6. Ask user to confirm before broadcasting

### Scenario 3: Check position in EtherFi Cash
User: "Show my EtherFi Cash position for address 0x..."
1. [Read] `collateralOf(userSafe)` → collateral breakdown
2. [Read] `borrowingOf(userSafe, USDC)` → debt amount
3. [Read] `remainingBorrowingCapacityInUSD(userSafe)` → borrowing room
4. [Read] `liquidatable(userSafe)` → health status
5. Output: full position summary

### Scenario 4: Repay USDC debt for a UserSafe
User: "Repay 0.01 USDC debt for UserSafe 0x..."
1. [Read] Resolve wallet and check USDC balance
2. [Read] Check USDC allowance for DebtManagerProxy
3. If insufficient: [Write] approve(DebtManagerProxy, amount) on USDC
4. [Write] repay(userSafe, USDC, amount) on DebtManagerProxy — ask user to confirm

---

## §4 External API Dependencies

| API | Usage | Auth |
|---|---|---|
| Scroll RPC (`https://rpc.scroll.io`) | All eth_call reads | None |
| `https://scroll-rpc.publicnode.com` | Fallback RPC | None |

---

## §5 Config Parameters

| Parameter | Default | Notes |
|---|---|---|
| chain | 534352 | Scroll mainnet |
| rpc_url | `https://rpc.scroll.io` | Scroll RPC |
| dry_run | false | If true, skip all on-chain txs |
| debt_manager | `0x8f9d2Cd33551CE06dD0564Ba147513F715c2F4a0` | DebtManagerProxy |
| usdc | `0x06eFdBFf2a14a7c8E15944D1F4A48F9F95F663A4` | USDC on Scroll |
| weeth | `0x01f0a31698C4d065659b9bdC21B3610292a1c506` | weETH on Scroll |
