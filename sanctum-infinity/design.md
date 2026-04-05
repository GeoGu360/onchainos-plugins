# Sanctum Infinity — Plugin Design Doc

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `sanctum-infinity` |
| dapp_name | Sanctum Infinity |
| target_chains | Solana (501) |
| target_protocols | Sanctum Infinity (INF) LST Pool |
| version | 0.1.0 |
| category | defi-protocol |

---

## §1 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | No — igneous-labs/inf-1.5 is a program repo; no client SDK. |
| SDK 支持哪些技术栈？ | TypeScript (@sanctum-inf/sdk) — not Rust |
| 有 REST API？ | **Yes** — `https://sanctum-s-api.fly.dev` (router/swap) + `https://extra-api.sanctum.so` (info) |
| 有官方 Skill？ | No |
| 开源社区有类似 Skill？ | No known plugin-store skill |
| 支持哪些链？ | Solana only (chain 501) |
| 是否需要 onchainos 广播？ | Yes — swap and liquidity ops return serialized tx (base64) → `onchainos wallet contract-call --unsigned-tx` |

**接入路径**: API (REST) — no Rust SDK exists; use HTTP client to call Sanctum S API and Extra API.

---

## §2 接口映射

### 操作列表

| 操作 | 类型 | 链 |
|------|------|----|
| `pools` — 查询 INF 池信息及 LST 分配 | 链下读取 | Solana |
| `quote` — 获取 LST→LST 兑换报价 | 链下读取 | Solana |
| `swap` — 执行 LST→LST 兑换 | 链上写操作 | Solana |
| `deposit` — 向 INF 池存入 LST 赚取 fee | 链上写操作 | Solana |
| `withdraw` — 从 INF 池取出 LST | 链上写操作 | Solana |
| `positions` — 查询用户 INF 持仓 | 链下读取 | Solana |

---

### 链下查询表

#### pools — 查询 INF 池信息

**API**: `GET https://extra-api.sanctum.so/v1/infinity/allocation/current`

Response shape:
```json
{
  "infinity": {
    "<lst_mint>": {
      "amt": "1000000000000",     // U64Str — raw atomics
      "solValue": "1050000000000", // U64Str — lamports
      "share": 0.15               // float, 0.0-1.0
    }
  }
}
```

Also calls `GET https://extra-api.sanctum.so/v1/apy/latest?lst=INF` for APY.
Also calls `GET https://extra-api.sanctum.so/v1/sol-value/current?lst=INF` for NAV.

#### quote — 获取交换报价

**API**: `GET https://sanctum-s-api.fly.dev/v2/swap/quote`

Parameters:
- `input` (query): input LST mint (B58Pubkey) or wSOL mint for SOL
- `outputLstMint` (query): output LST mint (B58Pubkey)
- `amount` (query): U64Str — raw atomics (lamports for SOL)
- `mode` (query): "ExactIn" | "ExactOut" (default ExactIn)

Response:
```json
{
  "inAmount": "1000000000",
  "outAmount": "998500000",
  "swapSrc": "SPool",
  "fees": [
    { "code": "S_POOL_REMOVE_LIQUIDITY", "rate": "0.001", "amt": "1000000", "mint": "<mint>" }
  ]
}
```

#### positions — 查询用户 INF 持仓

Reads token account for INF mint from wallet, combined with SOL value.

**Wallet balance**: `onchainos wallet balance --chain 501`
Looks for INF mint `5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm` in tokenAssets.

Also calls sol-value to get INF→SOL rate for USD display.

---

### 链上写操作表

#### swap — LST→LST 兑换

**API**: `POST https://sanctum-s-api.fly.dev/v1/swap`

Request body:
```json
{
  "input": "<input_lst_mint>",           // B58Pubkey
  "outputLstMint": "<output_lst_mint>",  // B58Pubkey
  "amount": "<amount_atomics>",          // U64Str — raw atomics
  "quotedAmount": "<min_out_atomics>",   // U64Str — slippage threshold (from quote outAmount * (1-slippage))
  "mode": "ExactIn",                     // SwapMode
  "signer": "<wallet_pubkey>",           // B58Pubkey
  "swapSrc": "SPool"                     // force Infinity pool routing
}
```

Response:
```json
{
  "tx": "<base64_versioned_transaction>"
}
```

**onchainos**: `onchainos wallet contract-call --chain 501 --to 5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx --unsigned-tx <base58_tx> --force`

Note: base64 → base58 conversion required before passing to onchainos.
`to` = INF pool program ID: `5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx`

#### deposit — 向 Infinity 池存入 LST

**API**: `POST https://sanctum-s-api.fly.dev/v1/liquidity/add`

Request body:
```json
{
  "lstMint": "<lst_mint>",            // B58Pubkey — which LST to deposit
  "amount": "<amount_atomics>",       // U64Str — raw atomics
  "quotedAmount": "<min_lp_atomics>", // U64Str — minimum LP tokens expected (from quote)
  "signer": "<wallet_pubkey>"         // B58Pubkey
}
```

Quote first: `GET https://sanctum-s-api.fly.dev/v1/liquidity/add/quote?lstMint=<mint>&amount=<amount>`
→ `{ "lpAmount": "<expected_lp>" }`

Response: `{ "tx": "<base64_tx>" }`

**onchainos**: same pattern — base64→base58, `--to 5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx --force`

#### withdraw — 从 Infinity 池取出 LST

**API**: `POST https://sanctum-s-api.fly.dev/v1/liquidity/remove`

Request body:
```json
{
  "lstMint": "<lst_mint>",               // B58Pubkey — which LST to receive
  "amount": "<lp_amount_atomics>",       // U64Str — INF/LP tokens to burn
  "quotedAmount": "<min_lst_atomics>",   // U64Str — minimum LST expected
  "signer": "<wallet_pubkey>"
}
```

Quote first: `GET https://sanctum-s-api.fly.dev/v1/liquidity/remove/quote?lstMint=<mint>&amount=<lp_amount>`
→ `{ "lstAmount": "<expected_lst>" }`

Response: `{ "tx": "<base64_tx>" }`

---

## §3 用户场景

### 场景 1: 查询 Infinity 池当前状态

**用户**: "Show me the Sanctum Infinity pool stats"

**动作序列**:
1. [链下] `GET /v1/infinity/allocation/current` → 获取各 LST 分配
2. [链下] `GET /v1/sol-value/current?lst=INF` → 获取 INF NAV (lamports per INF)
3. [链下] `GET /v1/apy/latest?lst=INF` → 获取 APY
4. 输出：总 TVL, APY, 各 LST 持仓比例

### 场景 2: 将 JitoSOL 兑换为 INF

**用户**: "Swap 0.005 JitoSOL to INF using Sanctum Infinity"

**动作序列**:
1. [链下] `GET /v2/swap/quote?input=<jitoSOL_mint>&outputLstMint=<INF_mint>&amount=5000000` → 获取报价
2. 显示报价给用户，**询问用户确认**
3. [链下] `resolve_wallet_solana()` → 获取钱包地址
4. [链上] `POST /v1/swap` with signer + quotedAmount (with slippage) → 获取 base64 tx
5. base64 → base58
6. `onchainos wallet contract-call --chain 501 --to <program_id> --unsigned-tx <base58> --force`
7. 输出 txHash

### 场景 3: 向 Infinity 池存入 JitoSOL

**用户**: "Deposit 0.005 JitoSOL to Sanctum Infinity pool"

**动作序列**:
1. [链下] `GET /v1/liquidity/add/quote?lstMint=<jitoSOL>&amount=5000000` → 获取预期 LP 数量
2. 显示报价，**询问用户确认**
3. [链下] `resolve_wallet_solana()` → 获取钱包地址
4. [链上] `POST /v1/liquidity/add` with signer + quotedAmount → 获取 base64 tx
5. base64 → base58
6. `onchainos wallet contract-call --chain 501 --to <program_id> --unsigned-tx <base58> --force`
7. 输出 txHash

### 场景 4: 从 Infinity 池取回 LST

**用户**: "Withdraw 0.001 INF from Sanctum Infinity as JitoSOL"

**动作序列**:
1. [链下] `GET /v1/liquidity/remove/quote?lstMint=<jitoSOL>&amount=1000000` → 获取预期 LST 数量
2. 显示报价，**询问用户确认**
3. [链下] `resolve_wallet_solana()` → 获取钱包地址
4. [链上] `POST /v1/liquidity/remove` with signer + quotedAmount → 获取 base64 tx
5. base64 → base58
6. `onchainos wallet contract-call --chain 501 --to <program_id> --unsigned-tx <base58> --force`
7. 输出 txHash

---

## §4 外部 API 依赖

| API | Purpose | URL |
|-----|---------|-----|
| Sanctum Extra API | Pool stats, APY, sol-value, LST metadata | `https://extra-api.sanctum.so` |
| Sanctum S Router API | Swap quotes, swap tx, liquidity tx | `https://sanctum-s-api.fly.dev` |

---

## §5 配置参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| chain | 501 | Solana mainnet |
| dry_run | false | 模拟模式，不广播 |
| slippage | 0.5 | 滑点百分比 (0.5 = 0.5%) |
| swap_src | SPool | 强制使用 Infinity pool |

---

## Key Addresses

| Item | Value |
|------|-------|
| INF Token Mint | `5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm` |
| INF Pool Program ID | `5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx` |
| JitoSOL Mint | `J1toso1uCk3RLmjorhTtrVBVzHQDSsvVQ6n8CGBbBTkp` |
| mSOL Mint | `mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So` |
| wSOL (native SOL) | `So11111111111111111111111111111111111111112` |

## Amount Format Note

- All amounts in Sanctum S API are in **raw atomics** (lamports for SOL, 10^9 for 1.0 SOL)
- `1 JitoSOL = 1_000_000_000 raw atomics` (9 decimals)
- User inputs UI units (e.g., "0.005 JitoSOL"), code converts to atomics: `(amount * 10^9) as u64`
