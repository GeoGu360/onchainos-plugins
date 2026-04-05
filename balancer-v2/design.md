# Balancer V2 Plugin — Design Document

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| `plugin_name` | `balancer-v2` |
| `dapp_name` | Balancer V2 |
| `target_chains` | EVM: Arbitrum (42161), Ethereum (1) |
| `target_protocols` | DEX / AMM |
| `category` | dex |
| `author` | GeoGu360 |

---

## §1 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 否。Balancer 官方无 Rust SDK。 |
| SDK 支持哪些技术栈？ | TypeScript (balancer-sdk), Python (balpy) |
| 有 REST API？ | 有 Balancer API: `https://api.balancer.fi/` (GraphQL + REST) |
| 有官方 Skill？ | 否 |
| 开源社区有类似 Skill？ | 否 |
| 支持哪些链？ | Ethereum (1), Arbitrum (42161), Polygon (137), Base (8453), Optimism (10), Gnosis (100) |
| 是否需要 onchainos 广播？ | Yes — 所有链上操作（swap, joinPool, exitPool, approve）必须通过 onchainos wallet contract-call |

**接入路径：** API（直接 eth_call + onchainos wallet contract-call 构造 Vault calldata）

---

## §2 接口映射

### 需要接入的操作

| 操作 | 类型 | 描述 |
|------|------|------|
| `pools` | 链下查询 | 列出指定链上的 Balancer V2 池 |
| `pool-info` | 链下查询 | 查询单个池的详情（代币、余额、权重） |
| `quote` | 链下查询 | 报价：单次 swap 预估输出量 |
| `positions` | 链下查询 | 查询钱包的 LP 仓位（持有的 BPT） |
| `swap` | 链上写操作 | 通过 Vault.swap() 执行单次代币兑换 |
| `join` | 链上写操作 | 通过 Vault.joinPool() 添加流动性 |
| `exit` | 链上写操作 | 通过 Vault.exitPool() 移除流动性 |

---

### 链下查询表

#### pools — 列出池
- **接口**：`eth_call` Vault.getPoolTokens() + Balancer Subgraph (可选)
- **RPC**: `https://arbitrum-one-rpc.publicnode.com` (chain 42161)
- **合约**: Vault `0xBA12222222228d8Ba445958a75a0704d566BF2C8`
- **方法**: 通过 Balancer Subgraph API 查询池列表
- **API**: `POST https://api.thegraph.com/subgraphs/name/balancer-labs/balancer-arbitrum-v2`
- **Query**:
```graphql
{
  pools(first: 20, orderBy: totalLiquidity, orderDirection: desc, where: {totalLiquidity_gt: "1000"}) {
    id
    address
    poolType
    totalLiquidity
    swapFee
    tokens {
      address
      symbol
      decimals
      weight
      balance
    }
  }
}
```
- **返回**: id (poolId), address, poolType, totalLiquidity, swapFee, tokens[]

#### pool-info — 池详情
- **接口**: `eth_call` Vault.getPoolTokens(poolId)
- **Selector**: `0xf94d4668` (getPoolTokens(bytes32)) — verified via `cast sig`
- **参数**: poolId (bytes32)
- **返回**: tokens[], balances[], lastChangeBlock
- 额外读取 `getPool(poolId)` → pool address → 调用 `getSwapFeePercentage()`, `getPoolId()`, `totalSupply()`

#### quote — 报价
- **接口**: `eth_call` BalancerQueries.querySwap()
- **合约**: BalancerQueries `0xE39B5e3B6D74016b2F6A9673D7d7493B6DF549d5` (Arbitrum)
- **Selector**: `0xe969f6b3` (querySwap((bytes32,uint8,address,address,uint256,bytes),(address,bool,address,bool))) — verified via `cast sig`
- **参数**:
  - singleSwap: {poolId, kind=0(GIVEN_IN), assetIn, assetOut, amount, userData=0x}
  - funds: {sender=zero, fromInternalBalance=false, recipient=zero, toInternalBalance=false}
- **返回**: amountOut (uint256)

#### positions — LP 仓位
- **接口**: `eth_call` ERC20.balanceOf(wallet) 对每个已知池的 BPT token
- **Selector**: `0x70a08231` (balanceOf(address)) — verified via `cast sig`
- 对每个池地址调用 balanceOf(wallet)，筛选 balance > 0
- 需要已知池地址列表（通过 Subgraph 查询）

---

### 链上写操作表

#### swap — 代币兑换

**合约**: Vault `0xBA12222222228d8Ba445958a75a0704d566BF2C8`
**Selector**: `0x52bbbe29` (swap((bytes32,uint8,address,address,uint256,bytes),(address,bool,address,bool),uint256,uint256)) — verified via `cast sig`

**calldata 构造** (使用 alloy-sol-types sol! 宏):
```rust
sol! {
    struct SingleSwap {
        bytes32 poolId;
        uint8 kind;        // SwapKind: 0=GIVEN_IN, 1=GIVEN_OUT
        address assetIn;
        address assetOut;
        uint256 amount;
        bytes userData;
    }
    struct FundManagement {
        address sender;
        bool fromInternalBalance;
        address recipient;
        bool toInternalBalance;
    }
    function swap(
        SingleSwap singleSwap,
        FundManagement funds,
        uint256 limit,      // min amount out (GIVEN_IN) or max amount in (GIVEN_OUT)
        uint256 deadline
    ) external returns (uint256);
}
```

**流程**:
1. 查询 quote 获取预期输出 amountOut
2. 计算 limit = amountOut * (1 - slippage) / 1
3. ERC-20 approve: token.approve(Vault, amount)  [selector 0x095ea7b3]
4. Vault.swap(singleSwap, funds, limit, deadline)  [selector 0x52bbbe29, --force]

**注意**:
- ETH swap 用 WETH address (`0x82aF49447D8a07e3bd95BD0d56f35241523fBab1` on Arbitrum)
- recipient 必须是钱包地址，永不传零地址
- 必须加 `--force` flag

#### join — 添加流动性

**合约**: Vault `0xBA12222222228d8Ba445958a75a0704d566BF2C8`
**Selector**: `0xb95cac28` (joinPool(bytes32,address,address,(address[],uint256[],bytes,bool))) — verified via `cast sig`

**userData 格式** (WeightedPool JOIN_KIND):
- EXACT_TOKENS_IN_FOR_BPT_OUT: `abi.encode(uint256(1), uint256[] amountsIn, uint256 minimumBPT)`
- INIT: `abi.encode(uint256(0), uint256[] amountsIn)` (first join only)

**流程**:
1. ERC-20 approve all tokens → Vault
2. Vault.joinPool(poolId, sender, recipient, JoinPoolRequest{assets, maxAmountsIn, userData, false})

#### exit — 移除流动性

**合约**: Vault `0xBA12222222228d8Ba445958a75a0704d566BF2C8`
**Selector**: `0x8bdb3913` (exitPool(bytes32,address,address,(address[],uint256[],bytes,bool))) — verified via `cast sig`

**userData 格式** (WeightedPool EXIT_KIND):
- EXACT_BPT_IN_FOR_TOKENS_OUT: `abi.encode(uint256(1), uint256 bptAmountIn)`

**流程**:
1. Vault.exitPool(poolId, sender, recipient, ExitPoolRequest{assets, minAmountsOut, userData, false})

---

## §3 用户场景

### 场景 1：查询 Arbitrum 上的 Balancer 池并报价

**用户说**: "Show me the top Balancer V2 pools on Arbitrum and give me a quote to swap 0.001 WETH for USDC"

**Agent 动作序列**:
1. [链下查询] `balancer-v2 pools --chain 42161` → 通过 Subgraph 查询前 20 个池
2. [链下查询] `balancer-v2 quote --from WETH --to USDC --amount 0.001 --pool 0x64541216...0002 --chain 42161` → 调用 BalancerQueries.querySwap()
3. 返回 amountOut 给用户

### 场景 2：Swap WETH → USDC on Arbitrum

**用户说**: "Swap 0.001 WETH for USDC on Balancer on Arbitrum"

**Agent 动作序列**:
1. [链下查询] resolve wallet address for chain 42161
2. [链下查询] `balancer-v2 quote --from WETH --to USDC --amount 0.001 --chain 42161` → 获取预期 amountOut
3. [链上操作] Dry-run first: `balancer-v2 swap --from WETH --to USDC --amount 0.001 --chain 42161 --dry-run`
4. Ask user to confirm: "Swap 0.001 WETH for ~X USDC on Balancer V2? (fee: Y%)"
5. [链上操作] ERC-20 approve: `onchainos wallet contract-call --chain 42161 --to <WETH> --input-data <approve_calldata> --force`
6. [链上操作] Vault.swap: `onchainos wallet contract-call --chain 42161 --to <Vault> --input-data <swap_calldata> --force`
7. 返回 txHash

### 场景 3：查询 LP 仓位

**用户说**: "What are my Balancer V2 LP positions on Arbitrum?"

**Agent 动作序列**:
1. [链下查询] resolve wallet address for chain 42161
2. [链下查询] `balancer-v2 positions --chain 42161` → 对已知池批量调用 balanceOf(wallet)
3. 返回非零 BPT 余额的池信息

### 场景 4：添加流动性

**用户说**: "Add liquidity to the WBTC/WETH/USDC pool on Balancer V2 on Arbitrum with 1 USDC"

**Agent 动作序列**:
1. [链下查询] 查询 pool-info for pool 0x64541216...0002
2. [链上操作] Dry-run: `balancer-v2 join --pool 0x64541216...0002 --amounts 0,0,1 --chain 42161 --dry-run`
3. Ask user to confirm operation
4. [链上操作] Approve USDC → Vault (`onchainos wallet contract-call --force`)
5. [链上操作] joinPool (`onchainos wallet contract-call --force`)
6. 返回 txHash + BPT received

---

## §4 外部 API 依赖

| API | 用途 | URL |
|-----|------|-----|
| Arbitrum JSON-RPC | eth_call (getPoolTokens, querySwap, balanceOf) | `https://arbitrum-one-rpc.publicnode.com` |
| Ethereum JSON-RPC | eth_call on mainnet | `https://ethereum.publicnode.com` |
| Balancer Subgraph (Arbitrum) | 查询池列表、流动性数据 | `https://api.thegraph.com/subgraphs/name/balancer-labs/balancer-arbitrum-v2` |
| Balancer Subgraph (Ethereum) | 查询池列表 | `https://api.thegraph.com/subgraphs/name/balancer-labs/balancer` |

---

## §5 配置参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `chain` | `42161` | 链 ID (42161=Arbitrum, 1=Ethereum) |
| `slippage` | `0.5` | 滑点百分比 (0.5 = 0.5%) |
| `dry_run` | `false` | 模拟模式，不广播 |
| `deadline` | `now + 300s` | 交易截止时间 (Unix timestamp) |

---

## §6 合约地址

### Arbitrum (42161) — Primary Test Chain

| 合约 | 地址 | 备注 |
|------|------|------|
| Vault | `0xBA12222222228d8Ba445958a75a0704d566BF2C8` | 所有操作的唯一入口 |
| BalancerQueries | `0xE39B5e3B6D74016b2F6A9673D7d7493B6DF549d5` | 链上报价 |
| BalancerHelpers | `0x77d46184d22CA6a3726a2F500c776767b6A3d6Ab` | 辅助工具 |
| WETH | `0x82aF49447D8a07e3bd95BD0d56f35241523fBab1` | Wrapped Ether |
| USDC.e | `0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8` | Bridged USDC |
| USDT | `0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9` | Tether USD |
| WBTC | `0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0F` | Wrapped BTC |

### Known Test Pools (Arbitrum 42161)

| Pool | ID | Tokens | Type |
|------|-----|--------|------|
| WBTC/WETH/USDC.e | `0x64541216bafffeec8ea535bb71fbc927831d0595000100000000000000000002` | WBTC/WETH/USDC.e | WeightedPool (33/33/33) |
| DAI/USDT/USDC.e | `0x1533a3278f3f9141d5f820a184ea4b017fce2382000000000000000000000016` | DAI/USDT/USDC.e | StablePool |
| wstETH/WETH | `0x36bf227d6bac96e2ab1ebb5492ecec69c691943f000200000000000000000316` | wstETH/WETH | WeightedPool |

### Ethereum (1) — Secondary

| 合约 | 地址 |
|------|------|
| Vault | `0xBA12222222228d8Ba445958a75a0704d566BF2C8` |
| BalancerQueries | `0xE39B5e3B6D74016b2F6A9673D7d7493B6DF549d5` |

---

## §7 Function Selectors (全部通过 `cast sig` 验证)

| 函数 | Selector | 来源 |
|------|----------|------|
| `swap((bytes32,uint8,address,address,uint256,bytes),(address,bool,address,bool),uint256,uint256)` | `0x52bbbe29` | cast sig ✅ |
| `batchSwap(uint8,(bytes32,uint256,uint256,uint256,bytes)[],address[],(address,bool,address,bool),int256[],uint256)` | `0x945bcec9` | cast sig ✅ |
| `joinPool(bytes32,address,address,(address[],uint256[],bytes,bool))` | `0xb95cac28` | cast sig ✅ |
| `exitPool(bytes32,address,address,(address[],uint256[],bytes,bool))` | `0x8bdb3913` | cast sig ✅ |
| `getPool(bytes32)` | `0xf6c00927` | cast sig ✅ |
| `getPoolTokens(bytes32)` | `0xf94d4668` | cast sig ✅ |
| `querySwap((bytes32,uint8,address,address,uint256,bytes),(address,bool,address,bool))` | `0xe969f6b3` | cast sig ✅ |
| `queryJoin(bytes32,address,address,(address[],uint256[],bytes,bool))` | `0x9ebbf05d` | cast sig ✅ |
| `queryExit(bytes32,address,address,(address[],uint256[],bytes,bool))` | `0xc7b2c52c` | cast sig ✅ |
| `getNormalizedWeights()` | `0xf89f27ed` | cast sig ✅ |
| `getSwapFeePercentage()` | `0x55c67628` | cast sig ✅ |
| `getPoolId()` | `0x38fff2d0` | cast sig ✅ |
| `totalSupply()` | `0x18160ddd` | cast sig ✅ |
| `balanceOf(address)` | `0x70a08231` | cast sig ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | cast sig ✅ |
| `allowance(address,address)` | `0xdd62ed3e` | cast sig ✅ |
| `decimals()` | `0x313ce567` | cast sig ✅ |
