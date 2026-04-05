---
name: sanctum-infinity
description: "Sanctum Infinity is Solana's flagship infinite-LST pool. Swap between any LSTs at near-zero cost, deposit LSTs to earn staking yields + trading fees, and query pool stats. Trigger phrases: swap LST on Sanctum, swap jitoSOL to INF, swap mSOL to INF, deposit to Sanctum Infinity, add liquidity Sanctum, withdraw from Sanctum Infinity, Sanctum pool stats, INF pool, check INF balance. Chinese: 在Sanctum上交换LST, 存入Sanctum Infinity, 从Sanctum取回, 查询Sanctum池子"
license: MIT
metadata:
  author: skylavis-sky
  version: "0.1.0"
---

## Overview

Sanctum Infinity (INF) is a multi-LST liquidity pool on Solana. It holds a basket of LSTs (jitoSOL, mSOL, bSOL, and many others) and earns staking yields + trading fees. You can:

1. **Swap** any supported LST to any other LST at near-zero fees
2. **Deposit** LST to the pool and receive INF tokens representing your share
3. **Withdraw** your LST back by burning INF tokens
4. **Query** pool stats, APY, allocations, and your own INF holdings

## Architecture

- Read ops (pools, quote, positions) → direct REST API calls to Sanctum Extra API / Router API; no confirmation needed
- Write ops (swap, deposit, withdraw) → after user confirmation, fetch serialized transaction from Sanctum Router API, convert base64→base58, submit via `onchainos wallet contract-call --chain 501 --unsigned-tx <base58_tx> --force`

## Commands

### pools — Query Infinity Pool Stats

**Trigger**: "Show Sanctum Infinity pool stats", "What's in the INF pool?", "Sanctum pool allocations"

```bash
sanctum-infinity pools
```

**Output**: INF NAV (SOL per INF), APY, total TVL, top LST allocations by pool share.

---

### quote — Get Swap Quote

**Trigger**: "Quote swapping 0.005 jitoSOL to INF on Sanctum", "How much INF do I get for 0.005 jitoSOL?"

```bash
sanctum-infinity quote --from jitoSOL --to INF --amount 0.005
sanctum-infinity quote --from mSOL --to jitoSOL --amount 0.01 --slippage 1.0
```

**Parameters**:
- `--from`: Input LST symbol (jitoSOL, mSOL, INF, SOL) or mint address
- `--to`: Output LST symbol or mint address
- `--amount`: Amount in UI units (e.g. 0.005 jitoSOL)
- `--slippage`: Slippage tolerance % (default 0.5)

**Output**: inAmount, outAmount, minimum output (after slippage), fees breakdown.

---

### swap — Execute LST→LST Swap

**Trigger**: "Swap 0.001 jitoSOL to INF on Sanctum Infinity", "Convert my jitoSOL to INF"

```bash
sanctum-infinity swap --from jitoSOL --to INF --amount 0.001
sanctum-infinity --dry-run swap --from jitoSOL --to INF --amount 0.001
```

**Parameters**:
- `--from`: Input LST symbol or mint address
- `--to`: Output LST symbol or mint address
- `--amount`: Amount in UI units
- `--slippage`: Slippage tolerance % (default 0.5)
- `--dry-run`: Preview without broadcasting

**Execution Flow for Write Operations**:
1. Run with `--dry-run` first to preview the swap
2. **Ask user to confirm** before executing on-chain
3. Get swap quote, then fetch serialized transaction from Sanctum Router API
4. Convert base64→base58, submit via `onchainos wallet contract-call --chain 501 --unsigned-tx <tx> --force`
5. Return txHash

---

### deposit — Add Liquidity to Infinity Pool

**Trigger**: "Deposit 0.005 jitoSOL to Sanctum Infinity", "Add liquidity to INF pool", "Stake in Sanctum Infinity"

```bash
sanctum-infinity deposit --lst jitoSOL --amount 0.005
sanctum-infinity --dry-run deposit --lst jitoSOL --amount 0.005
```

**Parameters**:
- `--lst`: LST symbol or mint address to deposit
- `--amount`: Amount in UI units
- `--slippage`: Slippage tolerance % (default 0.5)
- `--dry-run`: Preview without broadcasting

**Execution Flow for Write Operations**:
1. Run with `--dry-run` first to see expected INF received
2. **Ask user to confirm** before executing on-chain
3. Get liquidity add quote, then fetch serialized transaction
4. Convert base64→base58, submit via `onchainos wallet contract-call --chain 501 --unsigned-tx <tx> --force`
5. Return txHash and INF received

---

### withdraw — Remove Liquidity from Infinity Pool

**Trigger**: "Withdraw 0.001 INF from Sanctum as jitoSOL", "Remove liquidity from Sanctum Infinity"

```bash
sanctum-infinity withdraw --lst jitoSOL --amount 0.001
sanctum-infinity --dry-run withdraw --lst jitoSOL --amount 0.001
```

**Parameters**:
- `--lst`: LST to receive (symbol or mint address)
- `--amount`: Amount of INF to burn (UI units)
- `--slippage`: Slippage tolerance % (default 0.5)
- `--dry-run`: Preview without broadcasting

**Execution Flow for Write Operations**:
1. Run with `--dry-run` first to see expected LST received
2. **Ask user to confirm** before executing on-chain
3. Get liquidity remove quote, then fetch serialized transaction
4. Convert base64→base58, submit via `onchainos wallet contract-call --chain 501 --unsigned-tx <tx> --force`
5. Return txHash and LST received

---

### positions — View Your INF Holdings

**Trigger**: "How much INF do I hold?", "Show my Sanctum Infinity balance", "Check my INF position"

```bash
sanctum-infinity positions
```

**Output**: INF balance, current NAV in SOL, total value in SOL.

---

## Key Addresses

| Item | Value |
|------|-------|
| INF Token Mint | `5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm` |
| INF Pool Program | `5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx` |

## Supported LST Symbols

- `INF` — Sanctum Infinity token
- `jitoSOL` — JitoSOL
- `mSOL` — Marinade staked SOL
- `SOL` / `wSOL` — native SOL
- Any LST mint address (base58)
