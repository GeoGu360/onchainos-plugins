# Skill Audit Report — Sky Lending (MakerDAO CDP)

**Source**: `/tmp/onchainos-plugins/sky-lending`
**测试时间**: 2026-04-06
**测试钱包 (EVM)**: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
**测试链**: Ethereum Mainnet (chain ID 1)
**Binary**: `sky-lending`

---

## 总结

| 项目 | 结果 |
|------|------|
| 编译 | ✅ (2 dead_code warnings，无错误) |
| 命令测试通过 | 7 / 7 |
| 链上写操作 | N/A（全部 dry-run；符合设计） |
| 发现 Bug | 3 项（已全部修复） |
| ABI Selector 验证 | ✅ 全部正确 |
| 安装 / 卸载 | ✅ |

---

## 命令测试结果

| # | 命令 | 状态 | Tx Hash | 备注 |
|---|------|------|---------|------|
| 1 | `sky-lending ilks` | ✅ | — | 返回 ETH-A/WBTC-A/USDC-A/WSTETH-A 参数；修复后稳定费率正确显示 |
| 2 | `sky-lending vaults` | ✅ | — | 测试地址无 CDP，提示友好 |
| 3 | `sky-lending open-vault --ilk ETH-A` | ✅ | — | dry-run，calldata 正确 |
| 4 | `sky-lending open-vault --ilk WSTETH-A` | ✅ | — | 修复后 ilk bytes32 正确（64 chars） |
| 5 | `sky-lending deposit-collateral --amount-eth 1.0` | ✅ | — | dry-run，wei 换算正确 |
| 6 | `sky-lending draw-dai --amount-dai 500.0` | ✅ | — | dry-run，两步 calldata 正确 |
| 7 | `sky-lending repay-dai --amount-dai 100.0` | ✅ | — | dry-run，三步 calldata 正确，int256 two's complement 正确 |
| 8 | `sky-lending withdraw-collateral --amount-eth 0.5` | ✅ | — | dry-run，负 dink 编码正确 |
| 9 | 错误处理：`--ilk INVALID` | ✅ | — | 返回友好错误信息 |
| 10 | 错误处理：`--amount-eth -1.0` | ✅ | — | clap 拒绝负值 |
| 11 | 错误处理：`--amount-dai 0` | ✅ | — | 返回 "must be greater than 0" |

---

## 发现的 Bug（已全部修复）

### Bug 1 — P0：Jug 合约地址缺少末位字符（稳定费率完全失效）

**文件**: `src/config.rs:12`, `skills/sky-lending/SKILL.md`

**现象**: `JUG` 地址 `0x19c0976f590D67707E62397C87829d896Dc0f1F` 为 39 位十六进制（应为 40 位）。RPC 节点以 `invalid argument: hex string of odd length` 拒绝所有 Jug 调用，导致 `ilks` 命令的稳定费率一行永远静默消失。

**根本原因**: 末位字符 `1` 被截断。正确地址为 `0x19c0976f590D67707E62397C87829d896Dc0f1F1`（已通过 `eth_getCode` 确认有合约代码，来源：MakerDAO changelog `1.9.12/contracts.json`）。

**修复**:
```
// before
pub const JUG: &str = "0x19c0976f590D67707E62397C87829d896Dc0f1F";
// after
pub const JUG: &str = "0x19c0976f590D67707E62397C87829d896Dc0f1F1";
```
同步更新 `SKILL.md` 中的合约地址表。

**影响**: `ilks` 命令稳定费率字段（Stability Fee）在修复前对所有 ilk 均静默不显示。

---

### Bug 2 — P1：Jug.ilks() 返回 2 个 uint256，代码却用 5-uint256 解码器（静默失败）

**文件**: `src/commands/ilks.rs:62`

**现象**: `Jug.ilks(bytes32)` 只返回 `(uint256 duty, uint256 rho)` = 128 字节。原代码调用 `decode_five_uint256_f64()` 要求至少 320 字节，必然报 "Return data too short"，被 `Err(_) => {}` 静默吞掉。

**修复**: 改用 `rpc::decode_two_uint256(&hex)` 正确解码两个 uint256。

```rust
// before
if let Ok((duty_f64, _rho, _, _, _)) = rpc::decode_five_uint256_f64(&hex) {
// after
if let Ok((duty_raw, _rho)) = rpc::decode_two_uint256(&hex) {
    let duty_normalized = duty_raw as f64 / 1e27;
```

---

### Bug 3 — P1：WSTETH-A ilk bytes32 编码多一个零字节（66 chars 而非 64）

**文件**: `src/config.rs:38`

**现象**: `ILK_WSTETH_A` 为 66 个十六进制字符（33 字节），ABI 编码应为严格 32 字节（64 chars）。传入 EVM 函数时会导致 ABI 解码错误，所有涉及 WSTETH-A 的链上调用均返回空数据或错误。

**修复**:
```
// before (66 chars)
pub const ILK_WSTETH_A: &str = "5753544554482d4100000000000000000000000000000000000000000000000000";
// after (64 chars, correct bytes32)
pub const ILK_WSTETH_A: &str = "5753544554482d41000000000000000000000000000000000000000000000000";
```

---

## ABI Selector 验证（cast sig）

所有 function selector 与源码硬编码值完全一致：

| 函数签名 | 期望 selector | 实际（cast sig） | 状态 |
|---------|--------------|----------------|------|
| `open(bytes32,address)` | `6090dec5` | `6090dec5` | ✅ |
| `urns(uint256)` | `2726b073` | `2726b073` | ✅ |
| `ilks(uint256)` | `2c2cb9fd` | `2c2cb9fd` | ✅ |
| `first(address)` | `fc73d771` | `fc73d771` | ✅ |
| `count(address)` | `05d85eda` | `05d85eda` | ✅ |
| `list(uint256)` | `80c9419e` | `80c9419e` | ✅ |
| `urns(bytes32,address)` | `2424be5c` | `2424be5c` | ✅ |
| `ilks(bytes32)` | `d9638d36` | `d9638d36` | ✅ |
| `frob(bytes32,address,address,address,int256,int256)` | `76088703` | `76088703` | ✅ |
| `join(address)` | `28ffe6c8` | `28ffe6c8` | ✅ |
| `exit(address,uint256)` | `ef693bed` | `ef693bed` | ✅ |
| `join(address,uint256)` | `3b4da69f` | `3b4da69f` | ✅ |
| `approve(address,uint256)` | `095ea7b3` | `095ea7b3` | ✅ |

---

## 代码静态审查

### 优点
- int256 两的补码实现正确（经 Python 交叉验证）
- 所有写操作强制 dry-run，符合 CDP 高风险场景的安全设计
- 错误信息用户友好，无裸 panic/unwrap
- RPC 层与 onchainos CLI 层分离清晰
- 链式链表遍历有 50 个 vault 上限，防止无限循环

### 改进建议（P2）

1. **dead_code 警告**：`decode_five_uint256`、`decode_jug_ilks` 两个函数从未被调用，可删除或加 `#[allow(dead_code)]`。

2. **稳定费近似精度**：当前用线性近似 `(duty/1e27 - 1) * 31_536_000`，更准确应用指数公式 `(duty/1e27)^31536000 - 1`。对小费率（< 5%）差异可接受，但较高费率时误差约 0.3-0.5 个百分点。

3. **SKILL.md 触发词**：缺少中文触发词（如"存入ETH"、"借DAI"、"CDP抵押"），以及"Do NOT use for..."规则区分 Sky 与 Spark、Aave 等其他借贷协议。

4. **vaults 命令**：对已迁移到 Sky/Endgame 系统的 CDP，可补充指向 `sky.money` 界面的链接提示。

---

## SKILL.md 质量评估

| 项目 | 评估 |
|------|------|
| description 字段 ASCII-only | ✅ |
| 命令均有参数表和示例 | ✅ |
| 覆盖中文触发词 | ❌（无中文触发词） |
| Do NOT use for 规则 | ❌（缺失） |
| 错误处理表 | ✅ |
| 风险提示 | ✅（详尽） |
| Skill Routing 章节 | ✅ |

---

## 总体评分

**⭐⭐⭐⭐** (4/5)

代码架构清晰，安全设计合理（全 dry-run），ABI 编码正确。修复前 3 个 bug（Jug 地址截断、Jug 解码器错误、WSTETH-A bytes32 超长）导致稳定费率显示和 WSTETH-A 操作完全失效。修复后所有 7 条命令均通过测试。
