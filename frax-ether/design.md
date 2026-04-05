# Frax Ether Plugin — Design Document

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `frax-ether` |
| dapp_name | Frax Ether |
| target_chains | Ethereum mainnet (chain ID: 1) |
| target_protocols | frxETHMinter, sfrxETH (ERC-4626) |
| category | defi-protocol |
| tags | liquid-staking, frxETH, sfrxETH, ERC-4626 |

---

## §1 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 无专用 Rust SDK；合约调用通过 ABI 编码 |
| SDK 支持哪些技术栈？ | JavaScript/TypeScript (frax.finance SDK); 本插件直接 ABI 编码 |
| 有 REST API？ | 是 — https://api.frax.finance/v2/frxeth/summary/history (APR/price data) |
| 有官方 Skill？ | 无 |
| 开源社区有类似 Skill？ | Lido liquid staking 可参考 (同类 ETH → liquid token 模式) |
| 支持哪些链？ | Ethereum mainnet 仅 (chain ID: 1) |
| 是否需要 onchainos 广播？ | 是 — ETH deposit 和 frxETH→sfrxETH deposit 均需链上广播 |

**接入路径**：直接 ABI 编码合约调用（无 SDK），通过 `onchainos wallet contract-call` 广播。

---

## §2 接口映射

### 需要接入的操作表

| 操作 | 类型 | 说明 |
|------|------|------|
| stake | 链上写 | ETH → frxETH (frxETHMinter.submit) |
| stake-frx | 链上写 | frxETH → sfrxETH (ERC-4626 deposit) |
| unstake | 链上写 | sfrxETH → frxETH (ERC-4626 redeem) |
| rates | 链下查询 | 查询 sfrxETH APR 和当前汇率 |
| positions | 链下查询 | 查询 frxETH + sfrxETH 余额和价值 |

---

### 链下查询表

#### `rates` — 获取 sfrxETH APR 和汇率

- **API Endpoint**: `GET https://api.frax.finance/v2/frxeth/summary/history?range=1d`
- **关键参数**: 无
- **返回值**: `sfrxethApr` (APR as percentage), `sfrxethFrxethPrice` (sfrxETH per frxETH exchange rate)
- **链上补充**: `convertToAssets(1e18)` on sfrxETH to get current price

Verified response sample:
```json
{
  "sfrxethApr": 2.8547773,
  "sfrxethFrxethPrice": 1.1547058,
  "frxethEthCurvePrice": 0.9978599,
  "ethPriceUsd": 2032.63
}
```

#### `positions` — 查询用户持仓

- **eth_call**: `balanceOf(address)` on frxETH (0x5E8422345238F34275888049021821E8E08CAa1f)
  - Selector: `0x70a08231` (cast sig "balanceOf(address)" ✅)
- **eth_call**: `balanceOf(address)` on sfrxETH (0xac3E018457B222d93114458476f3E3416Abbe38F)
  - Selector: `0x70a08231`
- **eth_call**: `convertToAssets(uint256)` on sfrxETH to get frxETH value of sfrxETH
  - Selector: `0x07a2d13a` (cast sig "convertToAssets(uint256)" ✅)

---

### 链上写操作表

#### `stake` — ETH → frxETH

**Contract**: frxETHMinter `0xbAFA44EFE7901E04E39Dad13167D089C559c1138`

**Function**: `submit(address referral)` — payable, deposit ETH, receive frxETH

**Selector**: `0xa1903eab` (cast sig "submit(address)" ✅)

**Calldata construction**:
```
0xa1903eab
+ address padded 32 bytes (referral = zero address 0x0000...0000)
```

**onchainos command**:
```bash
onchainos wallet contract-call \
  --chain 1 \
  --to 0xbAFA44EFE7901E04E39Dad13167D089C559c1138 \
  --input-data 0xa1903eab0000000000000000000000000000000000000000000000000000000000000000 \
  --amt <wei_amount>
```

Note: No ERC-20 approve needed (native ETH deposit).

---

#### `stake-frx` — frxETH → sfrxETH

**Contract**: sfrxETH (ERC-4626) `0xac3E018457B222d93114458476f3E3416Abbe38F`

**Step 1 — ERC-20 approve**:
- Token: frxETH `0x5E8422345238F34275888049021821E8E08CAa1f`
- Spender: sfrxETH `0xac3E018457B222d93114458476f3E3416Abbe38F`
- Selector: `0x095ea7b3` (standard approve)

**Step 2 — ERC-4626 deposit**:
- Function: `deposit(uint256 assets, address receiver)`
- Selector: `0x6e553f65` (cast sig "deposit(uint256,address)" ✅)
- `assets` = frxETH amount in wei
- `receiver` = user wallet address

**onchainos commands**:
```bash
# Step 1: approve
onchainos wallet contract-call \
  --chain 1 \
  --to 0x5E8422345238F34275888049021821E8E08CAa1f \
  --input-data 0x095ea7b3<spender_padded><amount_hex>

# Step 2: deposit
onchainos wallet contract-call \
  --chain 1 \
  --to 0xac3E018457B222d93114458476f3E3416Abbe38F \
  --input-data 0x6e553f65<amount_padded><receiver_padded>
```

---

#### `unstake` — sfrxETH → frxETH

**Contract**: sfrxETH (ERC-4626) `0xac3E018457B222d93114458476f3E3416Abbe38F`

**Function**: `redeem(uint256 shares, address receiver, address owner)`
- Selector: `0xba087652` (cast sig "redeem(uint256,address,address)" ✅)
- `shares` = sfrxETH amount in wei
- `receiver` = user wallet (receives frxETH)
- `owner` = user wallet (owns the sfrxETH)

**onchainos command**:
```bash
onchainos wallet contract-call \
  --chain 1 \
  --to 0xac3E018457B222d93114458476f3E3416Abbe38F \
  --input-data 0xba087652<shares_padded><receiver_padded><owner_padded>
```

---

## §3 用户场景

### 场景 1: 质押 ETH 获得 frxETH

**用户说**: "stake 0.00005 ETH to get frxETH on Frax"

**Agent 动作**:
1. 链下: 检查用户 ETH 余额
2. 计算 amount = 50000000000000 wei (0.00005 ETH)
3. 构造 calldata: `0xa1903eab` + zero address (referral)
4. **Ask user to confirm** before proceeding
5. 链上: `onchainos wallet contract-call --chain 1 --to 0xbAFA44EFE7901E04E39Dad13167D089C559c1138 --input-data 0xa1903eab... --amt 50000000000000`
6. 返回 txHash 和收到的 frxETH 数量

### 场景 2: 质押 frxETH 获得收益 (sfrxETH)

**用户说**: "stake my frxETH to earn yield with Frax"

**Agent 动作**:
1. 链下: 查询用户 frxETH 余额
2. 确认数量 (如 0.00005 frxETH = 50000000000000 wei)
3. Step 1 — **Ask user to confirm** approve
4. 链上: ERC-20 approve frxETH to sfrxETH
5. Step 2 — **Ask user to confirm** deposit
6. 链上: ERC-4626 deposit frxETH → sfrxETH
7. 返回收到的 sfrxETH 数量和当前 APR

### 场景 3: 查询收益率和汇率

**用户说**: "what is the current sfrxETH APR and exchange rate?"

**Agent 动作**:
1. 链下: GET `https://api.frax.finance/v2/frxeth/summary/history?range=1d`
2. 提取 `sfrxethApr`, `sfrxethFrxethPrice`
3. 链下: eth_call `convertToAssets(1e18)` on sfrxETH
4. 返回 APR (%), 当前 sfrxETH/frxETH 比率, ETH 价格

### 场景 4: 查询持仓

**用户说**: "show my Frax ETH positions"

**Agent 动作**:
1. 解析用户钱包地址 via `onchainos wallet addresses`
2. eth_call `balanceOf(address)` on frxETH contract
3. eth_call `balanceOf(address)` on sfrxETH contract
4. eth_call `convertToAssets(sfrxethBalance)` to get frxETH value of sfrxETH
5. 返回 frxETH balance, sfrxETH balance, underlying frxETH value, total USD value

### 场景 5: 赎回 sfrxETH 换回 frxETH

**用户说**: "unstake my sfrxETH back to frxETH"

**Agent 动作**:
1. 查询 sfrxETH 余额
2. **Ask user to confirm** before proceeding
3. 链上: ERC-4626 redeem sfrxETH → frxETH
4. 返回收到的 frxETH 数量和 txHash

---

## §4 外部 API 依赖

| API | 用途 |
|-----|------|
| `https://api.frax.finance/v2/frxeth/summary/history` | sfrxETH APR, 汇率, ETH 价格 |
| `https://ethereum.publicnode.com` | Ethereum mainnet RPC (eth_call for balances) |

---

## §5 配置参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| chain | 1 | Ethereum mainnet |
| dry_run | false | 模拟模式，不广播 |
| frxETHMinter | `0xbAFA44EFE7901E04E39Dad13167D089C559c1138` | frxETHMinter 合约地址 |
| frxETH | `0x5E8422345238F34275888049021821E8E08CAa1f` | frxETH ERC-20 token |
| sfrxETH | `0xac3E018457B222d93114458476f3E3416Abbe38F` | sfrxETH ERC-4626 vault |

---

## §6 Key Contract Addresses (Ethereum Mainnet)

| Contract | Address | Verified |
|----------|---------|---------|
| frxETHMinter | `0xbAFA44EFE7901E04E39Dad13167D089C559c1138` | ✅ Etherscan |
| frxETH | `0x5E8422345238F34275888049021821E8E08CAa1f` | ✅ totalSupply 72236 ETH verified |
| sfrxETH | `0xac3E018457B222d93114458476f3E3416Abbe38F` | ✅ totalAssets confirmed, APR 2.85% |

---

## §7 Function Selector Verification

| Function Signature | cast sig Result | Status |
|-------------------|----------------|--------|
| `submit()` | `0x5bcb2fc6` | ✅ (submit(address) reverts; submit() works) |
| `deposit(uint256,address)` | `0x6e553f65` | ✅ |
| `redeem(uint256,address,address)` | `0xba087652` | ✅ |
| `convertToAssets(uint256)` | `0x07a2d13a` | ✅ |
| `convertToShares(uint256)` | `0xc6e6f592` | ✅ |
| `balanceOf(address)` | `0x70a08231` | ✅ |
| `totalAssets()` | `0x01e1d114` | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | ✅ |
| `pricePerShare()` | `0x99530b06` | ✅ (sfrxETH convenience fn) |
