# Synthetix V3 Plugin — Design Document

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| `plugin_name` | `synthetix-v3` |
| `dapp_name` | Synthetix V3 |
| `target_chains` | EVM — Base (8453) |
| `target_protocols` | Synthetix V3 Core (collateral management), Synthetix V3 Perps Market |
| `category` | `defi-protocol` |
| `author` | GeoGu360 |

---

## §1 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | No — 无官方 Rust SDK |
| SDK 支持哪些技术栈？ | JavaScript/TypeScript (@synthetixio/v3-contracts npm package, v8.10.0) |
| 有 REST API？ | No REST API — 纯链上合约调用，通过 eth_call 读取 + onchainos 写入 |
| 有官方 Skill？ | No |
| 开源社区有类似 Skill？ | No 已知 onchainos 社区 Skill |
| 支持哪些链？ | Base (8453) Andromeda 部署；Arbitrum (42161) Andromeda 部署 |
| 是否需要 onchainos 广播？ | Yes — deposit/withdraw 都是链上写操作 |

**接入路径**：API（直接用 eth_call 读合约 + onchainos wallet contract-call 写合约）

---

## §2 接口映射

### 2a. 需要接入的操作

| 操作 | 类型 | 描述 |
|------|------|------|
| `markets` | 链下查询 | 列出所有 Perps 市场及参数 |
| `positions` | 链下查询 | 查询账户的 Perps 持仓 |
| `collateral` | 链下查询 | 查询账户的抵押品余额和可提取金额 |
| `deposit-collateral` | 链上写操作 | 向 Core 存入 sUSDC 抵押品 |
| `withdraw-collateral` | 链上写操作 | 从 Core 提取 sUSDC 抵押品 |

### 2b. 链下查询表

#### markets
- **方法**：`eth_call` to `PerpsMarketProxy.getMarkets()` 然后 `getMarketSummary(marketId)`
- **合约**：`PerpsMarketProxy` = `0x0A2AF931eFFd34b81ebcc57E3d3c9B1E1dE1C9Ce`
- **选择器**（cast 验证）：
  - `getMarkets()`: `0xec2c9016`
  - `getMarketSummary(uint128)`: `0x41c2e8bd`
  - `currentFundingRate(uint128)`: `0xd435b2a2`
- **返回**：market ID, name/symbol, skew, size, funding rate

#### positions
- **方法**：`eth_call` to `PerpsMarketProxy.getAccountOpenPositions(accountId)` + `getOpenPosition(accountId, marketId)` + `getAvailableMargin(accountId)`
- **合约**：`PerpsMarketProxy` = `0x0A2AF931eFFd34b81ebcc57E3d3c9B1E1dE1C9Ce`
- **选择器**（cast 验证）：
  - `getAccountOpenPositions(uint128)`: `0x35254238` — 需要用户提供 accountId
  - `getOpenPosition(uint128,uint128)`: `0x22a73967`
  - `getAvailableMargin(uint128)`: `0x0a7dad2d`
- **参数**：`--account-id <ACCOUNT_ID>`（uint128）
- **返回**：open market IDs, position per market (pnl, accruedFunding, positionSize)

#### collateral
- **方法**：`eth_call` to `CoreProxy.getAccountCollateral(accountId, collateralType)` + `getAccountAvailableCollateral(accountId, collateralType)`
- **合约**：`CoreProxy` = `0x32C222A9A159782aFD7529c87FA34b96CA72C696`
- **选择器**（cast 验证）：
  - `getAccountCollateral(uint128,address)`: `0xef45148e`
  - `getAccountAvailableCollateral(uint128,address)`: `0x927482ff`
- **参数**：`--account-id <ACCOUNT_ID>`
- **collateral token**: sUSDC = `0xC74eA762cF06c9151cE074E6a569a5945b6302E7`
- **返回**：totalDeposited, totalAssigned, totalLocked, available

### 2c. 链上写操作表

#### deposit-collateral
流程：
1. `approve` sUSDC → CoreProxy（spender = CoreProxy）
2. `deposit(uint128 accountId, address collateralType, uint256 tokenAmount)` on CoreProxy

**ABI 编码**：
```
selector: 0x83802968 (deposit(uint128,address,uint256))

calldata:
  bytes4:  0x83802968
  bytes32: accountId (uint128 padded to 32 bytes)
  bytes32: collateralType (address padded)
  bytes32: tokenAmount (uint256 — 18 decimals for sUSDC)
```

**onchainos 命令**：
```bash
# Step 1: Approve sUSDC
onchainos wallet contract-call --chain 8453 \
  --to 0xC74eA762cF06c9151cE074E6a569a5945b6302E7 \
  --input-data 0x095ea7b3<spender_padded><amount_hex>

# Step 2: Deposit
onchainos wallet contract-call --chain 8453 \
  --to 0x32C222A9A159782aFD7529c87FA34b96CA72C696 \
  --input-data 0x83802968<accountId_padded><collateralType_padded><amount_padded>
```

**주의**: amount는 sUSDC (18 decimals), 즉 0.01 sUSDC = 10000000000000000 (1e16)

#### withdraw-collateral
流程：
1. `withdraw(uint128 accountId, address collateralType, uint256 tokenAmount)` on CoreProxy

**ABI 编码**：
```
selector: 0x95997c51 (withdraw(uint128,address,uint256))

calldata:
  bytes4:  0x95997c51
  bytes32: accountId (uint128)
  bytes32: collateralType (address)
  bytes32: tokenAmount (uint256, 18 decimals)
```

**onchainos 命令**：
```bash
onchainos wallet contract-call --chain 8453 \
  --to 0x32C222A9A159782aFD7529c87FA34b96CA72C696 \
  --input-data 0x95997c51<accountId_padded><collateralType_padded><amount_padded>
```

**注意**: withdraw 需要先 undelegateCollateral 或有足够的 available 余额

---

## §3 用户场景

### 场景 1：查询 Perps 市场
**用户**：Show me Synthetix perps markets on Base

**动作序列**：
1. (链下查询) `eth_call` CoreProxy.getMarkets() → 获取 market ID 列表
2. (链下查询) 对每个市场 `eth_call` PerpsMarketProxy.getMarketSummary(marketId) → 市场名称/skew/size
3. (链下查询) 对主要市场 `eth_call` currentFundingRate(marketId) → 资金费率
4. 返回 JSON：market ID、symbol、skew、size、funding rate

### 场景 2：查询账户抵押品和持仓
**用户**：Check my Synthetix V3 positions for account 12345

**动作序列**：
1. (链下查询) `eth_call` CoreProxy.getAccountCollateral(accountId, sUSDC) → totalDeposited/totalAssigned/totalLocked
2. (链下查询) `eth_call` CoreProxy.getAccountAvailableCollateral(accountId, sUSDC) → available amount
3. (链下查询) `eth_call` PerpsMarketProxy.getAccountOpenPositions(accountId) → open market IDs
4. 对每个 open market: `eth_call` getOpenPosition(accountId, marketId) → pnl/funding/size
5. (链下查询) `eth_call` PerpsMarketProxy.getAvailableMargin(accountId) → available margin
6. 返回完整仓位汇总

### 场景 3：存入抵押品
**用户**：Deposit 10 sUSDC as collateral in my Synthetix account 12345

**动作序列**：
1. 解析钱包地址：`onchainos wallet addresses` → chainIndex "8453"
2. (链下查询) 检查 sUSDC 余额：`eth_call` balanceOf(walletAddress) on sUSDC contract
3. 确认用户操作（**ask user to confirm** before proceeding）
4. (链上操作) `onchainos wallet contract-call` → approve sUSDC to CoreProxy
5. 等待 approve 确认（2-step pattern）
6. (链上操作) `onchainos wallet contract-call` → CoreProxy.deposit(accountId, sUSDC, amount)
7. 返回 txHash

### 场景 4：提取抵押品
**用户**：Withdraw 5 sUSDC from my Synthetix collateral account 12345

**动作序列**：
1. 解析钱包地址：`onchainos wallet addresses`
2. (链下查询) `eth_call` CoreProxy.getAccountAvailableCollateral(accountId, sUSDC) → 检查可提取余额
3. 验证请求金额 <= available
4. 确认用户操作（**ask user to confirm** before proceeding）
5. (链上操作) `onchainos wallet contract-call` → CoreProxy.withdraw(accountId, sUSDC, amount)
6. 返回 txHash

---

## §4 外部 API 依赖

| API | 用途 | 端点 |
|-----|------|------|
| Base RPC (eth_call) | 读取合约状态 | `https://base-rpc.publicnode.com` |
| CoreProxy | 抵押品管理、账户查询 | `0x32C222A9A159782aFD7529c87FA34b96CA72C696` |
| PerpsMarketProxy | 市场查询、持仓查询 | `0x0A2AF931eFFd34b81ebcc57E3d3c9B1E1dE1C9Ce` |
| sUSDC Token | ERC-20 approve | `0xC74eA762cF06c9151cE074E6a569a5945b6302E7` |

---

## §5 配置参数

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `chain` | u64 | 8453 | 链 ID (Base mainnet) |
| `account-id` | u128 | — | Synthetix V3 账户 ID |
| `amount` | f64 | — | 操作金额（人类可读，如 10.0） |
| `collateral` | String | "sUSDC" | 抵押品类型 (sUSDC) |
| `dry-run` | bool | false | 模拟模式，不广播 |
| `market-id` | u128 | — | Perps 市场 ID（可选，默认列出所有） |

---

## §6 合约地址汇总 (Base 8453 Andromeda)

| 合约 | 地址 |
|------|------|
| CoreProxy | `0x32C222A9A159782aFD7529c87FA34b96CA72C696` |
| AccountProxy | `0x63f4Dd0434BEB5baeCD27F3778a909278d8cf5b8` |
| USDProxy (snxUSD) | `0x09d51516F38980035153a554c26Df3C6f51a23C3` |
| SpotMarketProxy | `0x18141523403e2595D31b22604AcB8Fc06a4CaA61` |
| PerpsMarketProxy | `0x0A2AF931eFFd34b81ebcc57E3d3c9B1E1dE1C9Ce` |
| PerpsAccountProxy | `0xcb68b813210aFa0373F076239Ad4803f8809e8cf` |
| USDC (collateral) | `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913` |
| sUSDC (synth) | `0xC74eA762cF06c9151cE074E6a569a5945b6302E7` |
| WETH (collateral) | `0x4200000000000000000000000000000000000006` |
| Perps Super Market | ID = 2 |
| ETH Perps Market | ID = 100 |
| BTC Perps Market | ID = 200 |

---

## §7 Function Selectors (cast-verified)

| 函数签名 | 选择器 | 合约 |
|---------|--------|------|
| `deposit(uint128,address,uint256)` | `0x83802968` | CoreProxy |
| `withdraw(uint128,address,uint256)` | `0x95997c51` | CoreProxy |
| `createAccount()` | `0x9dca362f` | CoreProxy |
| `createAccount(uint128)` | `0xcadb09a5` | CoreProxy |
| `getAccountCollateral(uint128,address)` | `0xef45148e` | CoreProxy |
| `getAccountAvailableCollateral(uint128,address)` | `0x927482ff` | CoreProxy |
| `getCollateralConfigurations(bool)` | `0x75bf2444` | CoreProxy |
| `getCollateralPrice(address)` | `0x51a40994` | CoreProxy |
| `getPreferredPool()` | `0x3b390b57` | CoreProxy |
| `getPoolName(uint128)` | `0xf86e6f91` | CoreProxy |
| `getMarkets()` | `0xec2c9016` | PerpsMarketProxy |
| `getMarketSummary(uint128)` | `0x41c2e8bd` | PerpsMarketProxy |
| `getAvailableMargin(uint128)` | `0x0a7dad2d` | PerpsMarketProxy |
| `getOpenPosition(uint128,uint128)` | `0x22a73967` | PerpsMarketProxy |
| `currentFundingRate(uint128)` | `0xd435b2a2` | PerpsMarketProxy |
| `getAccountOpenPositions(uint128)` | `0x35254238` | PerpsMarketProxy |
| `approve(address,uint256)` | `0x095ea7b3` | ERC-20 (sUSDC) |
| `balanceOf(address)` | `0x70a08231` | ERC-20 |

> 所有 selectors 已通过 `cast sig` 验证（Foundry）。`getAccountOpenPositions` 通过 ABI 文件确认。
