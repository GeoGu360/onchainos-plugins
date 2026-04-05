# Mellow LRT Plugin Design

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | mellow-lrt |
| dapp_name | Mellow LRT (Mellow Protocol) |
| target_chains | EVM (Ethereum mainnet, chain ID 1) |
| target_protocols | ERC-4626 Vaults, EigenLayer/Symbiotic restaking |
| category | restaking |
| tags | lrt, restaking, liquid-restaking, yield, erc4626, symbiotic |
| version | 0.1.0 |

---

## §1 接入可行性调研表

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 否 — 无官方 Rust SDK |
| SDK 支持哪些技术栈？ | Solidity 合约 (GitHub: mellow-finance/simple-lrt) |
| 有 REST API？ | 是 — `https://api.mellow.finance/v1/` (verified working) |
| 有官方 Skill？ | 否 |
| 开源社区有类似 Skill？ | 否 |
| 支持哪些链？ | Ethereum mainnet (chain 1) — primary; also Holesky testnet |
| 是否需要 onchainos 广播？ | 是 — deposit 和 redeem 是链上写操作 |

**接入路径**: API (REST API for read queries + direct ERC-4626 contract calls for write ops)

---

## §2 接口映射

### 2a 需要接入的操作

| 操作 | 类型 | 描述 |
|------|------|------|
| vaults | 链下查询 | 列出所有 Mellow LRT vault（名称、地址、APR、TVL、underlying token） |
| positions | 链下查询 | 查询用户在 Mellow vault 中的持仓（余额、可赎回、待处理金额） |
| deposit | 链上写操作 | 存入 ETH/wstETH/stETH/WETH 到 Mellow vault（ERC-4626 deposit 或 EthWrapper） |
| withdraw | 链上写操作 | 赎回 LRT shares（ERC-4626 redeem，启动 2-step 提款流程） |
| claim | 链上写操作 | 领取已完成提款申请的 wstETH（2 epochs 后可领取） |

### 2b 链下查询表

#### `vaults` — 列出所有 vault

- **API Endpoint**: `GET https://api.mellow.finance/v1/vaults`
- **参数**: 无
- **返回**: 83 个 vault（含 71 个 Ethereum vault）
- **关键字段**:
  - `id`: vault ID（如 "ethereum-steaklrt"）
  - `address`: vault 合约地址
  - `symbol`: LRT token symbol（如 "steakLRT", "Re7LRT"）
  - `name`: 全名
  - `base_token`: underlying token（address, symbol, decimals）
  - `deposit_tokens[]`: 支持的存款 token（含可选 wrapper 地址）
  - `withdraw_tokens[]`: 提款 token
  - `apr`: 年化收益率（%）
  - `tvl_usd`: TVL (USD)
  - `price`: LRT token 价格（USD）
  - `chain_id`: 链 ID（筛选用 chain_id == 1）
  - `limit_usd`: 存款上限（0 表示无限制）

**实际响应样本**（steakLRT）:
```json
{
  "id": "ethereum-steaklrt",
  "type": "multi-vault",
  "chain_id": 1,
  "address": "0xBEEF69Ac7870777598A04B2bd4771c71212E6aBc",
  "symbol": "steakLRT",
  "name": "Steakhouse Resteaking Vault",
  "base_token": {"address": "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0", "symbol": "wstETH", "decimals": 18},
  "deposit_tokens": [
    {"address": "0x7f39...", "symbol": "wstETH"},
    {"address": "0xae7ab...", "symbol": "stETH", "wrapper": "0x83F6c979ce7a52C7027F08119222825E5bd50351"},
    {"address": "0xC02...", "symbol": "WETH", "wrapper": "0x83F6c979ce7a52C7027F08119222825E5bd50351"},
    {"address": "0x0000...0000", "symbol": "ETH", "wrapper": "0x83F6c979ce7a52C7027F08119222825E5bd50351"}
  ],
  "apr": 2.421,
  "tvl_usd": 2356721,
  "price": 2512.19
}
```

#### `positions` — 用户持仓

- **合约读取**（on-chain eth_call）:
  - `balanceOf(address)` → shares balance: selector `0x70a08231`
  - `convertToAssets(uint256)` → shares → assets: selector `0x07a2d13a`
  - `claimableAssetsOf(address)` → 可立即领取资产: selector `0xe7beaf9d`
  - `pendingAssetsOf(address)` → 待处理中资产: selector `0x63c6b4eb`

**注意**: claimableAssetsOf 和 pendingAssetsOf 仅在 MellowSymbioticVault 合约上可用。对于 vaults API 返回的 type="multi-vault" 和 type="dvv-vault" 合约，需验证这些函数是否存在。若 eth_call 失败则显示 N/A。

### 2c 链上写操作表

#### `deposit` — 存入 ETH/wstETH 到 vault

**两种路径**：

**路径 A: 直接存入 wstETH（最简单）**
1. `approve(vault, amount)` on wstETH token
   - 合约: wstETH = `0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0`
   - Calldata: `0x095ea7b3` + vault_address_padded + amount_hex
   - selector 来源: ERC-20 标准
   
2. `deposit(uint256 assets, address receiver)` on vault
   - selector: `0x6e553f65` (ERC-4626 标准)
   - 来源: `cast sig "deposit(uint256,address)"` → `0x6e553f65` ✅
   - Calldata: `0x6e553f65` + amount_hex_64 + receiver_address_padded

**路径 B: 通过 EthWrapper 存入 ETH/WETH/stETH（对用户更友好）**
- 合约: EthWrapper = `0x83F6c979ce7a52C7027F08119222825E5bd50351`
- 函数: `deposit(address depositToken, uint256 amount, address vault, address receiver, address referral) payable`
- selector: `0x0bb9f5e1`
- 来源: `cast sig "deposit(address,uint256,address,address,address)"` → `0x0bb9f5e1` ✅
- 对 ETH 存款: depositToken = `0x0000000000000000000000000000000000000000` (零地址), msg.value = amount
- 对 WETH/stETH: 先 approve EthWrapper, 然后 deposit
- referral 地址可以填零地址 `0x0000000000000000000000000000000000000000`

**对于 ETH deposit（我们的测试路径）**:
- to: `0x83F6c979ce7a52C7027F08119222825E5bd50351` (EthWrapper)
- calldata: `0x0bb9f5e1` + ETH_ADDR_padded + amount_64 + vault_padded + receiver_padded + referral_padded
- value: amount in wei (--amt)
- 无需事先 approve（直接发 ETH）

**onchainos 命令（ETH deposit via wrapper）**:
```bash
onchainos wallet contract-call \
  --chain 1 \
  --to 0x83F6c979ce7a52C7027F08119222825E5bd50351 \
  --input-data "0x0bb9f5e1{eth_addr_pad}{amount_pad}{vault_pad}{receiver_pad}{referral_pad}" \
  --amt {amount_in_wei}
```

#### `withdraw` — 赎回 LRT shares

- 函数: `redeem(uint256 shares, address receiver, address owner)`
- selector: `0xba087652` (ERC-4626 标准)
- 来源: `cast sig "redeem(uint256,address,address)"` → `0xba087652` ✅
- Calldata: `0xba087652` + shares_hex_64 + receiver_padded + owner_padded
- **注意**: 这是异步提款的第一步。资金可能不会立即返回，可能进入 pending queue

**onchainos 命令**:
```bash
onchainos wallet contract-call \
  --chain 1 \
  --to {vault_address} \
  --input-data "0xba087652{shares_hex}{receiver_pad}{owner_pad}"
```

#### `claim` — 领取已解锁的提款

- 函数: `claim(address account, address recipient, uint256 maxAmount)`
- selector: `0x996cba68`
- 来源: `cast sig "claim(address,address,uint256)"` → `0x996cba68` ✅
- **注意**: claim 函数在 withdrawal queue 合约上，不在主 vault 合约上
- 通常需要先通过 claimableAssetsOf 检查可领取金额

---

## §3 用户场景

### 场景 1：查询所有 Mellow LRT vault 并比较收益

**用户说**: "Show me all Mellow LRT vaults with their APR and TVL"

**Agent 动作序列**:
1. [链下查询] `GET https://api.mellow.finance/v1/vaults`
2. 过滤 chain_id == 1，按 TVL 排序
3. 展示 vault 列表：symbol, name, base_token, apr, tvl_usd, deposit_tokens

### 场景 2：存入 0.00005 ETH 到 steakLRT vault

**用户说**: "Deposit 0.00005 ETH into steakLRT on Mellow Protocol"

**Agent 动作序列**:
1. [链下查询] 从 API 获取 steakLRT vault 地址 = `0xBEEF69Ac7870777598A04B2bd4771c71212E6aBc`
2. [链下查询] 检查 maxDeposit (`0x402d267d`) → 确认未达上限
3. [询问确认] "You are about to deposit 0.00005 ETH into steakLRT vault. **Please confirm to proceed.**"
4. [链上写操作] 调用 EthWrapper.deposit 附带 ETH value:
   ```
   onchainos wallet contract-call --chain 1 
     --to 0x83F6c979ce7a52C7027F08119222825E5bd50351
     --input-data "0x0bb9f5e1{eth_addr}{amount}{vault}{receiver}{referral}"
     --amt 50000000000000
   ```
5. 等待 tx 确认，报告 txHash

### 场景 3：查询用户持仓

**用户说**: "What are my Mellow LRT positions?"

**Agent 动作序列**:
1. [链下查询] `onchainos wallet balance --chain 1 --output json` → 获取钱包地址
2. [链下查询] 从 API 获取所有 vault 地址
3. [链下查询] 对每个 vault eth_call `balanceOf(wallet)` (`0x70a08231`)
4. 对有余额的 vault，计算 `convertToAssets(shares)` (`0x07a2d13a`)
5. 展示持仓：vault, shares, estimatedAssets, pendingAssets

### 场景 4：申请赎回 LRT shares

**用户说**: "Redeem my steakLRT shares from Mellow"

**Agent 动作序列**:
1. [链下查询] 获取用户在 steakLRT vault 的 shares 余额
2. [询问确认] "You have X shares (≈ Y wstETH). Withdraw all? **Please confirm to proceed.**"
3. [链上写操作] 调用 redeem(shares, receiver, owner):
   ```
   onchainos wallet contract-call --chain 1
     --to 0xBEEF69Ac7870777598A04B2bd4771c71212E6aBc
     --input-data "0xba087652{shares}{receiver}{owner}"
   ```
4. 通知用户: "Withdrawal initiated. Funds may take up to 14 days to process through Symbiotic withdrawal queue."

---

## §4 外部 API 依赖

| API | 用途 | 认证 |
|-----|------|------|
| `https://api.mellow.finance/v1/vaults` | 获取所有 vault 列表（名称、APR、TVL、deposit_tokens） | 无 |
| `https://ethereum.publicnode.com` | Ethereum mainnet RPC（eth_call 读取合约） | 无 |

---

## §5 配置参数

| 参数 | 默认值 | 描述 |
|------|--------|------|
| chain | 1 | Chain ID (目前只支持 Ethereum mainnet) |
| dry_run | false | 模拟模式，不广播交易 |

---

## §6 关键合约地址（Ethereum mainnet）

| 合约 | 地址 | 描述 |
|------|------|------|
| EthWrapper | `0x83F6c979ce7a52C7027F08119222825E5bd50351` | 将 ETH/WETH/stETH 转换为 wstETH 并存入 vault |
| wstETH | `0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0` | Wrapped staked ETH (Lido) |
| steakLRT | `0xBEEF69Ac7870777598A04B2bd4771c71212E6aBc` | Steakhouse Resteaking Vault |
| Re7LRT | `0x84631c0d0081FDe56DeB72F6DE77abBbF6A9f93a` | Re7 Labs LRT Vault |
| amphrETH | `0x5fD13359Ba15A84B76f7F87568309040176167cd` | Amphor ETH LRT Vault |
| rstETH | `0x7a4EffD87C2f3C55CA251080b1343b605f327E3a` | rstETH Vault |
| pzETH | `0x8c9532a60E0E7C6BbD2B2c1303F63aCE1c3E9811` | Renzo pzETH Vault |
| DVstETH | `0x5E362eb2c0706Bd1d134689eC75176018385430B` | Decentralized Validator Vault |

## §7 Selector 验证表

| 函数签名 | cast sig 结果 | 说明 |
|---------|-------------|------|
| `deposit(uint256,address)` | `0x6e553f65` | ERC-4626 deposit (for wstETH direct) |
| `redeem(uint256,address,address)` | `0xba087652` | ERC-4626 redeem (shares withdraw) |
| `deposit(address,uint256,address,address,address)` | `0x0bb9f5e1` | EthWrapper deposit (ETH/WETH/stETH) |
| `approve(address,uint256)` | `0x095ea7b3` | ERC-20 approve |
| `balanceOf(address)` | `0x70a08231` | ERC-20 balanceOf |
| `convertToAssets(uint256)` | `0x07a2d13a` | ERC-4626 convertToAssets |
| `convertToShares(uint256)` | `0xc6e6f592` | ERC-4626 convertToShares |
| `totalAssets()` | `0x01e1d114` | ERC-4626 totalAssets |
| `maxDeposit(address)` | `0x402d267d` | ERC-4626 maxDeposit |
| `claimableAssetsOf(address)` | `0xe7beaf9d` | MellowSymbioticVault claimableAssets |
| `pendingAssetsOf(address)` | `0x63c6b4eb` | MellowSymbioticVault pendingAssets |
| `claim(address,address,uint256)` | `0x996cba68` | MellowSymbioticVault claim |
| `asset()` | `0x38d52e0f` | ERC-4626 underlying asset |
| `decimals()` | `0x313ce567` | ERC-20 decimals |
| `symbol()` | `0x95d89b41` | ERC-20 symbol |

All selectors verified with `cast sig` (Foundry).

---

## §8 テスト用ウォレット状況

- Wallet: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
- ETH balance: ~0.00526 ETH (> 0.001 reserve threshold ✅)
- USDT: ~15 USDT
- wstETH: 0
- **L4 test plan**: Deposit 0.00005 ETH into steakLRT via EthWrapper (total cost ~0.00005 ETH + gas)
- ETH post-test reserve: ~0.004 ETH (safely above 0.001 minimum)

---

## §9 注意事项

1. **异步提款**: Mellow vault 的提款是 2-step 流程。redeem() 启动提款请求，但实际资金可能需要 2+ epochs (约 14 天) 才能通过 Symbiotic withdrawal queue 到账。用户需在此后调用 claim() 领取。

2. **ETH 存款路径**: 用户用 ETH 存款时，通过 EthWrapper 自动转换为 wstETH 后再存入 vault。这比直接存入 wstETH 更方便但需要额外的 wrapper contract 调用。

3. **wstETH 直存**: 如果用户已有 wstETH，直接 approve + deposit(uint256,address) 更高效。

4. **Vault 数量**: API 返回 71 个 Ethereum vault。在 `vaults` 命令中按 TVL 排序，默认显示前 20 个。

5. **ETH 地址约定**: EthWrapper 中 ETH 的 depositToken 地址是 `0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE`（标准 ETH 占位符），在合约源码中定义为 `address public constant ETH = 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE`。注意 Mellow API 的 deposit_tokens 中 ETH 地址是 `0x0000...0000`（零地址），但合约内部使用的是 EEE 地址。
