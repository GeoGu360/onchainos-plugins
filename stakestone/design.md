# StakeStone Plugin Design

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | stakestone |
| dapp_name | StakeStone |
| target_chains | Ethereum (1) |
| category | defi-protocol |
| tags | staking, liquid-staking, stone, omnichain |
| version | 0.1.0 |

## §1 接入可行性调研表

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | No — protocol is pure on-chain contracts |
| SDK 支持哪些技术栈？ | N/A |
| 有 REST API？ | No public REST API found (api.stakestone.io returns Hello World only); docs confirm all ops via eth_call / eth_sendRawTransaction |
| 有官方 Skill？ | No |
| 开源社区有类似 Skill？ | No (similar: Lido liquid staking pattern) |
| 支持哪些链？ | Ethereum mainnet (vault), BNB Chain, Base, Arbitrum (STONE token bridged only) |
| 是否需要 onchainos 广播？ | Yes — stake (deposit + ETH value), requestWithdraw, cancelWithdraw all need on-chain tx |

**接入路径**: Direct on-chain (eth_call for reads, onchainos wallet contract-call for writes), modelled on Lido pattern.

## §2 接口映射

### 5a 需要接入的操作表

| 操作 | 类型 | 优先级 |
|------|------|--------|
| stake | 链上写 (ETH → STONE) | P0 |
| request-withdraw | 链上写 (queue unstake) | P0 |
| cancel-withdraw | 链上写 (cancel queued unstake) | P1 |
| get-rate | 链下查询 (STONE/ETH price) | P0 |
| get-position | 链下查询 (STONE balance + pending withdrawal) | P0 |

### 5b 链下查询表

| 操作 | 合约 / Method | 参数 | 返回 |
|------|--------------|------|------|
| get-rate | StoneVault.currentSharePrice() | none | uint256 price (ETH per STONE, 1e18) |
| get-rate | StoneVault.latestRoundID() | none | uint256 round |
| get-rate | StoneVault.withdrawFeeRate() | none | uint256 rate (div 1e6) |
| get-position | STONE.balanceOf(address) | address | uint256 STONE balance |
| get-position | StoneVault.userReceipts(address) | address | (withdrawRound, withdrawShares, withdrawableAmount) |
| get-position | StoneVault.getVaultAvailableAmount() | none | (idleAmount, investedAmount) |

### 5c 链上写操作表

| 操作 | 合约地址 | 函数签名 | Selector (cast sig ✅) | ABI 参数顺序 |
|------|---------|---------|----------------------|------------|
| stake | `0xA62F9C5af106FeEE069F38dE51098D9d81B90572` (StoneVault) | `deposit()` | `0xd0e30db0` ✅ | no params, ETH value |
| request-withdraw | `0xA62F9C5af106FeEE069F38dE51098D9d81B90572` (StoneVault) | `requestWithdraw(uint256)` | `0x745400c9` ✅ | _shares (STONE amount in wei) |
| cancel-withdraw | `0xA62F9C5af106FeEE069F38dE51098D9d81B90572` (StoneVault) | `cancelWithdraw(uint256)` | `0x9f01f7ba` ✅ | _shares (STONE amount in wei) |

### Key Contract Addresses (Ethereum Mainnet)

| Contract | Address |
|----------|---------|
| StoneVault | `0xA62F9C5af106FeEE069F38dE51098D9d81B90572` |
| STONE token | `0x7122985656e38BDC0302Db86685bb972b145bD3C` |

## §3 用户场景

### 场景 1: 查询当前 STONE 汇率和 APY
用户: "What's the current STONE exchange rate?"
1. eth_call StoneVault.currentSharePrice() → 1.063 ETH per STONE
2. eth_call StoneVault.latestRoundID() → round 274
3. eth_call StoneVault.withdrawFeeRate() → 0%
4. Display rate, round, APY estimate

### 场景 2: 质押 ETH 获取 STONE
用户: "Stake 0.00005 ETH into StakeStone"
1. eth_call currentSharePrice() → show expected STONE
2. Ask user to confirm staking amount and contract
3. onchainos wallet contract-call --chain 1 --to 0xA62F9... --input-data 0xd0e30db0 --amt 50000000000000

### 场景 3: 查询持仓
用户: "Show my StakeStone position"
1. Resolve wallet via onchainos wallet balance --chain 1
2. eth_call STONE.balanceOf(wallet) → STONE balance
3. eth_call StoneVault.userReceipts(wallet) → pending withdrawal info
4. eth_call currentSharePrice() → convert STONE balance to ETH value
5. Display full position summary

### 场景 4: 请求赎回
用户: "Unstake 0.001 STONE from StakeStone"
1. Check STONE balance ≥ requested amount
2. Show user: shares to withdraw, estimated ETH after fee
3. Ask user to confirm
4. onchainos wallet contract-call --chain 1 --to 0xA62F9... --input-data 0x745400c9<SHARES_HEX>

## §4 外部 API 依赖

| API | 用途 | Auth |
|-----|------|------|
| https://ethereum.publicnode.com | eth_call reads on Ethereum mainnet | None |

## §5 配置参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| chain_id | 1 | Ethereum mainnet |
| dry_run | false | Simulate without broadcasting |
| rpc_url | https://ethereum.publicnode.com | Ethereum RPC endpoint |
