# Kamino Liquidity — Plugin Design

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `kamino-liquidity` |
| dapp_name | Kamino Liquidity |
| target_chains | Solana (501) |
| target_protocols | Kamino Finance — KVault earn vaults |
| category | defi-protocol |
| version | 0.1.0 |

---

## §1 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 无。官方 SDK 为 TypeScript：`@kamino-finance/kliquidity-sdk` |
| SDK 支持哪些技术栈？ | TypeScript/JavaScript only |
| 有 REST API？ | ✅ `https://api.kamino.finance` — 无需 API Key |
| 有官方 Skill？ | 无 |
| 开源社区有类似 Skill？ | 已有 `kamino-lend` 插件作为参考 |
| 支持哪些链？ | Solana mainnet only |
| 是否需要 onchainos 广播？ | ✅ 是 — deposit/withdraw 使用 `onchainos wallet contract-call --chain 501 --unsigned-tx <base58>` |

**接入路径：** API（REST API 返回完整 unsigned transaction，通过 onchainos 广播）

**架构说明：**
- Kamino Liquidity = KVault Earn Vaults，单币存入，程序 ID `KvauGMspG5k6rtzrqqn7WNn3oZdyKqLKwK2XWQ8FLjd`
- REST API 在 `/ktx/kvault/deposit` 和 `/ktx/kvault/withdraw` 返回 base64 编码序列化交易
- 数据查询在 `/kvaults/vaults` 和 `/kvaults/users/{wallet}/positions`

---

## §2 接口映射

### 需要接入的操作表

| 操作 | 类型 | 说明 |
|------|------|------|
| `vaults` | 链下查询 | 列出所有活跃 KVault，含 TVL/APY/tokenMint |
| `positions` | 链下查询 | 查询用户在所有 KVault 的持仓（shares balance） |
| `deposit` | 链上写操作 | 存入指定 token 到指定 KVault，获得 shares |
| `withdraw` | 链上写操作 | 赎回 shares，换回 token |

---

### §2a 链下查询表

#### `vaults` — 列出所有 KVault

- **API Endpoint：** `GET https://api.kamino.finance/kvaults/vaults`
- **参数：** 无（返回全部，约 115 个）
- **Response 实测结构：**
  ```json
  [
    {
      "address": "FWcZUkWPCSWjBH16nAYMQtJjHZuDmmtkd4KHnDTUc7su",
      "programId": "KvauGMspG5k6rtzrqqn7WNn3oZdyKqLKwK2XWQ8FLjd",
      "state": {
        "name": "Getfi",
        "tokenMint": "GSoLRcWKQE5nbWTYFr83Ei3HGjnp9YzQNAFK6VAATg3",
        "tokenMintDecimals": 9,
        "tokenVault": "FQ48mxAnfeo8dWQXiupUhEiauuPdcWcgfeBa2MtKgpYb",
        "sharesMint": "Gr8tUD3wugkUG5vkR2HSkmLowM7uhzfG43ZstPGHk1Za",
        "sharesMintDecimals": 9,
        "tokenAvailable": "1000",
        "sharesIssued": "1000",
        "performanceFeeBps": 0,
        "managementFeeBps": 0,
        "vaultAllocationStrategy": [...]
      }
    }
  ]
  ```
- **关键输出字段：** address、state.name、state.tokenMint、state.tokenMintDecimals、state.tokenAvailable、state.sharesIssued

#### `positions` — 用户持仓

- **API Endpoint：** `GET https://api.kamino.finance/kvaults/users/{wallet}/positions`
- **参数：** wallet (path param) — Solana 钱包 base58 地址
- **Response 实测结构：**
  ```json
  [
    {
      "vault": "<vault_address>",
      "sharesAmount": "1000.5",
      "tokenAmount": "1001.2"
    }
  ]
  ```
  （当前用户无持仓时返回 `[]`）

---

### §2b 链上写操作表

#### `deposit` — 存入 token 到 KVault

- **API Endpoint：** `POST https://api.kamino.finance/ktx/kvault/deposit`
- **Request Body（实测验证）：**
  ```json
  {
    "kvault": "<vault_address>",
    "wallet": "<user_pubkey_base58>",
    "amount": "0.001"   // UI 单位，API 内部转换为 lamports
  }
  ```
- **Response（实测）：**
  ```json
  { "transaction": "<base64_serialized_tx>" }
  ```
- **onchainos 命令：**
  ```bash
  onchainos wallet contract-call \
    --chain 501 \
    --to KvauGMspG5k6rtzrqqn7WNh3oZdyKqLKwK2XWQ8FLjd \
    --unsigned-tx <base58_tx> \
    --force
  ```
  > ⚠️ API 返回 base64，onchainos 要求 base58 → 代码中必须转换
  > ⚠️ amount 使用 UI 单位（0.001 SOL，不是 1000000 lamports）
  > ⚠️ --force 必须加，否则 txHash: "pending" 不广播

#### `withdraw` — 赎回 shares 换回 token

- **API Endpoint：** `POST https://api.kamino.finance/ktx/kvault/withdraw`
- **Request Body（实测验证）：**
  ```json
  {
    "kvault": "<vault_address>",
    "wallet": "<user_pubkey_base58>",
    "amount": "1"   // shares 数量，UI 单位
  }
  ```
- **Response（实测）：**
  ```json
  { "transaction": "<base64_serialized_tx>" }
  ```
- **onchainos 命令：**
  ```bash
  onchainos wallet contract-call \
    --chain 501 \
    --to KvauGMspG5k6rtzrqqn7WNh3oZdyKqLKwK2XWQ8FLjd \
    --unsigned-tx <base58_tx> \
    --force
  ```

---

## §3 用户场景

### 场景 1：查询可用的 LP Vaults

> 用户：「帮我查看 Kamino 上有哪些流动性 vault」

Agent 动作：
1. 调用 `./kamino-liquidity vaults --chain 501`
2. 发起 `GET /kvaults/vaults` 查询所有 vault
3. 按 tokenAvailable 排序，展示前 20 个
4. 输出：vault 地址、名称、token mint、可用 token 量

### 场景 2：查看我的 Kamino Liquidity 持仓

> 用户：「查看我在 Kamino Liquidity 的持仓」

Agent 动作：
1. 解析钱包地址：从 `onchainos wallet balance --chain 501` 提取 `data.details[0].tokenAssets[0].address`
2. 调用 `./kamino-liquidity positions --chain 501`
3. 发起 `GET /kvaults/users/{wallet}/positions`
4. 输出：每个持仓的 vault 地址、shares 数量、对应 token 数量

### 场景 3：存入 SOL 到 Kamino SOL vault

> 用户：「帮我往 Kamino 的 SOL vault 存入 0.001 SOL」

Agent 动作：
1. `./kamino-liquidity deposit --vault GEodMsAREMV4JdKs1yUCTKpz4EtzxKoSDeM3NZkG1RRk --amount 0.001 --chain 501 --dry-run` → 预览
2. **询问用户确认**
3. 检查 SOL 余额
4. 发起 `POST /ktx/kvault/deposit` 获取 base64 交易
5. base64 → base58 转换
6. `onchainos wallet contract-call --chain 501 --to KvauGMspG5k6rtzrqqn7WNh3oZdyKqLKwK2XWQ8FLjd --unsigned-tx <base58> --force`
7. 提取 txHash，返回 solscan.io 链接

### 场景 4：赎回 Kamino vault shares

> 用户：「把我在 Kamino SOL vault 里的 0.5 shares 取出来」

Agent 动作：
1. `./kamino-liquidity withdraw --vault GEodMsAREMV4JdKs1yUCTKpz4EtzxKoSDeM3NZkG1RRk --amount 0.5 --chain 501 --dry-run` → 预览
2. **询问用户确认**
3. 发起 `POST /ktx/kvault/withdraw` 获取 base64 交易
4. base64 → base58 转换
5. `onchainos wallet contract-call --chain 501 --to KvauGMspG5k6rtzrqqn7WNh3oZdyKqLKwK2XWQ8FLjd --unsigned-tx <base58> --force`
6. 提取 txHash，返回 solscan.io 链接

---

## §4 外部 API 依赖

| API | 用途 | 认证 |
|-----|------|------|
| `GET https://api.kamino.finance/kvaults/vaults` | 获取所有 KVault 列表 | 无 |
| `GET https://api.kamino.finance/kvaults/users/{wallet}/positions` | 获取用户持仓 | 无 |
| `POST https://api.kamino.finance/ktx/kvault/deposit` | 构建存款交易 | 无 |
| `POST https://api.kamino.finance/ktx/kvault/withdraw` | 构建取款交易 | 无 |

---

## §5 配置参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `--chain` | u64 | 501 | 链 ID（仅支持 501） |
| `--vault` | String | — | KVault 地址（base58），deposit/withdraw 必填 |
| `--amount` | f64 | — | 存款/取款金额（UI 单位，如 0.001 SOL = 0.001） |
| `--dry-run` | bool | false | 模拟模式，不广播交易 |

---

## §6 关键常量

```
KVAULT_PROGRAM_ID = "KvauGMspG5k6rtzrqqn7WNh3oZdyKqLKwK2XWQ8FLjd"
KAMINO_API_BASE   = "https://api.kamino.finance"
SOLANA_CHAIN_ID   = 501
```

---

## §7 实测 API 验证记录

| Endpoint | Method | Status | 备注 |
|----------|--------|--------|------|
| `/kvaults/vaults` | GET | ✅ 200 | 返回约 115 个 vault 列表 |
| `/kvaults/users/{wallet}/positions` | GET | ✅ 200 | 无持仓时返回 `[]` |
| `/ktx/kvault/deposit` | POST | ✅ 200 | 返回 `{transaction: "<base64>"}` |
| `/ktx/kvault/withdraw` | POST | ✅ 200 | 返回 `{transaction: "<base64>"}` |

**Request body 字段确认：**
- deposit: `kvault` (not `vault`), `wallet` (not `owner`), `amount` (not `depositAmount` / `sharesAmount`)
- withdraw: `kvault`, `wallet`, `amount`（shares 数量）
