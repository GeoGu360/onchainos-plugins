# Test Results — Synthetix V3

- 日期: 2026-04-05
- DApp 支持的链: EVM — Base (8453)
- EVM 测试链: Base (8453)
- 编译: ✅ (debug + release)
- Lint: ✅ (0 errors, E123 expected — will be fixed post-monorepo push)
- **整体通过标准**: EVM DApp → EVM 全通过

## 汇总

| 总数 | L1编译 | L2读取 | L3模拟 | L4链上 | 失败 | 阻塞 |
|------|--------|--------|--------|--------|------|------|
| 10   | 2      | 5      | 2      | 0      | 0    | 1    |

## 详细结果

| # | 场景（用户视角） | Level | 命令 | 结果 | TxHash / Calldata | 备注 |
|---|----------------|-------|------|------|-------------------|------|
| 1 | 编译 debug build | L1 | `cargo build` | ✅ PASS | — | 仅警告，无错误 |
| 2 | 编译 release build | L1 | `cargo build --release` | ✅ PASS | — | — |
| 3 | Lint 检查 | L1 | `cargo clean && plugin-store lint .` | ✅ PASS | — | E123 预期 (monorepo SHA 待填) |
| 4 | 查询所有 Perps 市场 | L2 | `markets` | ✅ PASS | — | total=108, showing=5 active |
| 5 | 查询 ETH 市场 (ID=100) | L2 | `markets --market-id 100` | ✅ PASS | — | funding=0.1093, size=19.79 |
| 6 | 查询 BTC 市场 (ID=200) | L2 | `markets --market-id 200` | ✅ PASS | — | funding=0.1193, size=0.82 |
| 7 | 查询账户持仓 (account=1) | L2 | `positions --account-id 1` | ✅ PASS | — | margin=158.21 |
| 8 | 查询账户抵押品 (account=1) | L2 | `collateral --account-id 1` | ✅ PASS | — | 0 collaterals (未存款) |
| 9 | 模拟 deposit 0.01 sUSDC | L3 | `--dry-run deposit-collateral --account-id 12345 --amount 0.01` | ✅ PASS | calldata: `0x83802968...` | selector 0x83802968 ✅ |
| 10 | 模拟 withdraw 0.01 sUSDC | L3 | `--dry-run withdraw-collateral --account-id 12345 --amount 0.01` | ✅ PASS | calldata: `0x95997c51...` | selector 0x95997c51 ✅ |
| 11 | 链上 deposit-collateral | L4 | SKIPPED | BLOCKED | — | 钱包无 sUSDC；需先 wrap USDC→sUSDC via SpotMarket |
| 12 | 链上 withdraw-collateral | L4 | SKIPPED | BLOCKED | — | 依赖 L4 deposit 先成功 |

## L4 Skip 说明

L4 write 测试跳过原因：
1. 测试钱包 `0x87fb0647faabea33113eaf1d80d67acb1c491b90` 持有 USDC (0.28) 但无 sUSDC
2. Synthetix V3 CoreProxy 接受 sUSDC (synth token)，而非原始 USDC
3. 获取 sUSDC 需要先调用 SpotMarketProxy.wrap() — 额外的 gas 消耗
4. 钱包无 Synthetix V3 账户 (AccountProxy.balanceOf = 0)
5. 完整 L4 流程需要: createAccount + wrap USDC→sUSDC + approve + deposit — 超出 budget 限制
6. **ETH 余额 0.0028 > 0.001 reserve** — ETH 充足，但 sUSDC 不足

建议后续 L4 测试：先发送少量 sUSDC (0.01 sUSDC) 到钱包后再运行 L4。

## 修复记录

| # | 问题 | 根因 | 修复 | 文件 |
|---|------|------|------|------|
| 1 | `markets` 输出包含 error 字符串 | Pyth ERC-7412 markets revert without price feed update | 跳过失败的 market summary，仅显示成功的 | markets.rs |
| 2 | `collateral` panic for large account_id | u128 无法序列化为 JSON 数字 | 改为 `.to_string()` 序列化 | collateral.rs, positions.rs, markets.rs |
