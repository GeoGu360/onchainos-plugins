# Plugin PRD — Instadapp

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `instadapp` |
| dapp_name | Instadapp |
| target_chains | Ethereum (1) |
| target_protocols | Instadapp Lite ETH vault v1 (iETH), Instadapp Lite ETH v2 vault (iETHv2) |
| category | defi-protocol |
| tags | yield, vault, eth, steth, leverage, instadapp, lite |
| version | 0.1.0 |
| author | GeoGu360 |

---

## §1 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 无官方 Rust SDK |
| SDK 支持哪些技术栈？ | JavaScript/TypeScript (dsa-connect), Python (unofficial) |
| 有 REST API？ | ⚠️ API endpoint `https://api.instadapp.io/v2/mainnet/lite/vaults` exists but currently broken (returns 400 error due to self-referential call bug). Plugin will use direct on-chain RPC instead. |
| 有官方 Skill？ | 无 |
| 开源社区有类似 Skill？ | 无直接 Skill |
| 支持哪些链？ | Ethereum (1), Polygon (137), Arbitrum (42161) — Lite vaults are Ethereum-only |
| 是否需要 onchainos 广播？ | ✅ Yes — deposit/withdraw are on-chain write operations via supplyEth()/withdraw() for v1 or ERC-4626 deposit/redeem for v2 |

**接入路径：API (on-chain RPC)** — Since the REST API is broken, Rust will use direct Ethereum JSON-RPC (`eth_call`) for all read operations. Chain writes go through onchainos CLI.

**Focus:** Instadapp Lite ETH v1 vault (iETH) — simpler interface, accepts native ETH via `supplyEth()`. This is the primary product. We will also support iETHv2 vault for read operations.

---

## §2 接口映射

### 需要接入的操作

| # | 操作 | 类型 | 优先级 |
|---|------|------|--------|
| 1 | `vaults` — 列出 Instadapp Lite vaults 及其 APY/TVL 信息 | 链下查询 | P0 |
| 2 | `positions` — 查询用户在各 vault 的持仓余额 | 链下查询 | P0 |
| 3 | `rates` — 查询各 vault 的 exchange price / 收益 | 链下查询 | P0 |
| 4 | `deposit` — 向 iETH vault 存入 ETH (v1: supplyEth; v2: approve+deposit) | 链上写操作 | P0 |
| 5 | `withdraw` — 从 iETH vault 赎回 ETH | 链上写操作 | P0 |

---

### 链下查询表

#### 1. vaults — 列出活跃 Lite vaults

- **实现**: 直接通过 JSON-RPC `eth_call` 查询两个已知 vault 合约
- **RPC**: `https://ethereum.publicnode.com`
- **Vault 地址**:
  - iETH v1: `0xc383a3833A87009fD9597F8184979AF5eDFad019` (symbol: "iETH")
  - iETH v2: `0xa0d3707c569ff8c87fa923d3823ec5d81c98be78` (symbol: "iETHv2")
- **eth_call 查询每个 vault**:
  - `totalSupply()` selector `0x18160ddd` → total shares
  - `getCurrentExchangePrice()` selector `0xcc4a0158` → (exchangePrice, newRevenue) for v1
  - `exchangePrice()` selector `0x9e65741e` → exchangePrice for v2
  - `totalAssets()` selector `0x01e1d114` → v2 total assets
  - `netAssets()` selector `0x0782d421` → (netCollateral, netBorrow, balances, netSupply, netBal) for v1

- **返回值示例**:
  ```json
  {
    "vaults": [
      {
        "address": "0xc383a3833A87009fD9597F8184979AF5eDFad019",
        "name": "Instadapp ETH",
        "symbol": "iETH",
        "version": "v1",
        "underlying": "ETH",
        "exchange_price": "1.200478",
        "total_supply": "54.7737",
        "net_collateral_eth": "79.61",
        "net_borrow_eth": "0.0019"
      }
    ]
  }
  ```

#### 2. positions — 用户持仓

- **实现**: `eth_call balanceOf(address)` on each vault
- **selector**: `0x70a08231`
- **计算**: `underlying_balance = shares * exchangePrice / 1e18`

#### 3. rates — 收益率

- **实现**: 从 `getCurrentExchangePrice()` / `exchangePrice()` 推断 APR
  - exchange_price 从 1e18 开始，>1 表示已累积收益
  - 当前约 1.2 → ~20% 总收益率（无法直接获取 APR，展示 exchange price）
  - 也从 `netAssets()` 展示杠杆倍数

---

### 链上写操作表

#### 4a. deposit (ETH) — iETH v1 vault

**函数**: `supplyEth(address to_)` — 直接发送 ETH，铸造 iETH
- **合约地址**: `0xc383a3833A87009fD9597F8184979AF5eDFad019` (硬编码 v1 vault)
- **函数签名**: `supplyEth(address)`
- **Selector**: `0x87ee9312` ✅ (verified: `cast sig "supplyEth(address)"`)
- **ABI 编码**:
  ```
  0x87ee9312
  + pad32(receiver_address)   // user's wallet address
  ```
- **ETH value**: `--amt <wei>` — amount of ETH to deposit
- **onchainos 命令**:
  ```bash
  onchainos wallet contract-call --chain 1 \
    --to 0xc383a3833A87009fD9597F8184979AF5eDFad019 \
    --input-data 0x87ee9312<receiver_padded> \
    --amt <wei_amount>
  ```

#### 4b. deposit (stETH) — iETH v2 vault (ERC-4626)

**流程**: ERC-20 approve stETH → ERC-4626 deposit (with 3s delay)

**Step 4b-1: approve stETH**
- **合约**: stETH at `0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84`
- **Selector**: `0x095ea7b3` ✅ (`cast sig "approve(address,uint256)"`)
- **spender**: iETHv2 vault `0xa0d3707c569ff8c87fa923d3823ec5d81c98be78`
- **onchainos 命令**:
  ```bash
  onchainos wallet contract-call --chain 1 \
    --to 0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84 \
    --input-data 0x095ea7b3<vault_addr_padded><amount_padded>
  ```

**Step 4b-2: deposit stETH (after 3s delay)**
- **合约**: iETHv2 vault `0xa0d3707c569ff8c87fa923d3823ec5d81c98be78`
- **函数**: `deposit(uint256 assets_, address receiver_)`
- **Selector**: `0x6e553f65` ✅ (`cast sig "deposit(uint256,address)"`)
- **ABI 编码**:
  ```
  0x6e553f65
  + pad32(amount_in_wei)   // stETH amount (18 decimals)
  + pad32(receiver)        // user wallet address
  ```
- **onchainos 命令**:
  ```bash
  onchainos wallet contract-call --chain 1 \
    --to 0xa0d3707c569ff8c87fa923d3823ec5d81c98be78 \
    --input-data 0x6e553f65<amount_padded><receiver_padded>
  ```

#### 5. withdraw — from iETH v1 vault

**函数**: `withdraw(uint256 amount_, address to_)` — burns iETH shares, returns ETH
- **合约地址**: `0xc383a3833A87009fD9597F8184979AF5eDFad019`
- **函数签名**: `withdraw(uint256,address)`
- **Selector**: `0x00f714ce` ✅ (verified: `cast sig "withdraw(uint256,address)"`)
- **ABI 编码**:
  ```
  0x00f714ce
  + pad32(amount_shares)   // iETH shares to burn (18 decimals)
  + pad32(receiver)        // destination address
  ```
- **onchainos 命令**:
  ```bash
  onchainos wallet contract-call --chain 1 \
    --to 0xc383a3833A87009fD9597F8184979AF5eDFad019 \
    --input-data 0x00f714ce<amount_padded><receiver_padded>
  ```

#### 5b. redeem — from iETH v2 vault (ERC-4626)

**函数**: `redeem(uint256 shares_, address receiver_, address owner_)`
- **Selector**: `0xba087652` ✅ (`cast sig "redeem(uint256,address,address)"`)
- **合约**: iETHv2 vault `0xa0d3707c569ff8c87fa923d3823ec5d81c98be78`
- **onchainos 命令**:
  ```bash
  onchainos wallet contract-call --chain 1 \
    --to 0xa0d3707c569ff8c87fa923d3823ec5d81c98be78 \
    --input-data 0xba087652<shares_padded><receiver_padded><owner_padded>
  ```

---

### Function Selector 核对清单

| 函数签名 | cast sig 结果 | 用途 |
|---------|-------------|------|
| `supplyEth(address)` | `0x87ee9312` | iETH v1 — deposit ETH |
| `supply(address,uint256,address)` | `0x8b2a4df5` | iETH v1 — deposit WETH/stETH |
| `withdraw(uint256,address)` | `0x00f714ce` | iETH v1 — withdraw iETH shares |
| `getCurrentExchangePrice()` | `0xcc4a0158` | iETH v1 — get exchange price |
| `netAssets()` | `0x0782d421` | iETH v1 — get vault position |
| `approve(address,uint256)` | `0x095ea7b3` | ERC-20 approve |
| `deposit(uint256,address)` | `0x6e553f65` | iETHv2 — ERC-4626 deposit |
| `redeem(uint256,address,address)` | `0xba087652` | iETHv2 — ERC-4626 redeem |
| `exchangePrice()` | `0x9e65741e` | iETHv2 — get exchange price |
| `getNetAssets()` | `0x08bb5fb0` | iETHv2 — comprehensive net assets |
| `totalAssets()` | `0x01e1d114` | iETHv2 — total assets in vault |
| `totalSupply()` | `0x18160ddd` | ERC-20 totalSupply |
| `balanceOf(address)` | `0x70a08231` | ERC-20 balanceOf |
| `asset()` | `0x38d52e0f` | iETHv2 — underlying asset address |

---

## §3 用户场景

### 场景 1: 查看 Instadapp Lite vaults 信息

**用户对 Agent 说**: "Show me Instadapp Lite vaults and their current yields"

**Agent 动作序列**:
1. 链下查询: eth_call `getCurrentExchangePrice()` on iETH v1 vault
2. 链下查询: eth_call `totalSupply()` on iETH v1 vault
3. 链下查询: eth_call `netAssets()` on iETH v1 vault → 获取 TVL
4. 链下查询: eth_call `exchangePrice()` on iETHv2 vault
5. 链下查询: eth_call `totalAssets()` on iETHv2 vault
6. 汇总输出两个 vault 的信息

**预期输出**:
```json
{
  "ok": true,
  "data": {
    "chain_id": 1,
    "vaults": [
      {
        "address": "0xc383a3833A87009fD9597F8184979AF5eDFad019",
        "name": "Instadapp ETH",
        "symbol": "iETH",
        "version": "v1",
        "underlying": "ETH",
        "exchange_price_eth": "1.200478",
        "total_supply": "54.7737",
        "deposit_token": "ETH (native)"
      },
      {
        "address": "0xa0d3707c569ff8c87fa923d3823ec5d81c98be78",
        "name": "Instadapp ETH v2",
        "symbol": "iETHv2",
        "version": "v2",
        "underlying": "stETH",
        "exchange_price_steth": "1.205798",
        "deposit_token": "stETH"
      }
    ]
  }
}
```

---

### 场景 2: 查询用户在 Instadapp Lite 的持仓

**用户对 Agent 说**: "What are my Instadapp Lite positions on Ethereum?"

**Agent 动作序列**:
1. 解析钱包: `onchainos wallet addresses` → 提取 chainIndex "1" 对应地址
2. 链下查询: eth_call `balanceOf(user)` on iETH v1 vault
3. 链下查询: eth_call `balanceOf(user)` on iETHv2 vault
4. 链下查询: eth_call `getCurrentExchangePrice()` on iETH v1 → 计算 ETH 价值
5. 链下查询: eth_call `exchangePrice()` on iETHv2 → 计算 stETH 价值
6. 输出非零持仓

---

### 场景 3: 存入 ETH 到 Instadapp iETH vault

**用户对 Agent 说**: "Deposit 0.0001 ETH into Instadapp Lite iETH vault"

**Agent 动作序列**:
1. 解析钱包: `onchainos wallet addresses` → chainIndex "1" → user_wallet
2. 计算 wei: 0.0001 ETH = 100000000000000 wei
3. Dry-run 检查 (if --dry-run): 返回模拟响应，展示 calldata
4. **告知用户操作详情，请求用户确认** (deposit ETH into iETH vault)
5. 链上写操作: supplyEth(user_wallet) with --amt 100000000000000:
   ```
   onchainos wallet contract-call --chain 1 \
     --to 0xc383a3833A87009fD9597F8184979AF5eDFad019 \
     --input-data 0x87ee9312<user_wallet_padded> \
     --amt 100000000000000
   ```
6. 提取 txHash，输出结果

---

### 场景 4: 从 Instadapp iETH vault 赎回

**用户对 Agent 说**: "Withdraw my iETH from Instadapp Lite vault"

**Agent 动作序列**:
1. 解析钱包: `onchainos wallet addresses` → user_wallet
2. 链下查询: eth_call `balanceOf(user)` on iETH v1 → shares 余额
3. 如果 shares == 0: 返回 "No iETH position found"
4. 链下查询: eth_call `getCurrentExchangePrice()` → 计算 ETH 价值
5. Dry-run 检查 (if --dry-run): 返回模拟响应
6. **请求用户确认** 赎回数量
7. 链上写操作: withdraw(shares, user_wallet)
   ```
   onchainos wallet contract-call --chain 1 \
     --to 0xc383a3833A87009fD9597F8184979AF5eDFad019 \
     --input-data 0x00f714ce<shares_padded><user_wallet_padded>
   ```
8. 提取 txHash，输出结果

---

### 场景 5: 查看当前 exchange price 和收益

**用户对 Agent 说**: "What's the current APY for Instadapp Lite ETH vault?"

**Agent 动作序列**:
1. 链下查询: eth_call `getCurrentExchangePrice()` on iETH v1 vault
2. 展示 exchange price (当前 ~1.2 ETH per iETH = ~20% 总收益)
3. eth_call `netAssets()` → 展示 TVL, 杠杆信息
4. 输出完整的收益报告

---

## §4 外部 API 依赖

| API | 用途 | 认证 |
|-----|------|------|
| `https://ethereum.publicnode.com` | Ethereum mainnet JSON-RPC (eth_call) | 无需认证 |

注意: `https://api.instadapp.io/v2/mainnet/lite/vaults` 目前返回 400 错误，已放弃使用。

---

## §5 配置参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `chain` | u64 | 1 | Ethereum mainnet chain ID |
| `dry_run` | bool | false | 模拟运行，不发链上交易 |
| `vault` | Option\<String\> | None | 指定 vault: "v1"/"iETH" 或 "v2"/"iETHv2" (默认 v1) |
| `amount` | Option\<String\> | None | ETH 金额 (e.g. "0.0001") |
| `shares` | Option\<String\> | None | iETH shares 数量 (赎回时用) |
| `all` | bool | false | withdraw: 赎回全部 shares |

---

## §6 关键合约地址 (Ethereum Mainnet)

| 合约 | 地址 | 说明 |
|------|------|------|
| iETH vault v1 | `0xc383a3833A87009fD9597F8184979AF5eDFad019` | Instadapp Lite ETH v1 — accepts native ETH via supplyEth() |
| iETH vault v2 | `0xa0d3707c569ff8c87fa923d3823ec5d81c98be78` | Instadapp Lite ETH v2 — ERC-4626, accepts stETH |
| InstaVaultResolver | `0x739C30f2aF180b43aca064Ff08eB704639D21Cce` | Resolver for vault info (partial, v1 only) |
| stETH | `0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84` | Lido stETH (underlying for v2) |
| WETH | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` | Wrapped Ether |

**注意**: 
- iETH v1: 用 `supplyEth(address)` 存入 native ETH, 用 `withdraw(uint256,address)` 赎回
- iETH v2: ERC-4626 标准，存入 stETH, 用 `deposit(uint256,address)` 和 `redeem(uint256,address,address)` 操作
- 测试 (L4) 优先使用 iETH v1 (ETH deposit)，因为 iETH v2 requires stETH (no test wallet stETH)
- GUARDRAILS: 最小测试金额 0.00005 ETH (50000000000000 wei)
