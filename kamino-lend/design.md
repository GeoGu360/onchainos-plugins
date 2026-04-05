# Kamino Lend Plugin PRD

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| `plugin_name` | `kamino-lend` |
| `dapp_name` | Kamino Lend |
| `target_chains` | Solana (501) |
| `target_protocols` | Lending |
| `version` | 0.1.0 |
| `category` | defi-protocol |

---

## §1 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 无官方 Rust SDK。有 TypeScript SDK：https://github.com/Kamino-Finance/klend-sdk |
| SDK 支持哪些技术栈？ | TypeScript (官方)；无 Rust SDK |
| 有 REST API？ | ✅ 有：https://api.kamino.finance（无需 API Key） |
| 有官方 Skill？ | 无 |
| 开源社区有类似 Skill？ | 无已知社区 Rust/Plugin-Store Skill |
| 支持哪些链？ | Solana mainnet 501 |
| 是否需要 onchainos 广播？ | Yes — 所有链上写操作（supply/withdraw/borrow/repay）走 onchainos wallet contract-call --chain 501 --unsigned-tx |

**接入路径：** API（REST API，Rust reqwest 调用）

---

## §2 接口映射

### 2a. 操作列表

| # | 操作 | 类型 | 说明 |
|---|------|------|------|
| 1 | `markets` | 链下查询 | 列出所有 Kamino 借贷市场 |
| 2 | `positions` | 链下查询 | 查询用户当前持仓/义务(obligation) |
| 3 | `supply` | 链上写操作 | 存入资产到借贷市场 |
| 4 | `withdraw` | 链上写操作（dry-run 支持） | 从借贷市场提取资产 |
| 5 | `borrow` | 链上写操作（dry-run only） | 借出资产 |
| 6 | `repay` | 链上写操作（dry-run only） | 偿还借款 |

### 2b. 链下查询接口

**markets — 获取所有借贷市场**

```
GET https://api.kamino.finance/v2/kamino-market
```
- 参数：无
- 返回字段（数组）：
  - `lendingMarket` — 市场公钥（base58）
  - `name` — 市场名称（如 "Main Market", "JLP Market"）
  - `description` — 描述
  - `isPrimary` — 是否主市场
  - `lookupTable` — Address Lookup Table

**单市场储备详细指标（用于展示 APY、TVL、symbol 等）**

```
GET https://api.kamino.finance/kamino-market/{marketPubkey}/reserves/{reservePubkey}/metrics/history
```
- Query 参数：
  - `env=mainnet-beta`
  - `start`, `end`（ISO 8601 时间戳）
  - `frequency=DAILY`（可选）
- 返回：`{ reserve: str, history: [{ timestamp, metrics: { symbol, supplyInterestAPY, borrowInterestAPY, depositTvl, borrowTvl, mintAddress, totalLiquidity, loanToValue, ... } }] }`

**已知主市场储备地址（主市场：7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF）：**

| Symbol | Reserve Address |
|--------|----------------|
| USDC | `D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59` |
| SOL | `d4A2prbA2whesmvHaL88BH6Ewn5N4bTSU2Ze8P6Bc4Q` |

**positions — 查询用户持仓**

```
GET https://api.kamino.finance/kamino-market/{marketPubkey}/users/{userPubkey}/obligations
```
- 参数：
  - `marketPubkey` — 市场公钥
  - `userPubkey` — 用户钱包地址（base58）
- 返回：义务(obligation)数组，包含：
  - `obligationAddress` — 义务账户地址
  - `deposits` — 存款列表（`{ reserveAddress, amount, symbol }`）
  - `borrows` — 借款列表（`{ reserveAddress, amount, symbol }`）
  - `healthFactor` — 健康因子
  - `netAccountValue` — 净账户价值
  - `loanToValue` — 当前 LTV

### 2c. 链上写操作

所有写操作通过 Kamino REST API 获取 unsigned serialized transaction，然后通过 `onchainos wallet contract-call --chain 501` 广播。

**⚠️ 关键：API 返回 base64 编码的序列化交易（`data.transaction` 字段），必须转换为 base58 后才能传给 onchainos `--unsigned-tx` 参数。**

**⚠️ Amount 格式：UI 单位（human-readable）。示例：0.5 USDC 传 `"amount": "0.5"`，不是 500000。**

**supply（存款）**

```
POST https://api.kamino.finance/ktx/klend/deposit
Body: {
  "wallet": "<user_pubkey>",
  "market": "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF",
  "reserve": "<reserve_pubkey>",
  "amount": "<ui_amount_str>"  // e.g. "0.01"
}
Response: { "transaction": "<base64_serialized_tx>" }
```

onchainos 命令：
```bash
onchainos wallet contract-call \
  --chain 501 \
  --to KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD \
  --unsigned-tx <base58_tx> \
  --force
```

**withdraw（提款）**

```
POST https://api.kamino.finance/ktx/klend/withdraw
Body: {
  "wallet": "<user_pubkey>",
  "market": "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF",
  "reserve": "<reserve_pubkey>",
  "amount": "<ui_amount_str>"
}
Response: { "transaction": "<base64_serialized_tx>" }
```

**borrow（借款，dry-run only）**

```
POST https://api.kamino.finance/ktx/klend/borrow
Body: {
  "wallet": "<user_pubkey>",
  "market": "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF",
  "reserve": "<reserve_pubkey>",
  "amount": "<ui_amount_str>"
}
Response: { "transaction": "<base64_serialized_tx>" }
Note: Requires prior deposit (obligation must exist)
```

**repay（还款，dry-run only）**

```
POST https://api.kamino.finance/ktx/klend/repay
Body: {
  "wallet": "<user_pubkey>",
  "market": "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF",
  "reserve": "<reserve_pubkey>",
  "amount": "<ui_amount_str>"
}
Response: { "transaction": "<base64_serialized_tx>" }
```

**base64 → base58 转换（必须）：**

```rust
fn base64_to_base58(b64: &str) -> anyhow::Result<String> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let bytes = STANDARD.decode(b64.trim())?;
    Ok(bs58::encode(bytes).into_string())
}
```

---

## §3 用户场景

### 场景 1：查看可用借贷市场和利率

**用户说：** "Show me Kamino lending markets and their interest rates"

**Agent 动作序列：**
1. [链下查询] 调用 `kamino-lend markets` 命令
2. Binary 调用 `GET /v2/kamino-market` 获取所有市场列表
3. 对主市场（Main Market）中的关键储备（USDC、SOL），调用储备指标端点获取最新 APY
4. 返回 JSON：`{ markets: [{ name, market_pubkey, supply_apy, borrow_apy, deposit_tvl }] }`

### 场景 2：查询我的借贷持仓

**用户说：** "What are my current lending positions on Kamino?"

**Agent 动作序列：**
1. 从 onchainos 解析 Solana 钱包地址：`onchainos wallet balance --chain 501`，提取 `data.details[0].tokenAssets[0].address`
2. [链下查询] 调用 `kamino-lend positions`
3. Binary 调用 `GET /kamino-market/7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF/users/{wallet}/obligations`
4. 返回 JSON：`{ obligations: [{ deposits: [], borrows: [], health_factor, net_value }] }`

### 场景 3：存入 USDC 赚取收益

**用户说：** "Supply 0.01 USDC to Kamino lending"

**Agent 动作序列：**
1. 解析钱包地址：`onchainos wallet balance --chain 501`
2. 向用户展示当前 USDC supply APY，**请求用户确认**操作
3. [链上操作] 调用 `kamino-lend supply --token USDC --amount 0.01 --chain 501`
4. Binary 调用 `POST /ktx/klend/deposit`，body `{ wallet, market, reserve: "D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59", amount: "0.01" }`
5. API 返回 `{ transaction: "<base64>" }` → binary 做 base64→base58 转换
6. 调用 `onchainos wallet contract-call --chain 501 --to KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD --unsigned-tx <base58> --force`
7. 解析响应中的 txHash，返回到用户

### 场景 4：借款（dry-run 演示）

**用户说：** "I want to borrow 0.001 SOL from Kamino (dry-run)"

**Agent 动作序列：**
1. 解析钱包地址
2. [dry-run] 调用 `kamino-lend borrow --token SOL --amount 0.001 --chain 501 --dry-run`
3. Binary 在 dry-run 模式下返回模拟响应，不调用 API 和 onchainos
4. 返回：`{ ok: true, dry_run: true, data: { txHash: "" }, note: "Borrow requires prior supply as collateral" }`

### 场景 5：偿还借款（dry-run）

**用户说：** "Repay 0.001 SOL borrow on Kamino (dry-run)"

**Agent 动作序列：**
1. 解析钱包地址
2. [dry-run] 调用 `kamino-lend repay --token SOL --amount 0.001 --chain 501 --dry-run`
3. Binary 在 dry-run 模式下直接返回模拟响应
4. 返回：`{ ok: true, dry_run: true, data: { txHash: "" } }`

---

## §4 外部 API 依赖

| API | 用途 |
|-----|------|
| `https://api.kamino.finance` | 主 REST API：市场数据、持仓查询、交易构造 |

具体端点：
- `GET https://api.kamino.finance/v2/kamino-market`
- `GET https://api.kamino.finance/kamino-market/{market}/reserves/{reserve}/metrics/history`
- `GET https://api.kamino.finance/kamino-market/{market}/users/{wallet}/obligations`
- `POST https://api.kamino.finance/ktx/klend/deposit`
- `POST https://api.kamino.finance/ktx/klend/withdraw`
- `POST https://api.kamino.finance/ktx/klend/borrow`
- `POST https://api.kamino.finance/ktx/klend/repay`

---

## §5 配置参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `chain` | `501` | Solana mainnet |
| `market` | `7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF` | Kamino 主市场 |
| `dry_run` | `false` | 为 true 时不提交链上交易 |
| `api_url` | `https://api.kamino.finance` | Kamino REST API base URL |

**已知储备地址（主市场）：**

| Token | Reserve Address | Mint |
|-------|----------------|------|
| USDC | `D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59` | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` |
| SOL | `d4A2prbA2whesmvHaL88BH6Ewn5N4bTSU2Ze8P6Bc4Q` | native (11111111111111111111111111111111) |

**程序地址：**
- Kamino Lend Program: `KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD`
