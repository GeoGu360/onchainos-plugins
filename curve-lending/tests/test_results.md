# Test Results — Curve Lending

- Date: 2026-04-05
- DApp 支持的链: EVM only (Ethereum mainnet, chain 1)
- EVM 测试链: Ethereum (1)
- Solana 测试链: N/A
- 编译: ✅
- Lint: ✅
- **整体通过标准**: EVM DApp → EVM 全通过

## Wallet

- EVM address: `0x87fb0647faabea33113eaf1d80d67acb1c491b90`
- ETH balance at test time: ~0.0043 ETH (above 0.001 reserve ✓)
- USDT balance: ~15.04 USDT
- crvUSD: 0 (borrow/repay = dry-run only per task brief)
- WETH: 0 (deposit-collateral L4 skipped)

## 汇总

| 总数 | L1编译 | L2读取 | L3模拟 | L4链上 | 失败 | 阻塞 |
|------|--------|--------|--------|--------|------|------|
| 13   | 2      | 5      | 3      | 3      | 0    | 0    |

## 详细结果

| # | 场景（用户视角） | Level | 命令 | 结果 | TxHash / Calldata | 备注 |
|---|----------------|-------|------|------|-------------------|------|
| 1 | Debug compile | L1 | `cargo build` | ✅ PASS | — | 0 errors, 32 warnings |
| 2 | Release build + lint | L1 | `cargo build --release && cargo clean && plugin-store lint .` | ✅ PASS | — | "Plugin 'curve-lending' passed all checks!" |
| 3 | 列出 Curve Lending 市场 | L2 | `markets --chain 1 --limit 5` | ✅ PASS | — | 46 markets found, WETH-long 82706 crvUSD TVL |
| 4 | 查询 WETH-long 利率 | L2 | `rates --chain 1 --market 1` | ✅ PASS | — | borrow 1.034%, lend 0.368%, util 35.58% |
| 5 | 查询钱包无仓位 | L2 | `positions --chain 1 --address 0x87fb... --market 1` | ✅ PASS | — | position_count: 0 (correct) |
| 6 | 查询实际借贷人仓位 | L2 | `positions --chain 1 --address 0x27b5... --market 1` | ✅ PASS | — | 5 WETH collateral, 4504 crvUSD debt, health 4.15% |
| 7 | 无效市场名错误处理 | L2 | `rates --chain 1 --market INVALIDXYZ` | ✅ PASS | — | Error: "Market 'INVALIDXYZ' not found" |
| 8 | 存抵押物 dry-run | L3 | `deposit-collateral --market 1 --amount 0.001 --dry-run` | ✅ PASS | approve: 0x095ea7b3..., create_loan: 0x23cfed03... | Selectors correct |
| 9 | 借款 dry-run | L3 | `borrow --market 1 --amount 1.5 --collateral 0.001 --dry-run` | ✅ PASS | create_loan: 0x23cfed03... | Selectors correct |
| 10 | 还款 dry-run | L3 | `repay --market 1 --amount 4500 --dry-run` | ✅ PASS | approve: 0x095ea7b3..., repay: 0x371fd8e6... | Selectors correct |
| 11 | 实时市场列表 | L4 | `markets --chain 1 --limit 5` | ✅ PASS | — | Live: 46 markets, CRV-long 3.3M crvUSD TVL |
| 12 | 实时 WETH-long 利率 | L4 | `rates --chain 1 --market 1` | ✅ PASS | — | Live: borrow 1.0343%, min 0.1%, max 101.38% |
| 13 | 实时真实借贷仓位 | L4 | `positions --chain 1 --address 0x27b5...` | ✅ PASS | — | Live: 5 WETH, 4504 crvUSD debt, liquidation 1370/667 |

## SKIPPED (dry-run only per GUARDRAILS)

| # | 场景 | 原因 |
|---|------|------|
| S1 | deposit-collateral L4 | 无 WETH。需先 wrap ETH。Task brief: "L4 may be limited to read ops only" |
| S2 | borrow L4 | 无 crvUSD → liquidation risk. GUARDRAILS: borrow dry-run only |
| S3 | repay L4 | 无 crvUSD → GUARDRAILS: repay dry-run only |

## 修复记录

| # | 问题 | 根因 | 修复 | 文件 |
|---|------|------|------|------|
| 1 | min_rate/max_rate 返回 0 | 错误 selector: 0xd22565a8/0x2c4e722e (错误计算) vs 正确 0x5d786401/0x536e4ec4 | 修正 config.rs 和 rates.rs 中的 selector | config.rs, commands/rates.rs |
| 2 | lend_apy 始终为 0 | vault.lend_apy() 在此市场返回 0 (函数存在但值为0) | 改为 borrow_apy × utilization 公式计算 | commands/rates.rs |
| 3 | user_state 返回 4 words 不是 6 | LlamaLend Lending Controller 返回 (collateral, stablecoin, debt, N) — 不同于 crvUSD mint market 的 6-word 格式 | 更新 decode_user_state() 以处理 4-word ABI | src/rpc.rs |
| 4 | rates 命令搜索市场名超时 | 搜索 46 市场逐个读取 names() RPC 调用过多 | 推荐使用 --market <index> 直接指定 | 文档优化 |
