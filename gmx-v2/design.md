# GMX V2 Plugin — Design Document

> **Phase 1 output** — Produced by Researcher Agent  
> **DApp:** GMX V2  
> **Priority rank:** 15  
> **Dev directory:** `/Users/samsee/projects/plugin-store-dev/gmx-v2`  
> **Monorepo source dir:** `gmx-v2` (in `skylavis-sky/onchainos-plugins`)

---

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| `plugin_name` | `gmx-v2` |
| `dapp_name` | GMX V2 |
| `binary_name` | `gmx-v2` |
| `version` | `0.1.0` |
| `category` | `defi-protocol` |
| `tags` | `perpetuals`, `spot`, `trading`, `arbitrum`, `avalanche`, `leverage` |
| `target_chains` | Arbitrum (42161), Avalanche (43114) |
| `target_protocols` | GMX V2 Synthetics (ExchangeRouter, Reader, GM pools, GLV vaults) |
| `description` | Trade perpetuals and spot on GMX V2 — open/close leveraged positions, place limit/stop orders, add/remove GM pool liquidity, query markets and positions |

---

## §1 接入可行性调研表

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | **No.** Official SDK is TypeScript-only (`@gmx-io/sdk`). No Rust SDK exists. |
| SDK 支持哪些技术栈？ | TypeScript (Node.js 18+ and browser). SDK v1 (read+write), SDK v2 (read-only). <https://docs.gmx.io/docs/sdk/overview/> |
| 有 REST API？ | **Yes.** `https://{chain}-api.gmxinfra.io` — endpoints: `/markets`, `/markets/info`, `/prices/tickers`, `/prices/candles`, `/tokens`, `/signed_prices/latest`. No auth key required. |
| 有官方 Skill？ | **Yes.** `gmx-io/gmx-ai` on GitHub — includes `gmx-trading` and `gmx-liquidity` skills (TypeScript/Claude plugin format, not Rust). <https://github.com/gmx-io/gmx-ai> |
| 开源社区有类似 Skill？ | Official skill above. Also Compass Labs medium article with V2 integration guide. No Rust community skill found. |
| 支持哪些链？ | Arbitrum (42161), Avalanche (43114), Botanix (3637), MegaETH (4326). We integrate Arbitrum + Avalanche only (EVM). |
| 是否需要 onchainos 广播？ | **Yes.** All order placement, position management, liquidity operations, and claims are on-chain writes. Must use `onchainos wallet contract-call`. |

**接入路径：** `参考已有Skill` — The official `gmx-io/gmx-ai` repository provides contract addresses, API endpoints, ABI patterns, and operation logic. We implement in Rust calling the REST API for reads and `onchainos wallet contract-call` for writes, following the same patterns.

---

## §2 接口映射

### 2a. 操作总表

| # | 操作 | 类型 | 说明 |
|---|------|------|------|
| 1 | `list-markets` | 链下查询 | 获取所有活跃市场及池子数据 |
| 2 | `get-prices` | 链下查询 | 获取代币当前报价（oracle 价格） |
| 3 | `get-positions` | 链下查询 + 链上 eth_call | 查询账户当前持仓 |
| 4 | `get-orders` | 链下查询 + 链上 eth_call | 查询账户当前挂单 |
| 5 | `open-position` | 链上写操作 | 开多/空仓（市价单） |
| 6 | `close-position` | 链上写操作 | 平仓（市价或限价） |
| 7 | `place-order` | 链上写操作 | 下限价单 / 止损单 / 止盈单 |
| 8 | `cancel-order` | 链上写操作 | 撤销挂单 |
| 9 | `deposit-liquidity` | 链上写操作 | 向 GM 池子存入流动性（mint GM tokens） |
| 10 | `withdraw-liquidity` | 链上写操作 | 从 GM 池子撤出流动性（burn GM tokens） |
| 11 | `claim-funding-fees` | 链上写操作 | 领取 funding fee 收益 |

---

### 2b. 链下查询表

#### 操作 1: `list-markets`

**API Endpoint:**
```
GET https://{chain_base_url}/markets/info
```

where `chain_base_url`:
- Arbitrum: `https://arbitrum-api.gmxinfra.io`
- Avalanche: `https://avalanche-api.gmxinfra.io`

**No authentication required.**

**关键参数:** (none — returns all markets)

**返回值关键字段:**

```json
{
  "markets": [
    {
      "name": "ETH/USD [ETH-USDC]",
      "marketToken": "0x...",        // GM pool token address
      "indexToken": "0x...",          // underlying asset (ETH)
      "longToken": "0x...",           // long collateral token
      "shortToken": "0x...",          // USDC
      "isListed": true,
      "availableLiquidityLong": "...",
      "availableLiquidityShort": "...",
      "openInterestLong": "...",
      "openInterestShort": "...",
      "fundingRateLong": "...",
      "fundingRateShort": "...",
      "borrowingRateLong": "...",
      "borrowingRateShort": "..."
    }
  ]
}
```

Filter to `isListed: true` only. Display name, OI, liquidity, and rates.

---

#### 操作 2: `get-prices`

**API Endpoint:**
```
GET https://{chain_base_url}/prices/tickers
```

**返回值关键字段:**

```json
[
  {
    "tokenAddress": "0x...",
    "tokenSymbol": "ETH",
    "minPrice": "1800000000000000000000000000000000",  // 30-decimal precision
    "maxPrice": "1801000000000000000000000000000000",
    "updatedAt": 1700000000000
  }
]
```

**Price decoding:** Prices are in 30-decimal units. To display: `price / 10^30`. Example: `1800 * 10^30` = `1800.00 USD`.

---

#### 操作 3: `get-positions`

Uses on-chain `eth_call` to the Reader contract.

**Reader contract addresses:**
- Arbitrum: `0x470fbC46bcC0f16532691Df360A07d8Bf5ee0789`
- Avalanche: `0x62Cb8740E6986B29dC671B2EB596676f60590A5B`

**DataStore addresses:**
- Arbitrum: `0xFD70de6b91282D8017aA4E741e9Ae325CAb992d8`
- Avalanche: `0x2F0b22339414ADeD7D5F06f9D604c7fF5b2fe3f6`

**Function signature:**
```
getAccountPositions(address dataStore, address account, uint256 start, uint256 end)
```
**Selector:** `0x77cfb162` *(verified via keccak256)*

**Calldata construction:**
```
0x77cfb162
<dataStore address padded to 32 bytes>
<account address padded to 32 bytes>
<start = 0, padded to 32 bytes>
<end = 20, padded to 32 bytes>   // fetch up to 20 positions
```

**Returned position fields include:**
- `account`, `market`, `collateralToken`, `sizeInUsd`, `sizeInTokens`, `collateralAmount`, `borrowingFactor`, `fundingFeeAmountPerSize`, `isLong`

---

#### 操作 4: `get-orders`

**Function signature:**
```
getAccountOrders(address dataStore, address account, uint256 start, uint256 end)
```
**Selector:** `0x42a6f8d3` *(verified via keccak256)*

Same Reader and DataStore contracts as above. Calldata pattern is identical to `getAccountPositions`, swapping selector.

**Returned order fields include:**
- `orderType` (uint8 enum), `market`, `initialCollateralToken`, `sizeDeltaUsd`, `triggerPrice`, `acceptablePrice`, `isLong`, `executionFee`

**OrderType enum:**
```
0 = MarketSwap
1 = LimitSwap
2 = MarketIncrease
3 = LimitIncrease
4 = MarketDecrease
5 = LimitDecrease
6 = StopLossDecrease
7 = Liquidation
8 = StopIncrease
```

---

### 2c. 链上写操作表

> All write operations use `onchainos wallet contract-call`.  
> The pattern for GMX V2 is a **multicall** on ExchangeRouter: combine `sendWnt` (execution fee) + `sendTokens` (collateral) + `createOrder` into a single `multicall(bytes[])` transaction.

**ExchangeRouter addresses:**
- Arbitrum: `0x1C3fa76e6E1088bCE750f23a5BFcffa1efEF6A41`
- Avalanche: `0x8f550E53DFe96C055D5Bdb267c21F268fCAF63B2`

**Router (for ERC-20 approval spender):**
- Arbitrum: `0x7452c558d45f8afC8c83dAe62C3f8A5BE19c71f6`
- Avalanche: `0x820F5FfC5b525cD4d88Cd91aCf2c28F16530Cc68`

**OrderVault (for sendTokens destination):**
- Arbitrum: `0x31eF83a530Fde1B38EE9A18093A333D8Bbbc40D5`
- Avalanche: `0xD3D60D22d415aD43b7e64b510D86A30f19B1B12C`

---

#### 操作 5: `open-position` (Market Increase)

**Pre-conditions:**
1. Query current prices from `/prices/tickers`
2. Fetch execution fee: compute from `getGasLimits()` or use a safe default (~0.001 ETH on Arbitrum / ~0.01 AVAX on Avalanche)
3. ERC-20 approve collateral token to Router contract (if not already approved)

**Step A — ERC-20 Approve** (skip if allowance already sufficient):
```
Function: approve(address,uint256)
Selector: 0x095ea7b3  [verified via keccak256]
Target: <collateral token address>

Calldata:
0x095ea7b3
000000000000000000000000<router_address_no_0x>      // spender = Router
ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff  // max uint256

onchainos wallet contract-call \
  --chain <chain_id> \
  --to <COLLATERAL_TOKEN> \
  --input-data 0x095ea7b3<router_padded><max_uint256>
```

**Step B — multicall to ExchangeRouter:**

The multicall bundles 3 inner calls:

1. `sendWnt(address receiver, uint256 amount)` — send execution fee (ETH/AVAX) to OrderVault
   - Selector: `0x7d39aaf1` *(verified via keccak256)*
   - `--amt <execution_fee_wei>` attached to the outer contract-call

2. `sendTokens(address token, address receiver, uint256 amount)` — send collateral to OrderVault
   - Selector: `0xe6d66ac8` *(verified via keccak256)*

3. `createOrder((Addresses),(Numbers),(Flags))` — create market increase order
   - Selector: `0x97aedce2` *(verified via keccak256)*
   - `CreateOrderParams` struct encoding:
     - **Addresses tuple:** `(account, receiver, cancellationReceiver=account, callbackContract=0x0, uiFeeReceiver=0x0, market=<marketToken>, initialCollateralToken=<token>, swapPath=[])`
     - **Numbers tuple:** `(orderType=2[MarketIncrease], decreasePositionSwapType=0, sizeDeltaUsd=<size*10^30>, initialCollateralDeltaAmount=<collateral>, triggerPrice=0, acceptablePrice=<acceptablePrice>, executionFee=<fee>, callbackGasLimit=0, minOutputAmount=0, updatedAtTime=0, validFromTime=0, srcChainId=<chainId>)`
     - **Flags tuple:** `(isLong=<true/false>, shouldUnwrapNativeToken=false, isFrozen=false, autoCancel=false)`

**Outer multicall calldata:**
```
Selector: 0xac9650d8  [multicall(bytes[]), verified via keccak256]
```

```bash
onchainos wallet contract-call \
  --chain <CHAIN_ID> \
  --to <EXCHANGE_ROUTER> \
  --input-data <multicall_calldata> \
  --amt <execution_fee_wei> \
  --from <wallet_address> \
  --force
```

**Market token address:** obtained at runtime from `/markets` API by matching `name` or `indexToken`. **Never hardcode market addresses.**

**Acceptable price computation:** `minPrice * (1 - slippage)` for longs, `maxPrice * (1 + slippage)` for shorts. Default slippage: 1% (100 bps). Note: GMX prices at 30-decimal precision.

---

#### 操作 6: `close-position` (Market Decrease)

Same multicall pattern as open-position but with:
- `orderType = 4` (MarketDecrease)
- `sizeDeltaUsd` = amount to close (use full position size to close entirely)
- `initialCollateralDeltaAmount` = collateral to withdraw
- `isLong` = matches existing position direction
- For full close: `sizeDeltaUsd = position.sizeInUsd`, `initialCollateralDeltaAmount = position.collateralAmount`
- **No collateral approve needed** (collateral stays in vault)

Only `sendWnt` (execution fee) is needed in the multicall for decreases (no `sendTokens`).

---

#### 操作 7: `place-order` (Limit / Stop-Loss / Take-Profit)

Same multicall pattern as `open-position` but with different `orderType` and `triggerPrice`:
- Limit long entry: `orderType = 3` (LimitIncrease), `triggerPrice = limit_price * 10^30`
- Stop-loss: `orderType = 6` (StopLossDecrease), `triggerPrice = stop_price * 10^30`
- Take-profit: `orderType = 5` (LimitDecrease), `triggerPrice = tp_price * 10^30`

**Trigger price encoding:** `price_usd * 10^30` as uint256.

---

#### 操作 8: `cancel-order`

```
Function: cancelOrder(bytes32 key)
Selector: 0x7489ec23  [verified via keccak256]
Target: ExchangeRouter
```

**Order key:** obtained from `getAccountOrders` eth_call.

**Calldata:**
```
0x7489ec23
<order_key_bytes32>
```

```bash
onchainos wallet contract-call \
  --chain <CHAIN_ID> \
  --to <EXCHANGE_ROUTER> \
  --input-data 0x7489ec23<order_key> \
  --from <wallet_address> \
  --force
```

---

#### 操作 9: `deposit-liquidity` (GM Pool)

**DepositVault addresses:**
- Arbitrum: `0xF89e77e8Dc11691C9e8757e84aaFbCD8A67d7A55`
- Avalanche: `0x90c670825d0C62ede1c5ee9571d6d9a17A722DFF`

Multicall on ExchangeRouter:
1. `sendWnt` → DepositVault (execution fee)
2. `sendTokens(longToken, DepositVault, longAmount)` (if providing long-side)
3. `sendTokens(shortToken, DepositVault, shortAmount)` (if providing short-side)
4. `createDeposit(CreateDepositParams)` — params include `market`, `receiver`, `minMarketTokens`, `executionFee`

**Pre-condition:** Approve both tokens to Router.

```bash
# Approve long token
onchainos wallet contract-call --chain <CHAIN_ID> --to <LONG_TOKEN> \
  --input-data 0x095ea7b3<router_padded><max_uint256>

# Approve short token (USDC)
onchainos wallet contract-call --chain <CHAIN_ID> --to <SHORT_TOKEN> \
  --input-data 0x095ea7b3<router_padded><max_uint256>

# Deposit multicall
onchainos wallet contract-call --chain <CHAIN_ID> \
  --to <EXCHANGE_ROUTER> \
  --input-data <multicall_calldata> \
  --amt <execution_fee_wei> \
  --from <wallet_address> \
  --force
```

---

#### 操作 10: `withdraw-liquidity` (GM Pool)

**WithdrawalVault addresses:**
- Arbitrum: `0x0628D46b5D145f183AdB6Ef1f2c97eD1C4701C55`
- Avalanche: `0xf5F30B10141E1F63FC11eD772931A8294a591996`

Multicall on ExchangeRouter:
1. `sendWnt` → WithdrawalVault (execution fee)
2. `sendTokens(gmToken, WithdrawalVault, gmAmount)` — send GM tokens to vault
3. `createWithdrawal(CreateWithdrawalParams)` — params include `market`, `receiver`, `minLongTokenAmount`, `minShortTokenAmount`, `executionFee`

**Pre-condition:** Approve GM token to Router.

---

#### 操作 11: `claim-funding-fees`

```
Function: claimFundingFees(address[] markets, address[] tokens, address receiver)
Selector: 0xc41b1ab3  [verified via keccak256]
Target: ExchangeRouter
```

**Calldata:** ABI-encode the arrays of market addresses and token addresses.

```bash
onchainos wallet contract-call \
  --chain <CHAIN_ID> \
  --to <EXCHANGE_ROUTER> \
  --input-data <abi_encoded_claim_calldata> \
  --from <wallet_address> \
  --force
```

**Notes:**
- No execution fee (ETH value) needed for claim.
- Markets and tokens arrays must be same length and correspond pairwise.
- Query claimable amounts via Subsquid GraphQL before calling.

---

### Important Notes for Developer

1. **Two-phase execution:** GMX V2 uses a keeper model. `createOrder` creates a pending order; a keeper executes it 1–30 seconds later. The transaction hash from `contract-call` is the *creation* tx, not the execution tx.

2. **Execution fee:** Must send native token (ETH/AVAX) as value with the multicall. The surplus is refunded automatically by the protocol. Safe defaults: 0.001 ETH on Arbitrum, 0.012 AVAX on Avalanche.

3. **Price precision:** All GMX V2 prices use 30-decimal precision (1 USD = 10^30 in contract units). Token amounts use native decimals (USDC = 6, ETH = 18, WBTC = 8).

4. **Market addresses are NOT hardcoded.** Always fetch dynamically from `/markets` API at runtime and match by `indexToken` address or `name` string.

5. **No dry-run on contract-call.** Handle dry_run in Rust wrapper with early return; never pass `--dry-run` to onchainos CLI.

6. **Force flag:** All DEX/protocol `contract-call` invocations must include `--force` after user confirmation.

---

## §3 用户场景

### 场景 1: 开 ETH 多仓（核心 happy path）

**用户说:** "在 Arbitrum 上用 1000 USDC 开 5x ETH 多仓"

**Agent 动作序列:**

1. [链下查询] 调用 `GET https://arbitrum-api.gmxinfra.io/markets` — 查找 ETH/USD 市场，获取 `marketToken` 地址
2. [链下查询] 调用 `GET https://arbitrum-api.gmxinfra.io/prices/tickers` — 获取 ETH 当前 `minPrice` / `maxPrice`，换算为 USD (÷ 10^30)
3. [链下查询] 调用 `GET https://arbitrum-api.gmxinfra.io/markets/info` — 检查 ETH/USD 市场 `availableLiquidityLong`，确认 5000 USD 仓位可容纳
4. [Agent 计算] `sizeDeltaUsd = 1000 * 5 * 10^30 = 5000 * 10^30`；`initialCollateralDeltaAmount = 1000 * 10^6`（USDC 6位小数）；`acceptablePrice = currentMinPrice * 0.99`（1% 滑点容忍）
5. [Agent 风控] 检查仓位大小 < `availableLiquidityLong`，提示用户确认订单详情（市场、方向、杠杆、滑点、预估手续费）
6. [用户确认] 等待用户确认
7. [链上操作] 检查 USDC 对 Router 的 allowance（`eth_call` 到 USDC 合约的 `allowance(owner,spender)` ） 
8. [链上操作 — 如不足] `onchainos wallet contract-call --chain 42161 --to <USDC> --input-data 0x095ea7b3<router><max_uint256>` — 批准 USDC
9. [链上操作] 构造 multicall calldata：`[sendWnt(OrderVault, 0.001 ETH), sendTokens(USDC, OrderVault, 1000e6), createOrder(...MarketIncrease, isLong=true...)]`
10. [onchainos 命令] `onchainos wallet contract-call --chain 42161 --to 0x1C3fa76e6E1088bCE750f23a5BFcffa1efEF6A41 --input-data <multicall_hex> --amt 1000000000000000 --from <wallet> --force`
11. [链下查询] 10~30 秒后调用 `getAccountPositions` eth_call 确认持仓已创建，向用户显示仓位详情

---

### 场景 2: 查询持仓和挂单（查询类）

**用户说:** "查一下我在 GMX 上 Arbitrum 的所有持仓和挂单"

**Agent 动作序列:**

1. [链下查询] 调用 `onchainos wallet addresses` 获取当前 EVM 钱包地址
2. [链上 eth_call] 构造 `getAccountPositions(DataStore, walletAddr, 0, 20)` calldata (`0x77cfb162`)，调用 Reader 合约 `0x470fbC46bcC0f16532691Df360A07d8Bf5ee0789`
3. [链下查询] 调用 `GET https://arbitrum-api.gmxinfra.io/prices/tickers` 获取当前价格，计算每个持仓的未实现盈亏：`unrealizedPnl = (currentPrice - entryPrice) * sizeInTokens * (isLong ? 1 : -1)`
4. [链上 eth_call] 构造 `getAccountOrders(DataStore, walletAddr, 0, 20)` calldata (`0x42a6f8d3`)，调用 Reader 合约
5. [链下查询] 调用 `/markets` 解析 marketToken 地址 → 市场名称（用于展示）
6. [Agent 输出] 格式化展示持仓（市场/方向/规模/入场价/当前PnL/杠杆）和挂单（类型/触发价/规模）列表

---

### 场景 3: 设置止损单（风控操作）

**用户说:** "给我 ETH 多仓设一个止损，价格跌到 1700 美元时触发"

**Agent 动作序列:**

1. [链上 eth_call] `getAccountPositions(DataStore, wallet, 0, 20)` — 确认存在 ETH 多仓，获取 `sizeInUsd`、`collateralAmount`、`market`
2. [链下查询] `GET /prices/tickers` — 获取当前 ETH 价格，验证 1700 USD 止损价低于当前价格（多仓止损必须低于当前价）
3. [Agent 风控] 计算止损亏损比例：`(currentPrice - stopPrice) / currentPrice`，如亏损 > 80% 则警告用户
4. [Agent 计算] `triggerPrice = 1700 * 10^30`；`sizeDeltaUsd = position.sizeInUsd`（全部平仓）；`orderType = 6`（StopLossDecrease）
5. [Agent] 向用户展示止损单详情（触发价、预估收到抵押品），请求确认
6. [用户确认]
7. [链上操作] 构造 multicall：`[sendWnt(OrderVault, execution_fee), createOrder(...StopLossDecrease, triggerPrice=1700e30, isLong=true...)]`
8. [onchainos 命令] `onchainos wallet contract-call --chain 42161 --to <EXCHANGE_ROUTER> --input-data <multicall_hex> --amt <fee_wei> --from <wallet> --force`
9. [链上 eth_call] `getAccountOrders` — 确认止损单已创建，展示订单详情给用户

---

### 场景 4: 提供 GM 池流动性（补充场景）

**用户说:** "向 ETH/USD GM 池存入 500 USDC"

**Agent 动作序列:**

1. [链下查询] `GET /markets` — 查找 ETH/USD 市场的 `marketToken`、`longToken` (ETH)、`shortToken` (USDC)
2. [链下查询] `onchainos wallet balance --chain 42161` — 确认 USDC 余额 ≥ 500
3. [链下查询] `GET /markets/info` — 显示当前 GM 池 APY、TVL、池子构成比例
4. [Agent] 向用户展示预期铸造的 GM token 数量（通过 `getDepositAmountOut` eth_call 估算），请求确认
5. [用户确认]
6. [链上操作] `approve USDC → Router` (如不足)
7. [链上操作] multicall `sendWnt + sendTokens(USDC, DepositVault, 500e6) + createDeposit(...)`
8. [onchainos 命令] `onchainos wallet contract-call --chain 42161 --to <EXCHANGE_ROUTER> --input-data <multicall_hex> --amt <fee_wei> --from <wallet> --force`
9. [链下查询] 等待 keeper 执行后通过 `onchainos wallet balance` 确认 GM token 余额增加

---

## §4 外部 API 依赖

| API | URL 模式 | 用途 | Auth |
|-----|---------|------|------|
| GMX Oracle API (Arbitrum) | `https://arbitrum-api.gmxinfra.io` | 市场数据、价格、代币列表 | None |
| GMX Oracle API (Avalanche) | `https://avalanche-api.gmxinfra.io` | 市场数据、价格、代币列表 | None |
| GMX Oracle API fallback (Arbitrum) | `https://arbitrum-api.gmxinfra2.io` | 主 API 故障备份 | None |
| GMX Oracle API fallback (Avalanche) | `https://avalanche-api.gmxinfra2.io` | 主 API 故障备份 | None |
| Subsquid GraphQL (Arbitrum) | `https://gmx.squids.live/gmx-synthetics-arbitrum:prod/api/graphql` | 交易历史、claimable fees 查询 | None |
| Subsquid GraphQL (Avalanche) | `https://gmx.squids.live/gmx-synthetics-avalanche:prod/api/graphql` | 交易历史、claimable fees 查询 | None |
| Arbitrum RPC | `https://arb1.arbitrum.io/rpc` or `https://arbitrum.publicnode.com` | eth_call for Reader contract | None |
| Avalanche RPC | `https://api.avax.network/ext/bc/C/rpc` or `https://avalanche-c-chain-rpc.publicnode.com` | eth_call for Reader contract | None |

**API Endpoint Summary for `plugin.yaml`:**
```yaml
api_calls:
  - "https://arbitrum-api.gmxinfra.io"
  - "https://avalanche-api.gmxinfra.io"
  - "https://arbitrum-api.gmxinfra2.io"
  - "https://avalanche-api.gmxinfra2.io"
  - "https://gmx.squids.live/gmx-synthetics-arbitrum:prod/api/graphql"
  - "https://gmx.squids.live/gmx-synthetics-avalanche:prod/api/graphql"
  - "https://arbitrum.publicnode.com"
  - "https://avalanche-c-chain-rpc.publicnode.com"
```

---

## §5 配置参数

| 参数名 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `chain` | `string` | `"arbitrum"` | 目标链，`"arbitrum"` 或 `"avalanche"` |
| `slippage_bps` | `u32` | `100` | 滑点容忍（基点），默认 1%，范围 1–500 |
| `execution_fee_arbitrum` | `u64` | `1000000000000000` | Arbitrum 执行费（wei），默认 0.001 ETH |
| `execution_fee_avalanche` | `u64` | `12000000000000000` | Avalanche 执行费（nAVAX），默认 0.012 AVAX |
| `max_auto_cancel_orders` | `u32` | `10` | 每个持仓最大挂单数（Arbitrum上限11，其余6） |
| `ui_fee_receiver` | `address` | `0x0000...0000` | UI fee receiver，默认零地址（无额外费用） |
| `dry_run` | `bool` | `false` | 若为 true，跳过链上操作，仅模拟并返回预期 calldata |
| `rpc_url_arbitrum` | `string` | `"https://arbitrum.publicnode.com"` | Arbitrum RPC，覆盖默认值 |
| `rpc_url_avalanche` | `string` | `"https://avalanche-c-chain-rpc.publicnode.com"` | Avalanche RPC，覆盖默认值 |

---

## §6 合约地址速查表（运行时动态获取，此处仅供参考）

> **WARNING:** Developer Agent must NOT hardcode these addresses. They must be selected at runtime based on `chain` config. This table is reference only.

### Arbitrum (42161)

| Contract | Address |
|----------|---------|
| ExchangeRouter | `0x1C3fa76e6E1088bCE750f23a5BFcffa1efEF6A41` |
| Router (spender for approvals) | `0x7452c558d45f8afC8c83dAe62C3f8A5BE19c71f6` |
| OrderVault | `0x31eF83a530Fde1B38EE9A18093A333D8Bbbc40D5` |
| DepositVault | `0xF89e77e8Dc11691C9e8757e84aaFbCD8A67d7A55` |
| WithdrawalVault | `0x0628D46b5D145f183AdB6Ef1f2c97eD1C4701C55` |
| Reader | `0x470fbC46bcC0f16532691Df360A07d8Bf5ee0789` |
| DataStore | `0xFD70de6b91282D8017aA4E741e9Ae325CAb992d8` |
| GlvRouter | `0x7EAdEE2ca1b4D06a0d82fDF03D715550c26AA12F` |
| GlvVault | `0x393053B58f9678C9c28c2cE941fF6cac49C3F8f9` |

### Avalanche (43114)

| Contract | Address |
|----------|---------|
| ExchangeRouter | `0x8f550E53DFe96C055D5Bdb267c21F268fCAF63B2` |
| Router (spender for approvals) | `0x820F5FfC5b525cD4d88Cd91aCf2c28F16530Cc68` |
| OrderVault | `0xD3D60D22d415aD43b7e64b510D86A30f19B1B12C` |
| DepositVault | `0x90c670825d0C62ede1c5ee9571d6d9a17A722DFF` |
| WithdrawalVault | `0xf5F30B10141E1F63FC11eD772931A8294a591996` |
| Reader | `0x62Cb8740E6986B29dC671B2EB596676f60590A5B` |
| DataStore | `0x2F0b22339414ADeD7D5F06f9D604c7fF5b2fe3f6` |
| GlvRouter | `0x7E425c47b2Ff0bE67228c842B9C792D0BCe58ae6` |
| GlvVault | `0x527FB0bCfF63C47761039bB386cFE181A92a4701` |

---

## §7 Function Selector 验证记录

> All selectors verified using `pycryptodome` keccak256 (NOT Python `hashlib.sha3_256` which is NIST SHA3, not Ethereum Keccak — see kb/protocols/dex.md#python-sha3-wrong-selector).

| 函数签名 | Selector | 来源 |
|----------|----------|------|
| `multicall(bytes[])` | `0xac9650d8` | keccak256 verified + 4byte.directory confirmed |
| `sendWnt(address,uint256)` | `0x7d39aaf1` | keccak256 verified |
| `sendTokens(address,address,uint256)` | `0xe6d66ac8` | keccak256 verified |
| `createOrder((address,address,address,address,address,address,address,address[]),(uint8,uint8,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256),(bool,bool,bool,bool))` | `0x97aedce2` | keccak256 verified |
| `cancelOrder(bytes32)` | `0x7489ec23` | keccak256 verified |
| `updateOrder(bytes32,uint256,uint256,uint256,uint256,uint256,bool)` | `0xdd5baad2` | keccak256 verified |
| `claimFundingFees(address[],address[],address)` | `0xc41b1ab3` | keccak256 verified |
| `approve(address,uint256)` | `0x095ea7b3` | keccak256 verified (standard ERC-20) |
| `getMarkets(address,uint256,uint256)` | `0xce3264bf` | keccak256 verified + 4byte.directory confirmed |
| `getAccountPositions(address,address,uint256,uint256)` | `0x77cfb162` | keccak256 verified |
| `getAccountOrders(address,address,uint256,uint256)` | `0x42a6f8d3` | keccak256 verified |

---

## §8 已知风险与注意事项

1. **Keeper delay:** Orders are NOT executed immediately. The plugin must communicate to the user that the position will be open within 1–30 seconds after the tx lands.

2. **Max orders per position:** Arbitrum: 11 concurrent TP/SL orders per position. Avalanche: 6. Check existing order count before placing new conditional orders.

3. **Execution fee refund:** Surplus execution fee is auto-refunded by the protocol. The plugin should use conservative estimates (higher end) to avoid order rejection by keepers.

4. **Price staleness:** Oracle prices expire within seconds. Always fetch `/prices/tickers` immediately before building calldata for orders.

5. **Market liquidity check:** Before opening a position, verify `availableLiquidityLong` (for longs) or `availableLiquidityShort` (for shorts) from `/markets/info` exceeds `sizeDeltaUsd`.

6. **USDC address on Arbitrum:** `0xaf88d065e77c8cC2239327C5EDb3A432268e5831` (native USDC). Note: bridged USDC.e is `0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8` — always use native USDC.

7. **Swap-only markets:** Some markets have `indexToken = null` and are swap-only (no perpetuals). Filter these out for trading operations.

8. **GLV vaults:** Advanced feature — multi-market auto-rebalancing vaults. Out of scope for initial plugin implementation. Focus on GM pools first.
