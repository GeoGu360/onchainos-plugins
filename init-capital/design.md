# INIT Capital Plugin — Design Document

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `init-capital` |
| dapp_name | INIT Capital |
| target_chains | Blast (81457) — primary EVM (Mantle 5000 not supported by onchainos) |
| plugin_type | Skill + Binary (Rust) |
| onchainos_broadcast | Yes — all write ops via `wallet contract-call` |
| fork_repo | GeoGu360/plugin-store-community |
| source_repo | GeoGu360/onchainos-plugins |

---

## §1 接入可行性调研表

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | No — no official Rust SDK |
| SDK 支持哪些技术栈？ | N/A |
| 有 REST API？ | No public REST API found; uses direct contract calls |
| 有官方 Skill？ | No |
| 开源社区有类似 Skill？ | No found GitHub skill for INIT Capital |
| 支持哪些链？ | Mantle (5000), Blast (81457). Mantle not supported by onchainos → use Blast |
| 是否需要 onchainos 广播？ | Yes — all write ops are EVM contract calls |

**接入路径：** 直接 EVM 合约调用（via `wallet contract-call` + `eth_call` for reads）

**Note on chain selection:**
- INIT Capital primary deployment is Mantle (5000), but `onchainos` returns `"unknown chain: 5000"` → cannot use Mantle
- Blast (81457) deployment is supported by onchainos → use Blast
- Blast pools: WWETH (`0xD20989EB39348994AA99F686bb4554090d0C09F3`), WUSDB (`0xc5EaC92633aF47c0023Afa0116500ab86FAB430F`)
- Since wallet has no funds on Blast, L4 tests will be SKIPPED (L1-L3 only)

---

## §2 接口映射

### 2a 需要接入的操作

| 操作 | 类型 | 说明 |
|------|------|------|
| get-pools | 链下读取 | List all lending pools with rates, total assets |
| get-positions | 链下读取 | List user's positions with health factor |
| supply | 链上写 | Deposit tokens to lending pool (via MoneyMarketHook.execute) |
| withdraw | 链上写 | Withdraw collateral from position |
| borrow | 链上写 | Borrow assets from position |
| repay | 链上写 | Repay debt to position |
| health-factor | 链下读取 | Get position health factor |

### 2b 链下查询表

**get-pools** — query multiple pool contracts via `eth_call`:
- Contract: Each pool address (POOL_WWETH, POOL_WUSDB)
- `totalAssets()` → selector `0x01e1d114` → uint256 total supplied
- `getSupplyRate_e18()` → selector `0xbea205a2` → uint256 supply APR in e18
- `getBorrowRate_e18()` → selector `0x8d80c344` → uint256 borrow APR in e18
- `decimals()` → selector `0x313ce567` → uint8

**get-positions** — query POS_MANAGER via `eth_call`:
- `getViewerPosIdsLength(address)` → selector `0x0c4478c7` → uint256 count
- `getViewerPosIdsAt(address,uint256)` → selector `0xfd379b17` → uint256 posId
- `getPosInfo(uint256)` → selector `0x840eb81c` → (mode, viewer, initPosId, status)
- `getPosBorrInfo(uint256)` → selector `0x947557b3` → borrow info
- `getPosCollInfo(uint256)` → selector `0x056b0ac7` → collateral info
- `getCollAmt(uint256,address)` → selector `0x402414b3` → uint256 shares
- `getPosDebtShares(uint256,address)` → selector `0x10e28e71` → uint256 debt shares

**health-factor** — query INIT_CORE via `eth_call`:
- `getPosHealthCurrent_e18(uint256)` → selector `0xa72ca39b` → uint256 health (1e18 = 1.0)
- `getCollateralCreditCurrent_e36(uint256)` → selector `0xb26ec9af` → uint256
- `getBorrowCreditCurrent_e36(uint256)` → selector `0x147fce8c` → uint256

### 2c 链上写操作表 (EVM — Blast 81457)

All write operations go through the `MoneyMarketHook.execute()` function. The hook takes a single `OperationParams` struct and batches all operations atomically.

#### MoneyMarketHook: execute

| 操作 | 合约地址 | 函数签名 | Selector | ABI 参数顺序 |
|------|---------|---------|----------|------------|
| supply | `0xC02819a157320Ba2859951A1dfc1a5E76c424dD4` (MONEY_MARKET_HOOK Blast) | `execute((uint256,address,uint16,(address,uint256,(address,address))[],(address,uint256,(address,address),address)[],(address,uint256,address)[],(address,uint256)[],uint256,bool))` | `0x247d4981` ✅ | OperationParams tuple |
| withdraw | Same as above | Same | Same | OperationParams with withdrawParams |
| borrow | Same as above | Same | Same | OperationParams with borrowParams |
| repay | Same as above | Same | Same | OperationParams with repayParams |

#### ERC-20 approve (before deposit)

| 操作 | 合约地址 | 函数签名 | Selector | 说明 |
|------|---------|---------|----------|------|
| approve | token address | `approve(address,uint256)` | `0x095ea7b3` ✅ | Approve MoneyMarketHook as spender |

#### OperationParams struct encoding

```
OperationParams = (
  uint256 posId,           // 0 = create new position
  address viewer,          // position viewer (user's address or address(0))
  uint16 mode,             // 1 = general mode
  DepositParams[] depositParams,
  WithdrawParams[] withdrawParams,
  BorrowParams[] borrowParams,
  RepayParams[] repayParams,
  uint256 minHealth_e18,   // min health after ops (0 = no check)
  bool returnNative        // false for ERC20 tokens
)

DepositParams = (address pool, uint256 amt, RebaseHelperParams(address,address))
WithdrawParams = (address pool, uint256 shares, RebaseHelperParams(address,address), address to)
BorrowParams = (address pool, uint256 amt, address to)
RepayParams = (address pool, uint256 shares)
```

**Note on posId:**
- posId = 0 → creates a new position (first supply creates position)
- posId > 0 → operates on existing position

**Key Blast Contract Addresses:**
- INIT_CORE: `0xa7d36f2106b5a5D528a7e2e7a3f436d703113A10`
- POS_MANAGER: `0xA0e172f8BdC18854903959b8f7f73F0D332633fe`
- MONEY_MARKET_HOOK: `0xC02819a157320Ba2859951A1dfc1a5E76c424dD4`
- INIT_LENS: `0x56Fba2cC045C02d7adAE5A9dfDce795900b2860E`
- POOL_WWETH: `0xD20989EB39348994AA99F686bb4554090d0C09F3`
- POOL_WUSDB: `0xc5EaC92633aF47c0023Afa0116500ab86FAB430F`

**Token addresses on Blast:**
- WETH: `0x4300000000000000000000000000000000000004`
- USDB: `0x4300000000000000000000000000000000000003`
- ETH (native): `0x0000000000000000000000000000000000000000`

**RPC URL for Blast:** `https://rpc.blast.io` (fallback: `https://blast-rpc.publicnode.com`)

---

## §3 用户场景

### 场景 1: 查看借贷市场利率

用户说: "Show me the INIT Capital lending pools on Blast with current rates"

Agent 动作序列:
1. 链下查询: `eth_call` to POOL_WWETH `getSupplyRate_e18()` → supply APY
2. 链下查询: `eth_call` to POOL_WWETH `getBorrowRate_e18()` → borrow APY
3. 链下查询: `eth_call` to POOL_WWETH `totalAssets()` → total supplied
4. Repeat for POOL_WUSDB
5. Format and display: pool name, supply APY%, borrow APY%, total liquidity

### 场景 2: 供应抵押品并借款

用户说: "Supply 0.01 WETH to INIT Capital on Blast and borrow 5 USDB"

Agent 动作序列:
1. 链下查询: resolve user wallet address via `onchainos wallet balance --chain 81457`
2. 链下查询: `eth_call` to POOL_WWETH to check user position and collateral factor
3. 链上写: ERC-20 approve WETH to MONEY_MARKET_HOOK (via `wallet contract-call`)
4. 链上写: `execute(OperationParams{posId:0, depositParams:[{pool:POOL_WWETH, amt:0.01e18}], borrowParams:[{pool:POOL_WUSDB, amt:5e18, to:userAddr}], minHealth_e18:1.1e18})` via `wallet contract-call`
5. 链下查询: Check resulting health factor via INIT_CORE `getPosHealthCurrent_e18(posId)`

### 场景 3: 查看持仓状态

用户说: "Show my INIT Capital positions on Blast"

Agent 动作序列:
1. 链下查询: resolve wallet via `onchainos wallet balance --chain 81457`
2. 链下查询: `eth_call` to POS_MANAGER `getViewerPosIdsLength(walletAddr)` → count
3. 链下查询: For each posId, `getViewerPosIdsAt(walletAddr, i)` → posId
4. 链下查询: For each posId, `getPosInfo(posId)`, `getPosBorrInfo(posId)`, `getPosCollInfo(posId)`
5. 链下查询: INIT_CORE `getPosHealthCurrent_e18(posId)` → health factor
6. Display: position ID, mode, collateral, debt, health factor

### 场景 4: 还款并提现

用户说: "Repay my USDB debt and withdraw WETH from INIT Capital position 1"

Agent 动作序列:
1. 链下查询: get debt shares via `getPosDebtShares(posId, POOL_WUSDB)`
2. 链上写: ERC-20 approve USDB to MONEY_MARKET_HOOK
3. 链上写: `execute(OperationParams{posId:1, repayParams:[{pool:POOL_WUSDB, shares:debtShares}], withdrawParams:[{pool:POOL_WWETH, shares:collShares, to:userAddr}]})` via `wallet contract-call`

---

## §4 外部 API 依赖

| API | 用途 | 认证 |
|-----|------|------|
| Blast RPC `https://rpc.blast.io` | eth_call for reads | None |
| Blast RPC fallback `https://blast-rpc.publicnode.com` | Fallback | None |

---

## §5 配置参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| chain | u64 | 81457 | Chain ID (Blast) |
| rpc_url | string | https://rpc.blast.io | EVM RPC endpoint |
| dry_run | bool | false | Simulate without broadcasting |
| pos_id | u64 | 0 | Position ID (0 = create new) |
| mode | u16 | 1 | Position mode (1 = general) |

---

## §6 注意事项

1. **Position ID**: INIT Capital uses NFT-based positions. When posId=0, the hook creates a new position. Existing positions are identified by posId from POS_MANAGER.
2. **Borrow/repay = dry-run only** (GUARDRAILS): L4 tests only do supply/withdraw. Borrow and repay are L3 dry-run only.
3. **No funds on Blast**: L4 tests are SKIPPED due to zero wallet balance on Blast (81457). L1-L3 fully executed.
4. **Mantle (5000) unsupported**: onchainos returns "unknown chain: 5000". All on-chain work uses Blast (81457).
5. **WWETH vs WETH**: Blast pools use WWETH (a wrapped version specific to Blast). The actual token to deposit is WETH which gets auto-wrapped.
6. **RebaseHelperParams**: For standard ERC20s, set helper=address(0) and tokenIn=address(0).
7. **repay shares**: RepayParams.shares is the debt share amount (from getPosDebtShares), not the token amount.
