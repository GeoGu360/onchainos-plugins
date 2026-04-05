# LayerBank Plugin Design Document

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | layer-bank |
| dapp_name | LayerBank |
| target_chains | Scroll (534352) |
| target_protocols | LayerBank V2 |
| version | 0.1.0 |

## §1 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | ❌ 无 Rust SDK |
| SDK 支持哪些技术栈？ | 仅 Solidity 合约 |
| 有 REST API？ | ❌ 无官方 REST API |
| 有官方 Skill？ | ❌ 无 |
| 开源社区有类似 Skill？ | ❌ 无 |
| 支持哪些链？ | Linea, Scroll, Manta, Mode, zkLink, BNB, Morph, Hemi, Rootstock 等；**不含 Base (8453)** |
| 是否需要 onchainos 广播？ | Yes — supply/withdraw/borrow/repay 均为链上操作 |

**接入路径:** 直接 ABI 编码调用合约（无 SDK，无 API）

## §2 接口映射

### 链下查询

| 操作 | 合约/选择器 | 参数 | 返回值 |
|------|-----------|------|--------|
| 获取所有市场 | Core.allMarkets() `0x375a7cba` | — | address[] |
| 获取市场信息 | Core.marketInfoOf(lToken) `0x6e8584fd` | lToken address | (isListed,supplyCap,borrowCap,collateralFactor) |
| 获取账户流动性 | Core.accountLiquidityOf(account) `0xf8982e7a` | wallet address | (collateralInUSD,supplyInUSD,borrowInUSD) |
| lToken 汇率 | lToken.exchangeRate() `0x3ba0b9a9` | — | uint256 (1e18 scaled) |
| 总借款 | lToken.totalBorrow() `0x8285ef40` | — | uint256 |
| 可用流动性 | lToken.getCash() `0x3b1d21a2` | — | uint256 |
| lToken 供应量 | lToken.totalSupply() `0x18160ddd` | — | uint256 |
| 账户快照 | lToken.accountSnapshot(addr) `0x014a296f` | wallet address | (lTokenBal,borrowBal,exchangeRate) |
| 资产价格 | PriceCalculator.getUnderlyingPrice(lToken) `0xfc57d4df` | lToken address | uint256 (1e18 USD) |
| lToken 余额 | lToken.balanceOf(addr) `0x70a08231` | wallet address | uint256 |
| 借款余额 | lToken.borrowBalanceOf(addr) `0x374c49b4` | wallet address | uint256 |

### 链上写操作

| 操作 | 合约 | 选择器 | 参数 |
|------|------|--------|------|
| supply ETH | Core | `0xf2b9fdb8` | (lETH addr padded, 0) + msg.value=amount |
| supply ERC-20 | ERC-20.approve + Core | approve:`0x095ea7b3` + supply:`0xf2b9fdb8` | (lToken addr, amount) |
| withdraw | Core | `0x96294178` | (lToken addr, uAmount) |
| borrow | Core | `0x4b8a3529` | (lToken addr, amount) |
| repay ETH | Core | `0xabdb5ea8` | (lETH addr, amount) + msg.value |
| repay ERC-20 | ERC-20.approve + Core | approve:`0x095ea7b3` + repay:`0xabdb5ea8` | (lToken addr, amount) |

### Selector 验证 (via eth_utils keccak256)

| 函数签名 | 选择器 | 验证状态 |
|---------|--------|---------|
| supply(address,uint256) | 0xf2b9fdb8 | ✅ eth_utils |
| borrow(address,uint256) | 0x4b8a3529 | ✅ eth_utils |
| redeemUnderlying(address,uint256) | 0x96294178 | ✅ eth_utils |
| repayBorrow(address,uint256) | 0xabdb5ea8 | ✅ eth_utils |
| allMarkets() | 0x375a7cba | ✅ eth_utils + live test |
| accountLiquidityOf(address) | 0xf8982e7a | ✅ eth_utils + live test |
| marketInfoOf(address) | 0x6e8584fd | ✅ eth_utils + live test |
| getUnderlyingPrice(address) | 0xfc57d4df | ✅ eth_utils + live test |
| exchangeRate() | 0x3ba0b9a9 | ✅ live test (returns ~1.02) |
| totalBorrow() | 0x8285ef40 | ✅ live test (returns 46 ETH) |
| getCash() | 0x3b1d21a2 | ✅ live test (returns 52 ETH) |
| accountSnapshot(address) | 0x014a296f | ✅ eth_utils + live test |
| borrowBalanceOf(address) | 0x374c49b4 | ✅ eth_utils + live test |

## §3 用户场景

### 场景 1: 查询市场
用户: "LayerBank 上有哪些可借贷资产？"
1. 调用 `markets --chain 534352`
2. 遍历 MARKETS 列表，eth_call 各 lToken 汇率/借款/cash
3. 调用 PriceCalculator.getUnderlyingPrice 获取价格
4. 返回市场列表 JSON

### 场景 2: 查看持仓
用户: "我在 LayerBank 的抵押物和借款是多少？"
1. 解析钱包地址
2. Core.accountLiquidityOf(wallet) 获取总 USD 值
3. 遍历各 lToken.accountSnapshot(wallet) 获取明细
4. 计算健康因子 = collateral / borrow
5. 返回持仓汇总

### 场景 3: 存入 USDC
用户: "帮我向 LayerBank 存入 0.01 USDC"
1. 获取用户确认
2. dry-run 验证 calldata
3. ERC-20.approve(Core, 0.01 USDC)
4. 等待 3 秒
5. Core.supply(lUSDC, 10000) // 0.01 USDC = 10000 raw
6. 返回 txHash

## §4 外部 API 依赖

- `https://rpc.scroll.io` — Scroll mainnet JSON-RPC

## §5 配置参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| chain | 534352 | Scroll mainnet chain ID |
| dry_run | false | 模拟模式 |
| from | (logged-in wallet) | 发送方地址 |
