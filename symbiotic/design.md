# Symbiotic Plugin — Design Document

## §0 Plugin Meta

| 字段 | 值 |
|------|-----|
| plugin_name | `symbiotic` |
| dapp_name | Symbiotic |
| target_chains | Ethereum (chain ID: 1) |
| target_protocols | Restaking / Shared Security |
| category | defi-protocol |
| version | 0.1.0 |

---

## §1 接入可行性调研表

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 无 Rust SDK for DApp interaction. relay-client-rs 是 Relay sidecar（仅用于网络中间件，不适合用户交互） |
| SDK 支持哪些技术栈？ | TypeScript relay-client-ts, Go relay, Rust relay-client-rs（均为 Relay SDK，非 vault interaction SDK） |
| 有 REST API？ | 有，`https://app.symbiotic.fi/api/v2/` — vaults list, vault detail，已验证 |
| 有官方 Skill？ | 无 |
| 开源社区有类似 Skill？ | 无 |
| 支持哪些链？ | Ethereum Mainnet (chain ID: 1)。Holešky/Sepolia/Hoodi 仅测试网 |
| 是否需要 onchainos 广播？ | Yes — deposit/withdraw 是 EVM 链上写操作，需要 wallet contract-call |

**接入路径：** API（Symbiotic REST API 获取 vault 列表 + RPC eth_call 读取链上数据）+ 合约直调（deposit/withdraw 通过 onchainos wallet contract-call）

---

## §2 接口映射

### 2a. 需要接入的操作表

| 操作 | 类型 | 优先级 |
|------|------|--------|
| `vaults` — 列出所有 vault（名称、代币、TVL、APR） | 链下查询（REST API） | P0 |
| `positions` — 查询用户的 restaking 仓位 | 链下+链上查询 | P0 |
| `rates` — 查询 vault APR 和收益率 | 链下查询（REST API） | P0 |
| `deposit` — 向 vault 存款 | 链上写操作 (EVM) | P0 |
| `withdraw` — 从 vault 发起提款请求 | 链上写操作 (EVM) | P0 |

### 2b. 链下查询表

#### `vaults` — 列出 Vault

- **API Endpoint:** `GET https://app.symbiotic.fi/api/v2/vaults`
- **参数：** 无必需参数（可选 `limit`, `page`）
- **返回值（每个 vault）：**
  ```json
  {
    "address": "0x...",
    "meta": { "name": "wstETH", "description": "..." },
    "token": {
      "address": "0x...",
      "symbol": "wstETH",
      "decimals": 18,
      "usdPrice": 2516.47
    },
    "tvl": 59272873.71,
    "vaultRewardsApr": 0.0237,
    "restricted": false,
    "slashable": true,
    "legacy": false
  }
  ```
- **关键字段：** `address`, `meta.name`, `token.symbol`, `token.decimals`, `tvl`, `vaultRewardsApr`

#### `rates` — 查询 APR

- **API Endpoint:** `GET https://app.symbiotic.fi/api/v2/vaults` (same API, extract APR)
- 提取 `vaultRewardsApr` (decimal, e.g. 0.0237 = 2.37%)
- 也包含 `vaultRewards` 数组，每项含 `apr` 和奖励代币信息

#### `positions` — 查询用户仓位

- **链上查询（eth_call）** — 对每个 vault 调用 `activeBalanceOf(address)`
- **API Endpoint（可选增强）:** `GET https://app.symbiotic.fi/api/v2/vaults?limit=100` 获取所有 vault 地址，然后批量 eth_call
- **eth_call:**
  - `activeBalanceOf(address)` → `0x59f769a9` — 用户当前活跃存款余额（vault shares）
  - `withdrawalsOf(uint256 epoch, address account)` → `0xf5e7ee0f` — 特定 epoch 的待提款
  - `currentEpoch()` → `0x76671808` — 当前 epoch 号
  - `collateral()` → `0xd8dfeb45` — vault 的底层资产代币地址

### 2c. 链上写操作表

#### `deposit` — 存款到 Vault

**流程：**
1. ERC-20 approve（collateral token → vault）
2. vault.deposit(onBehalfOf, amount)

**步骤 1: ERC-20 Approve**
```
合约: <collateral_token_address> (e.g. wstETH: 0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0)
Function: approve(address,uint256)
Selector: 0x095ea7b3 [verified: cast sig "approve(address,uint256)"]
Calldata: 0x095ea7b3
         + spender (vault address, left-padded 32 bytes)
         + amount (uint256, 32 bytes)
onchainos: wallet contract-call --chain 1 --to <collateral_token> --input-data <calldata>
```

**步骤 2: Vault Deposit**
```
合约: <vault_address> (dynamic, from user input or vaults list)
Function: deposit(address,uint256)
Selector: 0x47e7ef24 [verified: cast sig "deposit(address,uint256)"]
Parameters:
  - onBehalfOf (address): user wallet address
  - amount (uint256): amount in wei (token units × 10^decimals)
Calldata: 0x47e7ef24
         + onBehalfOf (32 bytes, left-padded)
         + amount (32 bytes)
onchainos: wallet contract-call --chain 1 --to <vault_address> --input-data <calldata>
```

**注意：** vault 可能有 `depositLimit` — 在 design.md §2b 中通过 eth_call `isDepositLimit()` + `depositLimit()` 检查。

#### `withdraw` — 发起提款请求

**Symbiotic 提款是两阶段的：**
1. 调用 `withdraw(claimer, amount)` 发起提款请求（锁定到当前 epoch 结束）
2. epoch 结束后，调用 `claim(recipient, epoch)` 领取

**步骤 1: Vault Withdraw Request**
```
合约: <vault_address>
Function: withdraw(address,uint256)
Selector: 0xf3fef3a3 [verified: cast sig "withdraw(address,uint256)"]
Parameters:
  - claimer (address): user wallet address (who can claim after epoch)
  - amount (uint256): amount of underlying token to withdraw
Calldata: 0xf3fef3a3
         + claimer (32 bytes, left-padded)
         + amount (32 bytes)
onchainos: wallet contract-call --chain 1 --to <vault_address> --input-data <calldata>
```

**⚠️ 提款注意：** withdraw 请求提交后需等待 epoch 结束（通常 7 天），之后调用 `claim` 领取。

**可选：步骤 2: Claim（领取已解锁提款）**
```
合约: <vault_address>
Function: claim(address,uint256)
Selector: 0xaad3ec96 [verified: cast sig "claim(address,uint256)"]
Parameters:
  - recipient (address): 接收代币的地址
  - epoch (uint256): 要领取的 epoch 号
onchainos: wallet contract-call --chain 1 --to <vault_address> --input-data <calldata>
```

---

## §3 用户场景

### 场景 1：查询所有可用 Vault 和收益率

**用户说：** "Show me all Symbiotic vaults and their APR"

**动作序列：**
1. [链下查询] 调用 `GET https://app.symbiotic.fi/api/v2/vaults` 获取完整 vault 列表
2. 提取每个 vault 的 `meta.name`, `token.symbol`, `tvl`, `vaultRewardsApr`
3. 格式化输出，按 TVL 或 APR 排序显示

**预期输出：**
```
Symbiotic Vaults (Ethereum):
1. wstETH Vault (0xC329...) — TVL: $59.3M — APR: 2.37%
2. rETH Vault (0x03Bf...) — TVL: $19.1M — APR: 2.02%
3. cbETH Vault (0xB26f...) — TVL: $2.8M — APR: 2.80%
...
```

### 场景 2：查询用户的 Restaking 仓位

**用户说：** "What's my Symbiotic restaking position?"

**动作序列：**
1. [链下查询] `GET https://app.symbiotic.fi/api/v2/vaults?limit=100` 获取所有 vault 地址
2. [链上查询] 解析钱包地址：`onchainos wallet addresses` → chainIndex "1"
3. [链上查询] 对每个 vault，eth_call `activeBalanceOf(userAddr)` → `0x59f769a9`
4. 过滤出余额 > 0 的 vault
5. 对每个有余额的 vault，eth_call `collateral()` 获取底层代币地址，再查 token symbol
6. 格式化输出

### 场景 3：向 wstETH Vault 存款

**用户说：** "Deposit 0.01 wstETH into Symbiotic"

**动作序列：**
1. [链下查询] 从 vaults API 找到 wstETH vault 地址（0xC329400492c6ff2438472D4651Ad17389fCb843a）
2. [链下查询] 解析钱包地址：`onchainos wallet addresses` → chainIndex "1"
3. [dry-run] 显示将存入的 vault 名称、地址、金额 USD 价值
4. **请求用户确认** 后执行：
5. [链上写操作] ERC-20 approve: `wallet contract-call --chain 1 --to 0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0 --input-data 0x095ea7b3<vault_padded><amount_hex>`
6. [链上写操作] Vault deposit: `wallet contract-call --chain 1 --to 0xC329... --input-data 0x47e7ef24<wallet_padded><amount_hex>`
7. 返回 txHash

### 场景 4：从 Vault 发起提款

**用户说：** "Withdraw 0.01 wstETH from Symbiotic"

**动作序列：**
1. [链下查询] 解析钱包地址
2. [链下查询] 查询当前 vault 仓位（activeBalanceOf）
3. 检查余额是否充足
4. [dry-run] 显示提款金额，提醒用户提款需要等待 epoch 结束（约 7 天）
5. **请求用户确认** 后执行：
6. [链上写操作] `wallet contract-call --chain 1 --to <vault> --input-data 0xf3fef3a3<claimer_padded><amount_hex>`
7. 返回 txHash 和提示：提款请求已提交，需等 epoch 结束后调用 claim

---

## §4 外部 API 依赖

| API | 用途 | 认证 |
|-----|------|------|
| `https://app.symbiotic.fi/api/v2/vaults` | 获取 vault 列表、APR、TVL、代币信息 | 无需认证 |
| `https://ethereum.publicnode.com` | Ethereum mainnet RPC — eth_call (activeBalanceOf, collateral, currentEpoch) | 无需认证 |

---

## §5 配置参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `chain` | u64 | 1 | Ethereum mainnet |
| `dry_run` | bool | false | 仅模拟，不提交链上交易 |
| `vault` | String | 空（可选） | 指定 vault 地址（否则从 vault 名称/代币推断） |
| `token` | String | "wstETH" | 操作的代币符号（用于 vault 查找） |
| `amount` | String | 必填（写操作） | 存取金额（可读单位，如 "0.01"） |
| `limit` | u64 | 20 | vaults 命令返回条数 |

---

## §6 关键合约地址（已验证）

| 合约 | 地址 | 说明 |
|------|------|------|
| wstETH Vault | `0xC329400492c6ff2438472D4651Ad17389fCb843a` | 最大 wstETH vault ($59.3M TVL) — legacy |
| rETH Vault | `0x03Bf48b8a1B37FBeAd1EcAbcF15B98B924ffA5AC` | rETH vault ($19.1M TVL) |
| cbETH Vault | `0xB26ff591F44b04E78de18f43B46f8b70C6676984` | cbETH vault ($2.8M TVL) |
| wstETH Token | `0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0` | Lido wrapped stETH |
| OperatorRegistry | `0xAd817a6Bc954F678451A71363f04150FDD81Af9F` | Symbiotic operator registry |
| NetworkOptInService | `0x7133415b33B438843D581013f98A08704316633c` | Network opt-in service |

**注：** vault 地址在运行时动态从 API 获取，不硬编码（除主要 wstETH vault 作为默认值）。

---

## §7 函数选择器验证表

| 函数签名 | 选择器 | 验证方式 |
|---------|-------|---------|
| `deposit(address,uint256)` | `0x47e7ef24` | cast sig |
| `withdraw(address,uint256)` | `0xf3fef3a3` | cast sig |
| `redeem(address,uint256)` | `0x1e9a6950` | cast sig |
| `claim(address,uint256)` | `0xaad3ec96` | cast sig |
| `claimBatch(address,uint256[])` | `0x7c04c80a` | cast sig |
| `activeBalanceOf(address)` | `0x59f769a9` | cast sig |
| `withdrawalsOf(uint256,address)` | `0xf5e7ee0f` | cast sig |
| `slashableBalanceOf(address)` | `0xc31e8dd7` | cast sig |
| `totalStake()` | `0x8b0e9f3f` | cast sig |
| `collateral()` | `0xd8dfeb45` | cast sig |
| `currentEpoch()` | `0x76671808` | cast sig |
| `epochDuration()` | `0x4ff0876a` | cast sig |
| `approve(address,uint256)` | `0x095ea7b3` | cast sig |
