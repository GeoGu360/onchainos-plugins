# Venus Core Pool — Plugin Design (PRD)

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | venus |
| dapp_name | Venus Core Pool |
| target_chains | BSC (chain ID 56) — primary |
| target_protocols | Lending, Borrowing (Compound V2 fork) |
| category | defi-protocol |
|接入路径 | API (direct eth_call, no SDK) |
| source_repo | GeoGu360/onchainos-plugins |

---

## §1 接入可行性调研表

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 否 — 仅有 JavaScript/TypeScript SDK (@venusprotocol/isolated-pools) |
| SDK 支持哪些技术栈？ | JS/TS only |
| 有 REST API？ | Venus API (https://api.venus.io) 不稳定/文档缺失；改为直接 eth_call |
| 有官方 Skill？ | 否 |
| 开源社区有类似 Skill？ | 否 |
| 支持哪些链？ | BSC (56) 主部署；Ethereum (1) 和 Arbitrum (42161) 也有部署，本次只接 BSC |
| 是否需要 onchainos 广播？ | Yes — supply/borrow/repay/withdraw 均为链上写操作 |

**接入路径**: API (直接 eth_call 读取链上数据 + onchainos wallet contract-call 写操作)

**注意 (deprecated.md)**: Venus Core Pool 是 Compound V2 fork。检查 deprecated.md 中的处理方式：
- 通过 `markets(address)` 检查 `isListed` 和 `mintGuardianPaused` 判断市场是否冻结
- 如果 supply 被 guardian 暂停，SKILL.md 中需要提示用户

---

## §2 接口映射

### 2a 需要接入的操作

| 操作 | 类型 | 说明 |
|------|------|------|
| get-markets | 链下查询 | 查询所有市场的利率、TVL、借贷参数 |
| get-positions | 链下查询 | 查询用户在各市场的存款/借款余额 |
| supply | 链上写操作 | 存入 ERC-20 资产（mint vToken） |
| supply-bnb | 链上写操作 | 存入原生 BNB（mint vBNB，msg.value） |
| withdraw | 链上写操作 | 取回存款 (redeemUnderlying) |
| borrow | 链上写操作 | 借款 |
| repay | 链上写操作 | 还款 (repayBorrow) |
| enter-market | 链上写操作 | 启用抵押（enterMarkets） |
| claim-rewards | 链上写操作 | 领取 XVS 奖励 |

### 2b 链下查询表

| 操作 | eth_call / RPC | 关键参数 | 返回值 |
|------|--------------|---------|--------|
| get-markets | Comptroller.getAllMarkets() → 每个 vToken eth_call | chain_id | symbol, supplyRatePerBlock, borrowRatePerBlock, totalBorrows, getCash, exchangeRate |
| get-positions | vToken.getAccountSnapshot(wallet) per market | wallet, chain_id | vTokenBalance, borrowBalance, exchangeRate |
| get-account-liquidity | Comptroller.getAccountLiquidity(wallet) | wallet | error, liquidity, shortfall |

**RPC endpoints**:
- BSC: `https://bsc-rpc.publicnode.com`

**Key contract addresses (BSC mainnet, chain 56)**:

| Contract | Address |
|----------|---------|
| Comptroller | `0xfD36E2c2a6789Db23113685031d7F16329158384` |
| vUSDT | `0xfd5840cd36d94d7229439859c0112a4185bc0255` |
| vBNB | `0xa07c5b74c9b40447a954e1466938b865b6bbea36` |
| vBTC | `0x882c173bc7ff3b7786ca16dfed3dfffb9ee7847b` |
| vETH | `0xf508fcd89b8bd15579dc79a6827cb4686a3592c8` |
| vUSDC | `0xeca88125a5adbe82614ffc12d0db554e2e2867c8` |
| vXVS | `0x151b1e2635a717bcdc836ecd6fbb62b674fe3e1d` |

**Underlying token addresses (BSC)**:
| Token | Address | Decimals |
|-------|---------|---------|
| USDT | `0x55d398326f99059ff775485246999027b3197955` | 18 |
| BTC (BTCB) | `0x7130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c` | 18 |
| ETH | `0x2170ed0880ac9a755fd29b2688956bd959f933f8` | 18 |
| USDC | `0x8ac76a51cc950d9822d68b83fe1ad97b32cd580d` | 18 |

### 2c 链上写操作表（EVM BSC）

All write ops target BSC (chain ID 56).

| 操作 | 合约地址 | 函数签名 | Selector (cast sig ✅) | ABI 参数顺序 |
|------|---------|---------|----------------------|------------|
| supply (ERC-20) | vToken address (动态，by symbol) | `mint(uint256)` | `0xa0712d68` ✅ | mintAmount (underlying wei) |
| supply (BNB native) | vBNB `0xa07c5b74c9b40447a954e1466938b865b6bbea36` | `mint()` | `0x1249c58b` ✅ | (no args, value = BNB wei via --amt) |
| withdraw (by underlying amount) | vToken address | `redeemUnderlying(uint256)` | `0x852a12e3` ✅ | redeemAmount (underlying wei) |
| withdraw (by vToken amount) | vToken address | `redeem(uint256)` | `0xdb006a75` ✅ | redeemTokens (vToken wei) |
| borrow | vToken address | `borrow(uint256)` | `0xc5ebeaec` ✅ | borrowAmount (underlying wei) |
| repay (ERC-20) | vToken address | `repayBorrow(uint256)` | `0x0e752702` ✅ | repayAmount (underlying wei) |
| enter-market (enable collateral) | Comptroller | `enterMarkets(address[])` | `0xc2998238` ✅ | vTokens[] |
| claim-rewards (XVS) | Comptroller | `claimVenus(address)` | `0xadcd5fb9` ✅ | holder address |

**ERC-20 approve required for**: supply, repay
- approve(address,uint256) selector: `0x095ea7b3`
- spender = vToken address

**Note on BNB supply**:
- vBNB uses `mint()` (no args), requires `--amt <bnb_wei>` in onchainos CLI

---

## §3 用户场景

### 场景 1：查询 Venus 市场行情
用户说："Show me the Venus lending markets on BSC"

Agent 动作：
1. [链下] Comptroller.getAllMarkets() → list of vToken addresses
2. [链下] For each vToken: symbol(), supplyRatePerBlock(), borrowRatePerBlock(), getCash(), totalBorrows()
3. 计算年化收益 APY = ((supplyRatePerBlock * blocksPerYear + 1) ** blocksPerYear - 1) * 100
4. 输出市场列表 JSON

### 场景 2：存入 USDT 到 Venus
用户说："Supply 10 USDT to Venus on BSC"

Agent 动作：
1. [链下] 解析 vUSDT 地址: `0xfd5840cd36d94d7229439859c0112a4185bc0255`
2. [链下] 检查 USDT 余额
3. [链上] dry-run 预览
4. **ask user to confirm** before proceeding
5. [链上] ERC-20 approve(vUSDT, amount) via wallet contract-call
6. 等待 3 秒
7. [链上] vUSDT.mint(amount) via wallet contract-call
8. 输出 txHash

### 场景 3：查询用户持仓
用户说："Show my Venus positions"

Agent 动作：
1. [链下] 解析钱包地址 via resolve_wallet()
2. [链下] getAllMarkets()
3. [链下] 每个 vToken getAccountSnapshot(wallet)
4. [链下] Comptroller.getAccountLiquidity(wallet)
5. 输出持仓摘要 JSON (供应金额、借款金额、健康因子)

### 场景 4：借款 BNB
用户说："Borrow 0.001 BNB from Venus"

Agent 动作：
1. [链下] 确认用户已供应抵押品
2. [链下] 检查 getAccountLiquidity — 需要 liquidity > 0
3. dry-run 预览
4. **ask user to confirm** before proceeding
5. [链上] vBNB.borrow(amount) via wallet contract-call
6. 输出 txHash

---

## §4 外部 API 依赖

| API | URL | 用途 |
|-----|-----|------|
| BSC RPC | https://bsc-rpc.publicnode.com | 所有链上读取和写入 |

> Venus API (https://api.venus.io) 实测路径均返回 404，不使用。全部通过直接 eth_call 读取。

---

## §5 配置参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| chain | u64 | 56 | Chain ID (only 56 supported) |
| dry_run | bool | false | Simulate without broadcasting |
| asset | String | - | Token symbol (USDT, BNB, BTC, ETH, USDC) |
| amount | f64 | - | Human-readable amount |

---

## §6 架构说明

Venus Core Pool is a Compound V2 fork on BSC. All state reading is via direct `eth_call` — no REST API. Write operations use `onchainos wallet contract-call`.

**Block time**: ~3 seconds (BSC). blocksPerYear ≈ 10512000 for APY calculation.

**Interest rate model**: Per-block rates. APY = ((ratePerBlock * blocksPerYear) / 1e18 + 1)^blocksPerYear - 1, or approximately ratePerBlock * blocksPerYear / 1e18 * 100 for display.

**Exchange rate**: vToken → underlying at exchangeRate / 1e18 (approximately 0.02 for USDT at genesis, increases over time). underlyingAmount = vTokenBalance * exchangeRate / 1e18.
