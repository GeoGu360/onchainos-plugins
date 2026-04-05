# Jito Liquid Staking — Plugin Design

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| `plugin_name` | `jito` |
| `dapp_name` | Jito |
| `target_chains` | Solana (501) |
| `category` | defi-protocol |
| `description` | Jito MEV-enhanced liquid staking — stake SOL to receive JitoSOL, earn MEV rewards |
| `version` | 0.1.0 |

---

## §1 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 无专用 Rust SDK，使用 Solana JSON-RPC 直接交互 |
| SDK 支持哪些技术栈？ | TypeScript (`@solana/spl-stake-pool`), JavaScript |
| 有 REST API？ | 无托管 REST API；所有操作通过 Solana JSON-RPC 直接调用 SPL Stake Pool 程序 |
| 有官方 Skill？ | 无 |
| 开源社区有类似 Skill？ | 无（Bitable 记录确认） |
| 支持哪些链？ | Solana (mainnet-beta) 仅 |
| 是否需要 onchainos 广播？ | 是 — 链上写操作通过 `onchainos wallet contract-call --unsigned-tx` |

**接入路径：** API（Solana JSON-RPC + 手动构造 SPL Stake Pool 指令）

**技术架构：**
- Jito 使用 Solana SPL Stake Pool 程序（`SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy`）
- 没有托管 REST API —— 需要通过 Solana RPC 查询链上状态并手动构造交易
- 读取操作：解析 stake pool account data（611 bytes）
- 写操作：构造 DepositSol / WithdrawStake 指令，序列化为 base58 后通过 onchainos 广播

---

## §2 接口映射

### 需要接入的操作

| 操作 | 描述 | 类型 |
|------|------|------|
| `rates` | 查询当前 SOL↔JitoSOL 兑换率和 APY | 链下（RPC 读取） |
| `positions` | 查询用户 JitoSOL 余额和等值 SOL | 链下（RPC 读取） |
| `stake` | 存入 SOL，获得 JitoSOL | 链上写操作 |
| `unstake` | 赎回 JitoSOL，获得质押账户（解锁后可领取 SOL） | 链上写操作 |

---

### 链下查询

#### `rates` — 查询兑换率

通过 Solana RPC 读取 stake pool account，解析关键字段：

```
RPC: getAccountInfo
Address: Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb  (stake pool account)
Encoding: base64

SPL Stake Pool Account Layout (611 bytes):
  Offset 0:    account_type (u8) = 1 (StakePool)
  Offset 258:  total_lamports (u64 LE) — total SOL in pool
  Offset 266:  pool_token_supply (u64 LE) — total JitoSOL minted

Rate calculation:
  sol_per_jitosol = total_lamports / pool_token_supply
  jitosol_per_sol = pool_token_supply / total_lamports

APY estimation:
  - Read epoch schedule: getEpochSchedule
  - Approximate APY via rate changes (or use static well-known ~8% MEV boost)
  - Reported as approximate: ~8-10% APY (MEV-enhanced)
```

**Response fields:**
- `sol_per_jitosol`: f64 — how much SOL 1 JitoSOL is worth
- `jitosol_per_sol`: f64 — how much JitoSOL 1 SOL gets
- `total_staked_sol`: f64 — TVL in SOL
- `total_jitosol`: f64 — total JitoSOL supply
- `estimated_apy_pct`: f64 — approximate APY %

#### `positions` — 查询用户持仓

```
1. resolve_wallet_solana() → user_pubkey
2. Derive ATA address for JitoSOL mint:
   ATA = associated_token_address(user_pubkey, J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn)
3. getTokenAccountBalance(ata_address)
4. Also call rates() to convert to SOL equivalent
```

**Response fields:**
- `jitosol_balance`: f64 — JitoSOL balance
- `sol_value`: f64 — equivalent SOL value at current rate
- `wallet`: String — user's Solana address

---

### 链上写操作

#### `stake` — DepositSol instruction

**SPL Stake Pool DepositSol instruction (index 14):**

```
Instruction data: [14u8, lamports_u64_le (8 bytes)] = 9 bytes total

Account keys (order matters):
  0. stake_pool          : Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb   writable
  1. withdraw_authority  : PDA([pool_addr, "withdraw"], STAKE_POOL_PROGRAM_ID)  readonly
  2. reserve_stake       : (from pool state offset 130)                  writable
  3. from_user_lamports  : user wallet                                   writable, signer
  4. user_pool_token_ata : ATA(user, pool_mint)                          writable
  5. manager_fee_account : (from pool state offset 194)                  writable
  6. referrer_fee_acct   : same as user_pool_token_ata                   writable
  7. pool_mint           : J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn writable
  8. system_program      : 11111111111111111111111111111111               readonly
  9. token_program       : TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA  readonly

Pre-instruction: CreateAssociatedTokenAccount(user_ata) if not exists
  program: ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJe1bx8
  keys: [user_wallet, user_ata, user_wallet, pool_mint, system_program, token_program]
```

**Transaction construction (Rust):**
1. Fetch stake pool account → parse reserve_stake, pool_mint, manager_fee_account
2. Derive withdraw_authority PDA
3. Derive user ATA for pool_mint
4. Build CreateATA instruction (idempotent)
5. Build DepositSol instruction
6. Get recent blockhash
7. Serialize to base64 → convert to base58 → `onchainos wallet contract-call --chain 501 --to SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy --unsigned-tx <base58> --force`

**Amount format:** UI units (e.g., 0.001 SOL), converted to lamports internally (× 10^9)

---

#### `unstake` — WithdrawStake instruction

Unstaking from Jito creates a **stake account** that unlocks after the current epoch (~2-3 days). There is no instant withdraw (WithdrawSol is blocked). This is a delayed unstake.

**SPL Stake Pool WithdrawStake instruction (index 10):**

```
Instruction data: [10u8, pool_tokens_u64_le (8 bytes)] = 9 bytes total

Account keys:
  0. stake_pool           : Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb   writable
  1. validator_list       : (from pool state offset 98)                    writable
  2. withdraw_authority   : PDA derived                                    readonly
  3. validator_stake_acct : a validator stake account (fetch from list)   writable
  4. stake_dest           : new stake account keypair (must sign)         writable, signer
  5. user_stake_authority : user wallet                                    readonly
  6. user_transfer_auth   : user wallet                                    signer
  7. user_pool_token_ata  : user's JitoSOL ATA                            writable
  8. manager_fee_account  : from pool state                                writable
  9. pool_mint            : JitoSOL mint                                   writable
  10. sysvar_clock        : SysvarC1ock11111111111111111111111111111111111 readonly
  11. stake_program       : Stake11111111111111111111111111111111111111111 readonly
  12. token_program       : TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA  readonly

Pre-instruction: token Approve(user_ata, withdraw_authority, pool_tokens_amount)
```

**Implementation note:** WithdrawStake requires choosing a validator stake account to withdraw from. This is complex. For the initial implementation, we'll focus on `stake` (L4 tested) and provide `unstake` as L3 dry-run only.

---

## §3 用户场景

### 场景 1: 查询 JitoSOL 当前利率

**用户:** "查一下 Jito 当前的质押年化收益率"  
**Agent 动作序列:**
1. 调用 `./jito rates --chain 501`
2. Solana RPC: `getAccountInfo(Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb)`
3. 解析 total_lamports 和 pool_token_supply
4. 输出 SOL↔JitoSOL 兑换率和约 APY

### 场景 2: 查询持仓

**用户:** "查看我现在持有多少 JitoSOL"  
**Agent 动作序列:**
1. 调用 `./jito positions --chain 501`
2. `onchainos wallet balance --chain 501` → 解析 Solana 地址
3. 推导 ATA 地址：ATA(user_wallet, JitoSOL_mint)
4. Solana RPC: `getTokenAccountBalance(ata)`
5. 调用 rates() 计算等值 SOL
6. 输出 JitoSOL 余额和 SOL 等值

### 场景 3: 质押 SOL 获取 JitoSOL

**用户:** "帮我质押 0.001 SOL 到 Jito"  
**Agent 动作序列:**
1. 调用 `./jito stake --amount 0.001 --chain 501 --dry-run`（先预览）
2. 向用户展示：将质押 0.001 SOL，预计获得 ~0.000787 JitoSOL
3. 用户确认后，调用 `./jito stake --amount 0.001 --chain 501`
4. 内部：构造 DepositSol 交易 → base64 序列化 → base58 编码
5. `onchainos wallet contract-call --chain 501 --to SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy --unsigned-tx <base58_tx> --force`
6. 输出 txHash，附 solscan.io 链接

### 场景 4: 赎回 JitoSOL（延迟解锁）

**用户:** "我想赎回 0.005 JitoSOL"  
**Agent 动作序列:**
1. 调用 `./jito unstake --amount 0.005 --chain 501 --dry-run`（先预览）
2. 向用户说明：赎回创建质押账户，需等待当前 epoch 结束（约 2-3 天）后才能取回 SOL
3. 用户确认后，执行 WithdrawStake 交易
4. 输出 txHash 和预计解锁时间

---

## §4 外部 API 依赖

| API | 用途 | 认证 |
|-----|------|------|
| `https://api.mainnet-beta.solana.com` | Solana JSON-RPC (getAccountInfo, getTokenAccountBalance, getLatestBlockhash) | 无 |

---

## §5 配置参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `--chain` | u64 | 501 | Solana mainnet |
| `--amount` | f64 | — | SOL 数量（stake）或 JitoSOL 数量（unstake） |
| `--dry-run` | bool | false | 模拟操作，不广播 |

---

## §6 关键地址（Mainnet）

| 名称 | 地址 |
|------|------|
| SPL Stake Pool Program | `SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy` |
| Jito Stake Pool Account | `Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb` |
| JitoSOL Mint | `J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn` |
| Associated Token Program | `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJe1bx8` |
| Token Program | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` |
| System Program | `11111111111111111111111111111111` |

---

## §7 Solana 交易构造注意事项

1. **base64 → base58 转换**：Rust 序列化为 base64（内部），需要转换为 base58 再传给 onchainos
2. **ATA 创建**：质押前需确保用户有 JitoSOL ATA，若没有则在同一交易中创建
3. **Withdraw Authority PDA**：`find_program_address([pool_addr_bytes, b"withdraw"], STAKE_POOL_PROGRAM_ID)`
4. **Blockhash 过期**：构造交易后立即广播，不能缓存（60 秒过期）
5. **金额格式**：用户输入 UI 单位（0.001 SOL），内部乘以 10^9 转为 lamports
6. **Fee**：deposit fee 约 0%，epoch fee 约 5% of staking rewards，withdrawal fee 约 0.3%
