# GMX V1 Plugin Design

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `gmx-v1` |
| dapp_name | GMX V1 |
| target_chains | Arbitrum (42161), Avalanche (43114) |
| target_protocols | Perpetuals DEX, Spot Swap, GLP Liquidity |
| bitable_record_id | recvfIURyocgTp |

---

## §1 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | No — no official Rust SDK |
| SDK 支持哪些技术栈？ | JavaScript/TypeScript SDK (@gmx-io/sdk) |
| 有 REST API？ | Yes — https://arbitrum-api.gmxinfra.io (same infra as V2) |
| 有官方 Skill？ | No |
| 开源社区有类似 Skill？ | No dedicated V1 skill; GMX V2 plugin exists in repo |
| 支持哪些链？ | Arbitrum (42161), Avalanche (43114) |
| 是否需要 onchainos 广播？ | Yes — all write ops via onchainos wallet contract-call |

**接入路径：** API + direct ABI encoding (参考 GMX V2 plugin 结构)

**Key contracts on Arbitrum:**
- Router: `0xaBBc5F99639c9B6bCb58544ddf04CF3C176D2B00`
- PositionRouter: `0xb87a436B93fE243ff3BC3ff12dA8dcFF7A5a36a7`
- GlpManager: `0x321F653eED006AD1C29D174e17d96351BDe22649`
- RewardRouter: `0xA906F338CB21815cBc4Bc87ace9e68c87eF8d8F1`
- GLP token: `0x4277f18E69571a1c4f38c37e5f0E4B97B25Fcf7f` (Arbitrum)

**Key contracts on Avalanche:**
- Router: `0x5F719c2F1095F7B9fc68a68e35B51194f4b6abe8`
- PositionRouter: `0x195256074192170d1809527d3c462CF0430Bb4d7`
- GlpManager: `0xe1ae4d4b06A5Fe1fc288f6B4CD72f9F8323B107F`
- RewardRouter: `0x82147C5A7E850eA4E28155DF107F2590fD4ba327`

**V1 vs V2 key differences:**
- V1 uses direct execution (no keeper), simpler contract flow
- V1 swap: Router.swap() — no execution fee required
- V1 GLP: RewardRouter.mintAndStakeGlp() / unstakeAndRedeemGlp() — no execution fee
- V1 perp: PositionRouter.createIncreasePosition() / createDecreasePosition() — requires 0.0001 ETH execution fee
- GLP is the liquidity token (not GM as in V2)

---

## §2 接口映射

### Operations

| 操作 | 类型 | 说明 |
|------|------|------|
| get-prices | 链下查询 | Fetch oracle prices for GMX V1 tokens |
| get-positions | 链下查询 | Fetch open perp positions for wallet |
| swap | 链上写操作 | Swap tokens via Router.swap() |
| buy-glp | 链上写操作 | Mint GLP via RewardRouter.mintAndStakeGlp() |
| sell-glp | 链上写操作 | Redeem GLP via RewardRouter.unstakeAndRedeemGlp() |
| open-position | 链上写操作 | Open leveraged perp via PositionRouter.createIncreasePosition() |
| close-position | 链上写操作 | Close perp via PositionRouter.createDecreasePosition() |
| approve-token | 链上写操作 | ERC-20 approve for Router/GlpManager |

### 链下查询

| 操作 | API Endpoint | 参数 | 返回值 |
|------|-------------|------|--------|
| get-prices | `GET {api_base}/prices/tickers` | — | array of {tokenSymbol, minPrice, maxPrice, tokenAddress} |
| get-positions | `GET {api_base}/positions?account={addr}` | account | array of position objects |

### 链上写操作

| 操作 | 合约地址 | 函数签名 | Selector | ABI 参数顺序 |
|------|---------|---------|---------|------------|
| swap (token→token) | Router `0xaBBc5F99639c9B6bCb58544ddf04CF3C176D2B00` | `swap(address[],uint256,uint256,address)` | `0x6023e966` ✅ | path[], amountIn, minOut, receiver |
| swap (ETH→token) | Router `0xaBBc5F99639c9B6bCb58544ddf04CF3C176D2B00` | `swapETHToTokens(address[],uint256,address)` | `0xabe68eaa` ✅ | path[], minOut, receiver; ETH via --amt |
| swap (token→ETH) | Router `0xaBBc5F99639c9B6bCb58544ddf04CF3C176D2B00` | `swapTokensToETH(address[],uint256,uint256,address payable)` | `0x2d4ba6a7` ✅ | path[], amountIn, minOut, receiver |
| buy-glp | RewardRouter `0xA906F338CB21815cBc4Bc87ace9e68c87eF8d8F1` | `mintAndStakeGlp(address,uint256,uint256,uint256)` | `0x364e2311` ✅ | token, amount, minUsdg, minGlp |
| sell-glp | RewardRouter `0xA906F338CB21815cBc4Bc87ace9e68c87eF8d8F1` | `unstakeAndRedeemGlp(address,uint256,uint256,address)` | `0x0f3aa554` ✅ | tokenOut, glpAmount, minOut, receiver |
| open-position | PositionRouter `0xb87a436B93fE243ff3BC3ff12dA8dcFF7A5a36a7` | `createIncreasePosition(address[],address,uint256,uint256,uint256,bool,uint256,uint256,bytes32,address)` | `0xf2ae372f` ✅ | path[], indexToken, amountIn, minOut, sizeDelta, isLong, acceptablePrice, executionFee, referralCode, callbackTarget; ETH value = executionFee (0.0001 ETH) |
| close-position | PositionRouter `0xb87a436B93fE243ff3BC3ff12dA8dcFF7A5a36a7` | `createDecreasePosition(address[],address,uint256,uint256,bool,address,uint256,uint256,uint256,bool,address)` | `0x7be7d141` ✅ | path[], indexToken, collateralDelta, sizeDelta, isLong, receiver, acceptablePrice, minOut, executionFee, withdrawETH, callbackTarget; ETH value = executionFee |
| approve-token | ERC-20 token | `approve(address,uint256)` | `0x095ea7b3` ✅ | spender, amount |

**Note on execution fees:**
- swap: NO execution fee required
- buy-glp / sell-glp: NO execution fee required (direct execution)
- open-position / close-position: 0.0001 ETH (100,000,000,000,000 wei) execution fee — exceeds GUARDRAILS 0.00005 ETH L4 limit → skip L4 tests for these

---

## §3 用户场景

### 场景 1: 查询 GMX V1 token 价格
用户说: "Show me current token prices on GMX V1 Arbitrum"
1. 调用 get-prices --chain 42161
2. 解析 /prices/tickers API
3. 显示 token symbol, min/max USD 价格

### 场景 2: 用 USDC 换 WETH (swap)
用户说: "Swap 10 USDC to WETH on GMX V1 Arbitrum"
1. approve USDC to Router (if needed): approve-token --token USDC-addr --spender Router-addr
2. swap --chain 42161 --input-token USDC-addr --input-amount 10000000 --output-token WETH-addr --min-output 0
3. 编码 Router.swap([USDC, WETH], 10000000, 0, wallet) calldata
4. 提交 wallet contract-call

### 场景 3: 买 GLP (buy-glp)
用户说: "Buy GLP with 5 USDC on GMX V1"
1. approve USDC to GlpManager (if needed)
2. buy-glp --chain 42161 --token USDC-addr --amount 5000000 --min-usdg 0 --min-glp 0
3. 编码 RewardRouter.mintAndStakeGlp(USDC, 5000000, 0, 0) calldata
4. 提交 wallet contract-call

### 场景 4: 查询持仓
用户说: "Show my open positions on GMX V1"
1. resolve wallet address via onchainos
2. get-positions --chain 42161
3. 调用 /positions?account={wallet} API
4. 显示 market, direction, size, collateral, PnL

### 场景 5: 开多单 (dry-run only)
用户说: "Open a $1000 long ETH position on GMX V1 Arbitrum (dry run)"
1. open-position --chain 42161 --index-token WETH --collateral-token USDC --amount 5000000 --size-delta 1000 --is-long true --dry-run
2. 验证 calldata 结构 (selector 0xf2ae372f)

---

## §4 外部 API 依赖

| API | Base URL | Usage |
|-----|---------|-------|
| GMX Arbitrum API | `https://arbitrum-api.gmxinfra.io` | prices, positions |
| GMX Avalanche API | `https://avalanche-api.gmxinfra.io` | prices, positions |

---

## §5 配置参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| chain | 42161 | Chain ID (42161=Arbitrum, 43114=Avalanche) |
| dry_run | false | If true, print calldata without submitting |
| execution_fee | 100000000000000 | 0.0001 ETH in wei (for position operations) |
