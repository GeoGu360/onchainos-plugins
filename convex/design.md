# Convex Finance Plugin — Design Document

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `convex` |
| dapp_name | Convex Finance |
| version | 0.1.0 |
| target_chains | Ethereum (chain ID 1) |
| category | defi-protocol |
| tags | convex, crv, cvx, staking, yield, curve |
| author | GeoGu360 |

---

## §1 接入可行性调研表

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 无官方 Rust SDK。直接调合约（EVM ABI 调用）+ Convex/Curve REST API |
| SDK 支持哪些技术栈？ | 无官方 SDK。JavaScript/TypeScript 社区库，无 Rust |
| 有 REST API？ | 是 — `https://api.curve.fi/api/getPools/{blockchainId}/{registryId}` 和 `https://api.convexfinance.com/api/curve-v2/pools`（返回 HTML，不可用）；主用 Curve API |
| 有官方 Skill？ | 无 |
| 开源社区有类似 Skill？ | 无 Convex 专用 Skill，参考 Lido/Curve 实现模式 |
| 支持哪些链？ | 仅 Ethereum mainnet (chain ID 1) 核心合约 |
| 是否需要 onchainos 广播？ | Yes — stake-cvxcrv, unstake-cvxcrv, lock-cvx, unlock-cvx, claim-rewards 均为链上操作 |

**接入路径**: API（Curve REST API for pool data）+ 直接 EVM eth_call（合约读取）

---

## §2 接口映射

### 需要接入的操作表

| 操作 | 类型 | 描述 |
|------|------|------|
| get-pools | 链下查询 | 获取 Convex 支持的 Curve 池列表（含 APY） |
| get-positions | 链下查询 | 查询钱包的 cvxCRV staked、vlCVX locked 余额 |
| stake-cvxcrv | 链上写操作 | 将 cvxCRV 质押到 CvxCrvStaking 获取 CRV/CVX 奖励 |
| unstake-cvxcrv | 链上写操作 | 从 CvxCrvStaking 提取 cvxCRV |
| lock-cvx | 链上写操作 | 将 CVX 锁定为 vlCVX（16 周锁定期） |
| unlock-cvx | 链上写操作 | 解锁到期的 vlCVX（processExpiredLocks） |
| claim-rewards | 链上写操作 | 领取 cvxCRV staking 和/或 vlCVX 的奖励 |

### 链下查询表

#### get-pools
- **API**: `GET https://api.curve.fi/api/getPools/ethereum/main`
- **额外**: `GET https://api.curve.fi/api/getPools/ethereum/factory`
- **关键参数**: 无（可选 `--limit <n>` 控制返回数量）
- **Convex 特有字段**:
  - `convexApy`: Convex 提升后的 APY
  - `rewardsNeedNudging`: 是否有未领取奖励
- **返回关键字段**: `poolAddress`, `name`, `coins[]`, `totalSupply`, `apy`, `convexApy`

#### get-positions
- **方法**: eth_call（直接 RPC）
- **CvxCrvStaking.balanceOf(address)**: `0x70a08231` on `0x3Fe65692bfCD0e6CF84cB1E7d24108E434A7587e`
- **CvxCrvStaking.earned(address)**: `0x008cc262` on `0x3Fe65692bfCD0e6CF84cB1E7d24108E434A7587e`
- **vlCVX.balanceOf(address)**: `0x70a08231` on `0x72a19342e8F1838460eBFCCEf09F6585e32db86E`
- **vlCVX.lockedBalances(address)**: `0x0483a7f6` on `0x72a19342e8F1838460eBFCCEf09F6585e32db86E`

### 链上写操作表（EVM）

| 操作 | 合约地址 | 函数签名 | Selector（cast sig 验证 ✅） | ABI 参数顺序 |
|------|---------|---------|---------------------------|------------|
| ERC-20 approve cvxCRV | `0x62b9c7356a2dc64a1969e19c23e4f579f9810aa7` (cvxCRV token) | `approve(address,uint256)` | `0x095ea7b3` ✅ | spender=CvxCrvStaking, amount |
| stake-cvxcrv | `0x3Fe65692bfCD0e6CF84cB1E7d24108E434A7587e` (CvxCrvStaking) | `stake(uint256)` | `0xa694fc3a` ✅ | amount |
| unstake-cvxcrv | `0x3Fe65692bfCD0e6CF84cB1E7d24108E434A7587e` (CvxCrvStaking) | `withdraw(uint256,address,bool)` | `0x00ebf5dd` ✅ | amount, to(=wallet), claim(=false) |
| ERC-20 approve CVX | `0x4e3FBD56CD56c3e72c1403e103b45Db9da5B9D2B` (CVX token) | `approve(address,uint256)` | `0x095ea7b3` ✅ | spender=vlCVX, amount |
| lock-cvx | `0x72a19342e8F1838460eBFCCEf09F6585e32db86E` (vlCVX) | `lock(uint256,uint256)` | `0x1338736f` ✅ | _amount, _spendRatio(=0) |
| unlock-cvx | `0x72a19342e8F1838460eBFCCEf09F6585e32db86E` (vlCVX) | `processExpiredLocks(bool)` | `0x312ff839` ✅ | _relock(=false) |
| claim-rewards (cvxCRV staking) | `0x3Fe65692bfCD0e6CF84cB1E7d24108E434A7587e` (CvxCrvStaking) | `getReward(address,bool)` | `0x7050ccd9` ✅ | _account(=wallet), _claimExtras(=true) |
| claim-rewards (vlCVX) | `0x72a19342e8F1838460eBFCCEf09F6585e32db86E` (vlCVX) | `getReward()` | `0x3d18b912` ✅ | (no args) |

**Key contracts:**
- Booster: `0xF403C135812408BFbE8713b5A23a04b3D48AAE31` (for LP pool deposits, not in initial scope)
- CvxCrvStaking: `0x3Fe65692bfCD0e6CF84cB1E7d24108E434A7587e`
- vlCVX: `0x72a19342e8F1838460eBFCCEf09F6585e32db86E`
- CVX token: `0x4e3FBD56CD56c3e72c1403e103b45Db9da5B9D2B`
- cvxCRV token: `0x62b9c7356a2dc64a1969e19c23e4f579f9810aa7`
- CRV token: `0xD533a949740bb3306d119CC777fa900bA034cd52`
- RPC: `https://ethereum.publicnode.com`

---

## §3 用户场景

### 场景 1：查询 Convex 收益池

**用户**: "Show me the top Convex Curve pools and their APYs"

**动作序列**:
1. (链下查询) 调用 Curve API `GET https://api.curve.fi/api/getPools/ethereum/main`
2. (链下查询) 调用 Curve API `GET https://api.curve.fi/api/getPools/ethereum/factory`
3. 解析 poolData，按 convexApy 或 normalizedLpTokenPrice 排序
4. 返回 top 10 池: name, tokens, APY, convexApy, totalTvlUsd

### 场景 2：查询钱包持仓

**用户**: "What are my Convex positions?"

**动作序列**:
1. (链下) 解析钱包地址：`onchainos wallet balance --chain 1`
2. (链下 eth_call) `CvxCrvStaking.balanceOf(wallet)` → cvxCRV staked balance
3. (链下 eth_call) `CvxCrvStaking.earned(wallet)` → pending CRV rewards
4. (链下 eth_call) `vlCVX.balanceOf(wallet)` → vlCVX locked balance
5. (链下 eth_call) `CVX.balanceOf(wallet)` → CVX liquid balance
6. (链下 eth_call) `cvxCRV.balanceOf(wallet)` → cvxCRV liquid balance
7. 格式化并输出持仓摘要

### 场景 3：质押 cvxCRV

**用户**: "Stake 10 cvxCRV for me"

**动作序列**:
1. (链下) 解析钱包地址
2. (链下 eth_call) 检查 cvxCRV balance >= 10
3. (链下 eth_call) 检查 allowance(`wallet`, `CvxCrvStaking`)
4. 如果 allowance < amount:
   - Ask user to confirm approve
   - (链上) ERC-20 approve cvxCRV → CvxCrvStaking: `onchainos wallet contract-call --chain 1 --to 0x62b9c7356a2dc64a1969e19c23e4f579f9810aa7 --input-data <0x095ea7b3...>`
   - Wait ~15s for approval
5. Ask user to confirm stake
6. (链上) `CvxCrvStaking.stake(amount)`: `onchainos wallet contract-call --chain 1 --to 0x3Fe65692bfCD0e6CF84cB1E7d24108E434A7587e --input-data <0xa694fc3a...>`
7. 报告 txHash

### 场景 4：锁定 CVX

**用户**: "Lock 100 CVX as vlCVX"

**动作序列**:
1. (链下) 解析钱包地址
2. (链下 eth_call) 检查 CVX balance >= 100
3. (链下 eth_call) 检查 CVX allowance(`wallet`, `vlCVX`)
4. 如果 allowance < amount:
   - Ask user to confirm approve
   - (链上) ERC-20 approve CVX → vlCVX
5. Ask user to confirm lock (注意: 16 周锁定期)
6. (链上) `vlCVX.lock(amount, 0)`: `onchainos wallet contract-call --chain 1 --to 0x72a19342e8F1838460eBFCCEf09F6585e32db86E --input-data <0x1338736f...>`
7. 报告 txHash

### 场景 5：领取奖励

**用户**: "Claim my Convex rewards"

**动作序列**:
1. (链下) 解析钱包地址
2. (链下 eth_call) 检查 cvxCRV staking earned(wallet) — pending CRV
3. (链下 eth_call) 检查 vlCVX 是否有奖励
4. Ask user to confirm
5. 如果有 cvxCRV staking 奖励:
   - (链上) `CvxCrvStaking.getReward(wallet, true)`: `onchainos wallet contract-call --chain 1 --to 0x3Fe65692bfCD0e6CF84cB1E7d24108E434A7587e --input-data <0x7050ccd9...>`
6. 如果有 vlCVX 奖励:
   - (链上) `vlCVX.getReward()`: `onchainos wallet contract-call --chain 1 --to 0x72a19342e8F1838460eBFCCEf09F6585e32db86E --input-data <0x3d18b912...>`
7. 报告 txHash(es)

---

## §4 外部 API 依赖

| API | 用途 | 认证 |
|-----|------|------|
| `https://api.curve.fi/api/getPools/ethereum/main` | 主池列表 + APY | 无 |
| `https://api.curve.fi/api/getPools/ethereum/factory` | Factory 池列表 | 无 |
| `https://ethereum.publicnode.com` | Ethereum mainnet RPC for eth_call | 无 |

---

## §5 配置参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--chain` | `1` | 链 ID（仅支持 1） |
| `--dry-run` | false | 模拟模式，不广播 |
| `--from` | (onchainos 登录钱包) | 覆盖钱包地址 |
| `--limit` | `10` | get-pools 返回数量上限 |
