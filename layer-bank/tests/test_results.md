# 测试结果报告

- 日期: 2026-04-05
- DApp 支持的链: EVM (Scroll only — chain 534352)
- EVM 测试链: Scroll (534352)
- LayerBank NOT deployed on Base (8453) — confirmed via official GitHub README
- 编译: ✅
- Lint: ✅
- **整体通过标准**: EVM DApp → L1/L2/L3 全通过，L4 因 Scroll 无资金跳过

## 汇总

| 总数 | L1编译 | L2读取 | L3模拟 | L4链上 | 失败 | 阻塞 |
|------|--------|--------|--------|--------|------|------|
| 10   | 2      | 3      | 5      | 0(SKIP)| 0    | 0    |

## 详细结果

| # | 场景（用户视角） | Level | 命令 | 结果 | TxHash / Calldata | 备注 |
|---|----------------|-------|------|------|-------------------|------|
| 1 | 编译 (debug) | L1 | `cargo build` | ✅ PASS | — | 0 errors, 6 unused fn warnings |
| 2 | Lint 检查 | L1 | `cargo clean && plugin-store lint .` | ✅ PASS | — | 0 errors, 0 warnings |
| 3 | 查询所有 LayerBank 市场 | L2 | `markets` | ✅ PASS | — | ETH TVL $198K, USDC util 87% |
| 4 | 查看钱包持仓（空仓） | L2 | `positions --wallet 0x87fb...` | ✅ PASS | — | empty, health=∞ |
| 5 | 验证 ETH 市场数据 | L2 | `markets` → ETH entry | ✅ PASS | — | price=$2043, borrow=46 ETH |
| 6 | 模拟存入 0.001 ETH | L3 | `supply --asset ETH --amount 0.001 --dry-run` | ✅ PASS | calldata: `0xf2b9fdb8` | selector正确 |
| 7 | 模拟存入 0.01 USDC | L3 | `supply --asset USDC --amount 0.01 --dry-run` | ✅ PASS | step1: `0x095ea7b3`, step2: `0xf2b9fdb8` | 2步流程正确 |
| 8 | 模拟提取 0.01 USDC | L3 | `withdraw --asset USDC --amount 0.01 --dry-run` | ✅ PASS | calldata: `0x96294178` | selector正确 |
| 9 | 模拟借出 0.01 USDC | L3 | `borrow --asset USDC --amount 0.01 --dry-run` | ✅ PASS | calldata: `0x4b8a3529` | selector正确 |
| 10 | 模拟还款 0.01 USDC | L3 | `repay --asset USDC --amount 0.01 --dry-run` | ✅ PASS | step1: `0x095ea7b3`, step2: `0xabdb5ea8` | selector正确 |
| 11 | 链上存入 ETH | L4 | `supply --asset ETH --amount 0.0001` | ⏭️ SKIP | — | Scroll 无资金 |

## L4 跳过原因

LayerBank 未部署在 Base (chain 8453)，仅在 Scroll (chain 534352) 上有部署。
测试钱包在 Scroll 上余额为 0 ETH / 0 USDC。
Base 上的资金 (ETH=0.0028, USDC=0.27) 无法用于 Scroll 测试。

**解决方案**: 向 Scroll 充值 ≥ 0.001 ETH 后重跑 L4。

## Selector 验证

| 函数 | 预期 selector | 实际 calldata 前 4 字节 | 状态 |
|------|-------------|----------------------|------|
| supply(address,uint256) | 0xf2b9fdb8 | 0xf2b9fdb8 | ✅ |
| redeemUnderlying(address,uint256) | 0x96294178 | 0x96294178 | ✅ |
| borrow(address,uint256) | 0x4b8a3529 | 0x4b8a3529 | ✅ |
| repayBorrow(address,uint256) | 0xabdb5ea8 | 0xabdb5ea8 | ✅ |
| approve(address,uint256) | 0x095ea7b3 | 0x095ea7b3 | ✅ |

## 修复记录

无需修复 — 所有测试一次通过。
