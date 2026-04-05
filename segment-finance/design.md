# Segment Finance — Plugin Design (PRD)

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | segment-finance |
| dapp_name | Segment Finance |
| target_chains | BNB Chain (56) |
| target_protocols | Lending (Compound V2 fork) |
| bitable_record_id | recvfIWnsWnw0v |

## §1 接入可行性调研表

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 无 |
| SDK 支持哪些技术栈？ | 无官方 SDK；使用直接 eth_call + onchainos CLI |
| 有 REST API？ | 无 REST API；使用直接合约调用 |
| 有官方 Skill？ | 无 |
| 开源社区有类似 Skill？ | Venus (同样是 Compound V2 fork on BSC) — 直接参考 |
| 支持哪些链？ | BNB Chain (56), opBNB, BOB, CORE。本插件只接入 BSC (56) |
| 是否需要 onchainos 广播？ | Yes — 链上写操作通过 onchainos wallet contract-call |

**接入路径**: 参考 Venus 已有 Skill (Compound V2 fork)，直接 eth_call + onchainos

## §2 接口映射

### 需要接入的操作

| 操作 | 类型 | 说明 |
|------|------|------|
| get-markets | 链下查询 | 读取所有 seToken 市场 APY、利用率 |
| get-positions | 链下查询 | 读取用户供应/借贷仓位 |
| supply | 链上写 | 存入资产，收 seTokens |
| withdraw | 链上写 | 赎回底层资产 |
| borrow | 链上写 | 借款（dry-run only per GUARDRAILS） |
| repay | 链上写 | 还款 |
| enter-market | 链上写 | 开启抵押品 |

### 链下查询

**get-markets:**
- `seToken.supplyRatePerBlock()` selector: `0xae9d70b0`
- `seToken.borrowRatePerBlock()` selector: `0xf8f9da28`
- `seToken.totalBorrows()` selector: `0x47bd3718`
- `seToken.getCash()` selector: `0x3b1d21a2`
- `seToken.exchangeRateStored()` selector: `0x182df0f5`
- `Oracle.getUnderlyingPrice(address)` selector: `0xfc57d4df`
- APY = ratePerBlock * 10_512_000 / 1e18 * 100

**get-positions:**
- `seToken.getAccountSnapshot(address)` selector: `0xc37f68e2`
- `Comptroller.getAccountLiquidity(address)` selector: `0x5ec88c79`

### 链上写操作

| 操作 | 合约 | 函数签名 | Selector (cast sig 验证 ✅) | ABI 参数 |
|------|------|---------|--------------------------|---------|
| supply (ERC-20) | seToken | `mint(uint256)` | `0xa0712d68` ✅ | mintAmount |
| supply (BNB) | seBNB | `mint()` payable | `0x1249c58b` ✅ | --amt <wei> |
| withdraw | seToken | `redeemUnderlying(uint256)` | `0x852a12e3` ✅ | redeemAmount |
| borrow | seToken | `borrow(uint256)` | `0xc5ebeaec` ✅ | borrowAmount |
| repay (ERC-20) | seToken | `repayBorrow(uint256)` | `0x0e752702` ✅ | repayAmount |
| enter-market | Comptroller | `enterMarkets(address[])` | `0xc2998238` ✅ | markets[] |
| approve | ERC-20 | `approve(address,uint256)` | `0x095ea7b3` ✅ | spender, amount |

### Key Contracts (BSC mainnet)

| Contract | Address |
|----------|---------|
| Comptroller (Unitroller/Diamond) | `0x57E09c96DAEE58B77dc771B017de015C38060173` |
| Oracle | `0x763217cFeFac3B26191b1DCaE1926F65157B9A05` |
| seBNB | `0x5fceA94B96858048433359BB5278a402363328C3` |
| seUSDT | `0x44B1E0f4533FD155B9859a9DB292C90E5B300119` |
| seUSDC | `0x8969b89D5f38359fBE95Bbe392f5ad82dd93e226` |
| seBTC | `0x12CD46B96fe0D86E396248a623B81fD84dD0F61d` |
| seETH | `0x3821175E59CD0acDa6c5Fd3eBB618b204e5D7eed` |

**Important**: BSC USDT (`0x55d398326f99059ff775485246999027b3197955`) has **18 decimals** (BEP-20), unlike Ethereum USDT (6 decimals).

**Diamond proxy note**: Comptroller uses EIP-2535 Diamond pattern. `getAllMarkets()` (0xb0772d0b) returns garbage data in some cases — fallback to hardcoded known markets list.

## §3 用户场景

### 场景 1: 查询借贷市场

用户: "查看 Segment Finance 的借贷市场利率"
1. 调用 `segment-finance get-markets --chain 56`
2. 对每个 seToken 调用 supplyRatePerBlock / borrowRatePerBlock / getCash / totalBorrows
3. 计算年化 APY，获取 Oracle USD 价格
4. 返回市场列表

### 场景 2: 存入 USDT 获取利息

用户: "帮我在 Segment Finance 存入 10 USDT"
1. `segment-finance supply --asset USDT --amount 10.0 --chain 56 --dry-run`
2. 预览 calldata 和参数
3. **询问用户确认**
4. `approve(seUSDT, 10 USDT)` via `onchainos wallet contract-call`
5. 等待 3 秒
6. `seUSDT.mint(10 USDT)` via `onchainos wallet contract-call`
7. 返回 txHash

### 场景 3: 查看仓位并借款

用户: "查看我在 Segment Finance 的仓位，然后借 5 USDT"
1. `segment-finance get-positions --chain 56`
2. 检查 account_liquidity (健康因子)
3. `segment-finance enter-market --asset USDT --chain 56 --dry-run` (如未开启)
4. `segment-finance borrow --asset USDT --amount 5.0 --chain 56 --dry-run`
5. **询问用户确认**
6. 执行借款

## §4 外部 API 依赖

| API | URL | 用途 |
|-----|-----|------|
| BSC RPC | https://bsc-rpc.publicnode.com | eth_call 查询合约状态 |

## §5 配置参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| chain | 56 | BSC chain ID |
| dry_run | false | 模拟模式，不广播 |
| BLOCKS_PER_YEAR | 10_512_000 | BSC 约 3s/块 |
