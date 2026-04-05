# Notional V3 Plugin Design

## §0 Plugin Meta

- **plugin_name**: `notional-v3`
- **dapp_name**: Notional V3 (Notional Exponent)
- **target_chains**: Ethereum (chain ID 1) — primary deployment for Exponent (V4)
- **category**: `defi-protocol`
- **tags**: `lending`, `yield`, `leveraged-yield`, `fixed-rate`
- **接入路径**: API (REST + GraphQL subgraph, no Rust SDK)

---

## §1 接入可行性调研表

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | ❌ 无。TypeScript SDK 在 notional-monorepo（无 Rust） |
| SDK 支持哪些技术栈？ | TypeScript/JavaScript |
| 有 REST API？ | 部分：The Graph 子图（GraphQL）+ 直接 eth_call |
| 有官方 Skill？ | ❌ 无 |
| 开源社区有类似 Skill？ | ❌ 无 |
| 支持哪些链？ | Ethereum mainnet (1) — Exponent 目前仅部署在 ETH 主网 |
| 是否需要 onchainos 广播？ | ✅ Yes — enterPosition, exitPosition, initiateWithdraw |

### 重要背景说明

**Notional V3 协议状态**：
- 原始 Notional V3 在 Arbitrum (0x1344A36A1B56144C3Bc62E7757377D288fDE0369) 和 Ethereum (0x6e7058c91F85E0F6db4fc9da2CA41241f5e4263f) 均已**完全暂停** (paused)
- 所有用户调用通过代理路由到 pauseRouter，全部 revert
- 当前活跃产品为 **Notional Exponent** (即 V4)，是重新设计的杠杆收益协议
- Exponent 架构：MorphoLendingRouter (0x9a0c630c310030c4602d1a76583a3b16972ecaa0) + AddressRegistry + 多个 Vault 策略
- Exponent 目前仅部署在 **Ethereum mainnet (chain 1)**
- TVL：~$3.3M（主要在 Ethereum 的 Exponent 协议）
- 子图：https://api.studio.thegraph.com/query/60626/notional-exponent/version/latest（无需 API key）

---

## §2 接口映射

### 2a. 需要接入的操作

| 操作 | 类型 | 说明 |
|------|------|------|
| `get-vaults` | 链下查询 | 查询所有可用的杠杆收益 Vault |
| `get-positions` | 链下查询 | 查询钱包在各 Vault 的持仓 |
| `get-health-factor` | 链下查询 | 查询特定持仓的健康因子 |
| `enter-position` | 链上操作 | 进入杠杆收益仓位（存入资产 + 借款） |
| `exit-position` | 链上操作 | 退出仓位（赎回）|
| `initiate-withdraw` | 链上操作 | 发起提款（用于 staking 策略） |
| `claim-rewards` | 链上操作 | 领取激励奖励 |

### 2b. 链下查询表

| 操作 | API | 关键参数 | 返回值 |
|------|-----|---------|--------|
| `get-vaults` | GraphQL subgraph query `vaults` | isWhitelisted: true | vault id, asset symbol/address, yieldToken symbol |
| `get-positions` | GraphQL subgraph query `balances` + `account(address)` | wallet address | token type, currentBalance, vault info |
| `get-health-factor` | eth_call `healthFactor(address,address)` on MorphoLendingRouter | user address, vault address | healthFactor (uint256, precision 1e18) |
| `get-health-factor` | eth_call `balanceOfCollateral(address,address)` | user address, vault address | collateral balance |

**GraphQL Subgraph Endpoint**: `https://api.studio.thegraph.com/query/60626/notional-exponent/version/latest`

**Key Queries**:
```graphql
# List vaults
{ vaults(where: { isWhitelisted: true }) {
    id isWhitelisted
    asset { id symbol decimals tokenAddress }
    yieldToken { id symbol decimals tokenAddress }
} }

# Get positions for an account
{ account(id: "0x...") {
    balances {
      token { id symbol tokenType }
      current { currentBalance currentProfitAndLossAtSnapshot impliedFixedRate }
      lendingRouter { id name }
    }
} }
```

### 2c. 链上写操作表（EVM — Ethereum chain 1）

| 操作 | 合约地址（来源） | 函数签名（canonical ABI） | Selector（cast sig ✅） | ABI 参数顺序 |
|------|---------------|--------------------------|------------------------|------------|
| enter-position | `0x9a0c630C310030C4602d1A76583a3b16972ecAa0` (MorphoLendingRouter, hardcoded) | `enterPosition(address,address,uint256,uint256,bytes)` | `0xde13c617` ✅ | onBehalf(=wallet), vault, depositAssetAmount, borrowAmount(=0 for no leverage), depositData(=0x) |
| exit-position | `0x9a0c630C310030C4602d1A76583a3b16972ecAa0` | `exitPosition(address,address,uint256,uint16,bytes)` | `0x8a363181` ✅ | onBehalf(=wallet), vault, sharesAmount, param16(=0), data(=0x) |
| initiate-withdraw | `0x9a0c630C310030C4602d1A76583a3b16972ecAa0` | `initiateWithdraw(address,address,uint256)` | `0x37753799` ✅ | onBehalf(=wallet), vault, sharesAmount |
| claim-rewards | `0x9a0c630C310030C4602d1A76583a3b16972ecAa0` | `claimRewards(address,address)` | `0xf1e42ccd` ✅ | onBehalf(=wallet), vault |

**ERC-20 approve required for enter-position**: `asset.approve(morphoLendingRouter, depositAssetAmount)` first.

**Key Contract Addresses**:
- MorphoLendingRouter: `0x9a0c630C310030C4602d1A76583a3b16972ecAa0`
- AddressRegistry: `0xe335d314BD4eF7DD44F103dC124FEFb7Ce63eC95`
- Known Vaults (Ethereum mainnet):
  - sUSDe vault (USDC): `0xaf14d06a65c91541a5b2db627ecd1c92d7d9c48b`
  - mAPOLLO vault (USDC): `0x091356e6793a0d960174eaab4d470e39a99dd673`
  - mHYPER vault (USDC): `0x2a5c94fe8fa6c0c8d2a87e5c71ad628caa092ce4`
  - weETH vault (WETH): `0x7f723fee1e65a7d26be51a05af0b5efee4a7d5ae`
  - PT-sUSDE vault (USDC): `0x0e61e810f0918081cbfd2ac8c97e5866daf3f622`
  - liUSD-4w vault (USDC): `0x9fb57943926749b49a644f237a28b491c9b465e0`
  - Convex OETH/WETH vault (WETH): `0x2716561755154eef59bc48eb13712510b27f167f`
  - mHYPER vault 2 (USDC): `0x94f6cb4fae0eb3fa74e9847dff2ff52fd5ec7e6e`

**Token Addresses (Ethereum mainnet)**:
- USDC: `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48`
- WETH: `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2`

---

## §3 用户场景

### 场景 1：查询可用 Vault（读取）

**用户说**："Show me available Notional yield vaults"

**动作序列**:
1. [链下查询] GraphQL subgraph `vaults(where: { isWhitelisted: true })` → 返回 8 个活跃 Vault
2. [链下查询] 对每个 Vault eth_call `healthFactor` 确认 Morpho 市场存在
3. 格式化输出：vault 地址、asset（USDC/WETH）、yield strategy（sUSDe/mAPOLLO/weETH 等）

### 场景 2：查询持仓（读取）

**用户说**："Check my Notional positions"

**动作序列**:
1. [链下] resolve_wallet() 获取 EVM 钱包地址
2. [链下查询] GraphQL subgraph `account(id: "0x...")` with balances
3. [链下查询] eth_call `healthFactor(wallet, vault)` on MorphoLendingRouter for each position
4. 格式化输出：vault name, collateral balance, borrow amount, health factor

### 场景 3：进入仓位（链上写）

**用户说**："Enter the sUSDe Notional vault with 0.01 USDC"

**动作序列**:
1. [链下] resolve_wallet() → wallet address
2. [预览] dry-run 显示 calldata
3. [用户确认] Ask user to confirm before executing
4. [链上] ERC-20 approve: `USDC.approve(0x9a0c63..., 10000)` → onchainos wallet contract-call
5. [等待 3s]
6. [链上] `enterPosition(wallet, vault_addr, 10000, 0, 0x)` → onchainos wallet contract-call
7. 返回 txHash

### 场景 4：退出仓位（链上写）

**用户说**："Exit my sUSDe vault position"

**动作序列**:
1. [链下] resolve_wallet() → wallet address
2. [链下查询] subgraph `account` → get share balance (currentBalance)
3. [预览] dry-run 显示 calldata
4. [用户确认] Ask user to confirm before executing
5. [链上] `exitPosition(wallet, vault_addr, shares, 0, 0x)` → onchainos wallet contract-call
6. 返回 txHash

### 场景 5：查询健康因子（读取）

**用户说**："What's my health factor in the sUSDe vault?"

**动作序列**:
1. [链下] resolve_wallet() → wallet address
2. [链下查询] eth_call `healthFactor(wallet, vault)` on MorphoLendingRouter
3. [链下查询] eth_call `balanceOfCollateral(wallet, vault)` on MorphoLendingRouter
4. 格式化输出：health factor (value/1e18), collateral balance

---

## §4 外部 API 依赖

| API | URL | 认证 | 用途 |
|-----|-----|------|------|
| The Graph Subgraph | `https://api.studio.thegraph.com/query/60626/notional-exponent/version/latest` | 无需 API key | vault/position/token 数据 |
| Ethereum RPC | `https://ethereum.publicnode.com` | 无 | eth_call (healthFactor, balanceOfCollateral) |

---

## §5 配置参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--chain` | `1` (Ethereum) | 仅支持 chain ID 1 |
| `--dry-run` | false | 跳过 onchainos 广播，返回模拟响应 |
| `--vault` | — | Vault 合约地址（get-positions 时可选，enter/exit 时必须） |
| `--amount` | — | 资产数量（enter-position 用，UI 单位） |
