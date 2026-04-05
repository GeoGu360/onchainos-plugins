# Trader Joe DEX — Plugin Design (PRD)

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `trader-joe` |
| dapp_name | Trader Joe DEX |
| version | 0.1.0 |
| target_chains | EVM — Arbitrum (42161) |
| target_protocols | Trader Joe Liquidity Book V2.1/V2.2 |
| category | dex |
| tags | dex, swap, liquidity-book, concentrated-liquidity, arbitrum |

---

## §1 接入可行性调研表

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 无官方 Rust SDK。官方 SDK 为 TypeScript (@traderjoe-xyz/sdk-v2) |
| SDK 支持哪些技术栈？ | TypeScript/JavaScript SDK；无 Rust |
| 有 REST API？ | 无专用 REST API。直接通过合约调用（eth_call）交互 |
| 有官方 Skill？ | 无 |
| 开源社区有类似 Skill？ | 无 — 自行开发 |
| 支持哪些链？ | Avalanche C-Chain (43114), Arbitrum One (42161), BNB Chain (56), Ethereum (1), Base (8453), Fuji Testnet。接入 Arbitrum (42161) |
| 是否需要 onchainos 广播？ | Yes — swap 操作需要通过 onchainos wallet contract-call 广播 |

**接入路径**：直接合约调用（eth_call + onchainos wallet contract-call）

---

## §2 接口映射

### 2a 需要接入的操作

| 操作 | 类型 | 优先级 |
|------|------|--------|
| quote (获取报价) | 链下查询 | P0 |
| pools (查询交易对) | 链下查询 | P0 |
| swap (代币兑换) | 链上写操作 | P0 |

### 2b 链下查询表

#### quote — 获取最优报价

使用 LBQuoter 合约的 `findBestPathFromAmountIn` 函数。

- **合约**: LBQuoter `0xd76019A16606FDa4651f636D9751f500Ed776250` (Arbitrum)
- **函数签名**: `findBestPathFromAmountIn(address[],uint128)` → selector `0x0f902a40` ✅ (cast sig verified)
- **参数**:
  - `route: address[]` — 代币路径 [tokenIn, tokenOut]
  - `amountIn: uint128` — 输入数量（原子单位）
- **返回值** (Quote struct):
  ```
  struct Quote {
    address[] route;        // 代币路径
    address[] pairs;        // LBPair 地址数组
    uint256[] binSteps;     // 每跳的 binStep
    uint8[] versions;       // 每跳的版本 (0=V1, 1=V2, 2=V2_1, 3=V2_2)
    uint128[] amounts;      // [amountIn, ..., amountOut] 每跳数量
    uint128[] virtualAmountsWithoutSlippage; // 无滑点虚拟数量
    uint128[] fees;         // 每跳费用
  }
  ```
- **关键字段**: `amounts[last]` = amountOut, `binSteps[0]` = 最优 binStep, `versions[0]` = 版本

Also supports `findBestPathFromAmountOut(address[],uint128)` → selector `0x59214226` ✅ for exact-output quotes.

#### pools — 查询代币对

使用 LBFactory 合约的 `getAllLBPairs` 函数。

- **合约**: LBFactory `0xb43120c4745967fa9b93E79C149E66B0f2D6Fe0c` (Arbitrum)
- **函数签名**: `getAllLBPairs(address,address)` → selector `0x6622e0d7` ✅ (cast sig verified)
- **参数**:
  - `tokenX: address`
  - `tokenY: address`
- **返回值** (LBPairInformation[]):
  ```
  struct LBPairInformation {
    uint16 binStep;
    ILBPair LBPair;      // pair 地址
    bool createdByOwner;
    bool ignoredForRouting;
  }
  ```

Also: `getLBPairInformation(address,address,uint256)` → selector `0x704037bd` ✅ for specific binStep lookup.

### 2c 链上写操作表

#### swap — 代币兑换

使用 LBRouter 合约的 `swapExactTokensForTokens` 函数（ERC-20 to ERC-20）。

- **合约**: LBRouter V2.2 `0x18556DA13313f3532c54711497A8FedAC273220E` (Arbitrum)
- **函数签名**: `swapExactTokensForTokens(uint256,uint256,(uint256[],uint8[],address[]),address,uint256)` → selector `0x2a443fae` ✅

**流程**:
1. 调用 LBQuoter.findBestPathFromAmountIn 获取报价（binSteps, versions, pair）
2. 检查 tokenIn 的 ERC-20 allowance
3. 如果 allowance 不足，先 approve LBRouter（approve selector: `0x095ea7b3` ✅）
4. 等待 3 秒
5. 调用 swapExactTokensForTokens

**calldata 构造**（使用 alloy-sol-types）:
```rust
sol! {
    function swapExactTokensForTokens(
        uint256 amountIn,
        uint256 amountOutMin,
        (uint256[] pairBinSteps, uint8[] versions, address[] tokenPath) path,
        address to,
        uint256 deadline
    ) external returns (uint256 amountOut);
}
```

**参数说明**:
- `amountIn`: 输入数量（原子单位）
- `amountOutMin`: 最小输出数量（amountOut * (1 - slippage)）
- `path.pairBinSteps`: 从 quoter 获取的 binSteps 数组
- `path.versions`: 从 quoter 获取的 versions 数组（uint8）
- `path.tokenPath`: 代币路径地址数组 [tokenIn, ..., tokenOut]
- `to`: 接收地址（必须是钱包地址，不能是零地址）
- `deadline`: `block.timestamp + 300`（5分钟）

**onchainos 命令**:
```
onchainos wallet contract-call \
  --chain 42161 \
  --to 0x18556DA13313f3532c54711497A8FedAC273220E \
  --input-data <calldata> \
  --force
```
注意: DEX swap **必须加 `--force`**，否则返回 `txHash: "pending"` 不广播。

**ETH native swap**: 使用 `swapExactNATIVEForTokens` → selector `0xb066ea7c` ✅（附 `--amt <wei>`）

---

## §3 用户场景

### 场景 1：查询 USDT → WETH 兑换报价

**用户**: "在 Trader Joe 上查询用 0.01 USDT 能换多少 WETH"

**Agent 动作**:
1. [链下查询] 解析代币符号 → USDT: `0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9`, WETH: `0x82aF49447D8a07e3bd95BD0d56f35241523fBab1`
2. [链下查询] 调用 `LBQuoter.findBestPathFromAmountIn([USDT, WETH], 10000)`（0.01 USDT = 10000 raw）
3. [链下查询] 解析 Quote.amounts[last] → amountOut（WETH wei 单位）
4. 输出: `{"amountIn": "0.01", "amountOut": "0.0000049", "tokenIn": "USDT", "tokenOut": "WETH", "binStep": 15, "version": "V2_1", "fee_bps": 15}`

### 场景 2：执行 USDT 兑换 WETH

**用户**: "在 Trader Joe 上用 0.01 USDT 换 WETH，滑点 1%"

**Agent 动作**:
1. [链下查询] 获取当前钱包地址：`onchainos wallet balance --chain 42161 --output json`
2. [链下查询] 调用 LBQuoter 获取报价 → amountOut, binSteps, versions
3. [链下查询] 检查 USDT allowance：`allowance(wallet, LBRouter)` 
4. [链上操作] 如 allowance 不足，先 approve LBRouter 最大额度：
   ```
   onchainos wallet contract-call --chain 42161 --to <USDT_ADDR> --input-data 0x095ea7b3<router_padded><uint256_max> --force
   ```
5. 等待 3 秒
6. [链上操作] 调用 swapExactTokensForTokens（**ask user to confirm**）:
   ```
   onchainos wallet contract-call --chain 42161 --to <LBRouter> --input-data <calldata> --force
   ```
7. 提取 txHash，输出结果

### 场景 3：查询 WETH/USDT 交易对信息

**用户**: "查询 Trader Joe 上 WETH 和 USDT 的流动性池信息"

**Agent 动作**:
1. [链下查询] 调用 `LBFactory.getAllLBPairs(WETH, USDT)`
2. [链下查询] 解析每个 LBPairInformation：binStep, pairAddress, ignoredForRouting
3. [链下查询] 对每个 pair 调用 `LBPair.getActiveId()` 获取当前活跃 bin
4. 输出所有可用池及其 binStep、地址、活跃 bin

---

## §4 外部 API 依赖

| API | 用途 | 类型 |
|-----|------|------|
| `https://arb1.arbitrum.io/rpc` | 链上 eth_call（quoter, factory 查询） | JSON-RPC |
| LBQuoter `0xd76019A16606FDa4651f636D9751f500Ed776250` | 获取最优报价 | eth_call |
| LBFactory `0xb43120c4745967fa9b93E79C149E66B0f2D6Fe0c` | 查询交易对 | eth_call |
| LBRouter `0x18556DA13313f3532c54711497A8FedAC273220E` | 执行 swap | onchainos contract-call |
| onchainos wallet contract-call | 链上广播 | CLI |

---

## §5 配置参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `chain` | u64 | 42161 | EVM chain ID |
| `slippage` | f64 | 0.5 | 滑点百分比（0.5 = 0.5%） |
| `dry_run` | bool | false | 模拟模式，不广播 |
| `deadline_secs` | u64 | 300 | 交易截止时间（秒，从当前时间算） |

---

## §6 合约地址（Arbitrum 42161）

| 合约 | 地址 | 版本 |
|------|------|------|
| LBRouter | `0x18556DA13313f3532c54711497A8FedAC273220E` | V2.2 |
| LBFactory | `0xb43120c4745967fa9b93E79C149E66B0f2D6Fe0c` | V2.2 |
| LBQuoter | `0xd76019A16606FDa4651f636D9751f500Ed776250` | Multi-version |
| WETH | `0x82aF49447D8a07e3bd95BD0d56f35241523fBab1` | — |
| USDT (USD₮0) | `0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9` | — |
| USDC | `0xaf88d065e77c8cC2239327C5EDb3A432268e5831` | — |

---

## §7 Selector 验证记录

| 函数签名 | 验证命令 | Selector | 状态 |
|----------|---------|---------|------|
| `swapExactTokensForTokens(uint256,uint256,(uint256[],uint8[],address[]),address,uint256)` | `cast sig` | `0x2a443fae` | ✅ |
| `swapExactNATIVEForTokens(uint256,(uint256[],uint8[],address[]),address,uint256)` | `cast sig` | `0xb066ea7c` | ✅ |
| `swapExactTokensForNATIVE(uint256,uint256,(uint256[],uint8[],address[]),address payable,uint256)` | `cast sig` | `0x9ab6156b` | ✅ |
| `getAllLBPairs(address,address)` | `cast sig` | `0x6622e0d7` | ✅ |
| `getLBPairInformation(address,address,uint256)` | `cast sig` | `0x704037bd` | ✅ |
| `findBestPathFromAmountIn(address[],uint128)` | `cast sig` | `0x0f902a40` | ✅ |
| `findBestPathFromAmountOut(address[],uint128)` | `cast sig` | `0x59214226` | ✅ |
| `approve(address,uint256)` | `cast sig` | `0x095ea7b3` | ✅ |
| `allowance(address,address)` | `cast sig` | `0xdd62ed3e` | ✅ |
| `getSwapOut(address,uint128,bool)` | `cast sig` | verified via cast call | ✅ |

---

## §8 Liquidity Book 技术概述

Trader Joe 的 Liquidity Book (LB) 与 Uniswap V3 的关键区别：
- **离散 bin 模型**（而非连续 tick）：流动性分布在固定间隔的 bin 中
- **binStep**：价格精度参数（如 binStep=15 意味着相邻 bin 间价格差 0.15%）
- **版本**：V2.1 和 V2.2 都由同一 LBQuoter/LBRouter 处理，通过 `Version` 枚举区分
- **单一活跃 bin**：同一时间只有一个 bin 在接受交易（活跃 bin）
- **swapForY**：当 tokenX→tokenY 时为 true，tokenY→tokenX 时为 false
