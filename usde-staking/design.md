# USDe Staking (Ethena sUSDe) — Plugin Design

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `usde-staking` |
| dapp_name | USDe Staking (Ethena sUSDe) |
| target_chains | Ethereum mainnet (chain ID 1) |
| category | defi-protocol |
| tags | staking, yield, usde, susde, ethena, erc-4626 |
| bitable_record_id | recvfIV8pLOdVG |

---

## §1 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 无官方 Rust SDK |
| SDK 支持哪些技术栈？ | JS/TS SDK 仅有非官方实现 |
| 有 REST API？ | 是 — https://app.ethena.fi/api/ (yield data, no auth) |
| 有官方 Skill？ | 无 |
| 开源社区有类似 Skill？ | 无，需要从零开始 |
| 支持哪些链？ | Ethereum mainnet (chain 1) only |
| 是否需要 onchainos 广播？ | Yes — stake/unstake/claim are on-chain ERC-4626 operations |

**接入路径：** API (REST for yield data) + 直接 eth_call (on-chain reads) + onchainos (write ops)

---

## §2 接口映射

### 需要接入的操作

| 操作 | 类型 | 优先级 |
|------|------|--------|
| stake (deposit USDe → sUSDe) | 链上写 (ERC-4626 deposit) | P0 |
| request-unstake (initiate cooldown) | 链上写 (cooldownShares/cooldownAssets) | P0 |
| claim-unstake (claim after cooldown) | 链上写 (unstake) | P0 |
| get-rates (APY, exchange rate) | 链下查询 | P0 |
| get-positions (sUSDe balance, pending) | 链下查询 | P0 |

### 链下查询

| 操作 | 方式 | 关键参数 | 返回值 |
|------|------|---------|-------|
| get-rates | REST GET https://app.ethena.fi/api/yields/protocol-and-staking-yield | — | stakingYield, avg30dSusdeYield, avg90dSusdeYield |
| get-rates (exchange rate) | eth_call convertToAssets(1e18) | — | USDe per sUSDe |
| get-positions | eth_call balanceOf(wallet) on sUSDe | wallet address | sUSDe balance |
| get-positions (pending) | eth_call cooldowns(wallet) on sUSDe | wallet address | cooldownEnd, underlyingAmount |

### 链上写操作

| 操作 | 合约地址 | 函数签名 | Selector | ABI 参数顺序 |
|------|---------|---------|---------|------------|
| approve USDe | `0x4c9EDD5852cd905f086C759E8383e09bff1E68B3` (USDe token) | `approve(address,uint256)` | `0x095ea7b3` ✅ | spender(sUSDe addr), amount |
| stake | `0x9D39A5DE30e57443BfF2A8307A4256c8797A3497` (sUSDe ERC-4626) | `deposit(uint256,address)` | `0x6e553f65` ✅ | assets(USDe amount in wei), receiver(wallet) |
| request-unstake (shares) | `0x9D39A5DE30e57443BfF2A8307A4256c8797A3497` (sUSDe) | `cooldownShares(uint256)` | `0x9343d9e1` ✅ | shares(sUSDe amount in wei) |
| request-unstake (assets) | `0x9D39A5DE30e57443BfF2A8307A4256c8797A3497` (sUSDe) | `cooldownAssets(uint256)` | `0xcdac52ed` ✅ | assets(USDe amount in wei) |
| claim-unstake | `0x9D39A5DE30e57443BfF2A8307A4256c8797A3497` (sUSDe) | `unstake(address)` | `0xf2888dbb` ✅ | receiver(wallet) |

**Notes:**
- ERC-4626 `deposit(uint256 assets, address receiver)` — assets is USDe amount, receiver is wallet
- cooldown duration: 86400 seconds (1 day) as of April 2026
- `cooldownShares(uint256 shares)` — initiate unstake by specifying sUSDe share amount
- `cooldownAssets(uint256 assets)` — initiate unstake by specifying USDe asset amount
- `unstake(address receiver)` — claim USDe after cooldown expires
- 2-tx flow for stake: approve USDe → deposit (add 15s delay between them)

---

## §3 用户场景

### 场景 1: 查询 sUSDe 收益率和汇率

**用户说:** "What is the current sUSDe staking yield?"

**Agent 动作:**
1. [链下查询] GET https://app.ethena.fi/api/yields/protocol-and-staking-yield → stakingYield, avg30d
2. [eth_call] convertToAssets(1e18) on sUSDe → USDe per sUSDe exchange rate
3. 返回: current APY, 30d avg, 90d avg, exchange rate

### 场景 2: 质押 USDe 获得 sUSDe

**用户说:** "Stake 100 USDe to get sUSDe"

**Agent 动作:**
1. [链下查询] eth_call balanceOf(wallet) on USDe → check sufficient balance
2. [链下查询] previewDeposit(100 USDe) → expected sUSDe output
3. [确认] 向用户显示: amount, expected sUSDe, current APY → ask user to confirm
4. [链上写] approve USDe → sUSDe contract (if allowance insufficient)
5. [等待 15s]
6. [链上写] deposit(100 USDe, wallet) on sUSDe → returns sUSDe shares

### 场景 3: 发起解质押请求

**用户说:** "Unstake 50 sUSDe"

**Agent 动作:**
1. [链下查询] eth_call balanceOf(wallet) on sUSDe → verify sufficient sUSDe
2. [确认] 告知用户 cooldown period (1 day) → ask user to confirm
3. [链上写] cooldownShares(50 sUSDe) → initiates cooldown
4. 返回: cooldown end timestamp, claim instructions

### 场景 4: 查询持仓

**用户说:** "Show my sUSDe staking position"

**Agent 动作:**
1. [链下查询] eth_call balanceOf(wallet) on sUSDe → current sUSDe balance
2. [链下查询] eth_call convertToAssets(sUSDe_balance) → equivalent USDe value
3. [链下查询] eth_call cooldowns(wallet) on sUSDe → any pending unstake
4. 返回: sUSDe balance, USDe equivalent, pending unstake if any

### 场景 5: 领取解质押后的 USDe

**用户说:** "Claim my unstaked USDe"

**Agent 动作:**
1. [链下查询] eth_call cooldowns(wallet) → verify cooldown has expired
2. [确认] ask user to confirm claim
3. [链上写] unstake(wallet) → receive USDe

---

## §4 外部 API 依赖

| API | 用途 | 认证 |
|-----|------|------|
| https://app.ethena.fi/api/yields/protocol-and-staking-yield | 获取当前 APY 和历史收益率 | 无 |
| https://ethereum.publicnode.com | Ethereum mainnet RPC (eth_call) | 无 |

---

## §5 配置参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| chain_id | 1 | Ethereum mainnet |
| usde_address | `0x4c9EDD5852cd905f086C759E8383e09bff1E68B3` | USDe token |
| susde_address | `0x9D39A5DE30e57443BfF2A8307A4256c8797A3497` | sUSDe ERC-4626 vault |
| rpc_url | `https://ethereum.publicnode.com` | Public Ethereum RPC |
| dry_run | false | Simulate without broadcasting |
| cooldown_seconds | 86400 | 1 day (from contract) |
