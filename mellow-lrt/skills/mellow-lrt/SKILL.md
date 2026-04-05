---
name: mellow-lrt
description: "Mellow LRT — liquid restaking token vaults on Ethereum. Deposit ETH or wstETH to earn restaking yield through EigenLayer/Symbiotic. Trigger phrases: mellow lrt, mellow protocol, deposit ETH restaking, mellow vault, steakLRT, Re7LRT, liquid restaking token, mellow deposit, mellow withdraw, mellow positions. Chinese: 质押ETH到Mellow, 流动再质押, Mellow协议存款, Mellow LRT持仓"
license: MIT
metadata:
  author: GeoGu360
  version: "0.1.0"
---

## Overview

Mellow LRT enables users to deposit ETH, wstETH, WETH, or stETH into curated liquid restaking vaults. Each vault issues an LRT token (e.g. steakLRT, Re7LRT) backed by restaked assets earning yield from EigenLayer/Symbiotic.

**Supported chain**: Ethereum mainnet (chain 1)

## Architecture

- Read ops (vaults list, positions) → direct `eth_call` via public RPC + Mellow REST API; no confirmation needed
- Write ops (deposit, withdraw, claim) → after **user confirmation**, submits via `onchainos wallet contract-call`

## Execution Flow for Write Operations

1. Run with `--dry-run` first to preview calldata
2. **Ask user to confirm** the transaction before executing on-chain
3. Execute only after explicit user approval
4. Report transaction hash and outcome

---

## Commands

### `vaults` — List Mellow LRT Vaults

List all Mellow LRT vaults on Ethereum, sorted by TVL. Shows APR, TVL, accepted deposit tokens.

**Usage:**
```bash
mellow-lrt --chain 1 vaults
mellow-lrt --chain 1 vaults --limit 10
```

**Parameters:**
- `--chain`: Chain ID (default: 1)
- `--limit`: Max vaults to show (default: 20)

**Trigger phrases:** "show mellow vaults", "list mellow LRT options", "what are the best mellow restaking vaults", "mellow lrt APR"

---

### `positions` — Show User Positions

Show user's current LRT positions: shares held, estimated value, claimable and pending withdrawal amounts.

**Usage:**
```bash
mellow-lrt --chain 1 positions
mellow-lrt --chain 1 positions --wallet 0xABC...
```

**Parameters:**
- `--wallet`: Optional wallet address (resolved from onchainos if not provided)

**Trigger phrases:** "show my mellow positions", "what mellow LRT do I hold", "my mellow restaking balance", "check mellow vault positions"

---

### `deposit` — Deposit into a Mellow LRT Vault

Deposit ETH, wstETH, WETH, or stETH into a Mellow LRT vault and receive LRT shares.

**Usage:**
```bash
mellow-lrt --chain 1 deposit --vault steakLRT --token ETH --amount 0.00005
mellow-lrt --chain 1 deposit --vault Re7LRT --token wstETH --amount 0.001
mellow-lrt --chain 1 deposit --vault steakLRT --token ETH --amount 0.00005 --dry-run
```

**Parameters:**
- `--vault`: Vault symbol (steakLRT, Re7LRT, amphrETH, etc.) or vault address
- `--token`: Token to deposit — ETH, WETH, stETH, or wstETH (default: ETH)
- `--amount`: Amount to deposit (e.g. 0.00005)
- `--dry-run`: Preview calldata without broadcasting

**Trigger phrases:** "deposit ETH into mellow", "stake ETH in steakLRT", "put ETH into mellow restaking vault", "deposit wstETH mellow", "invest in mellow LRT"

**Note:** For ETH deposits, funds are automatically converted to wstETH via EthWrapper. For wstETH deposits, requires ERC-20 approve + ERC-4626 deposit. **Always ask user to confirm before submitting.**

---

### `withdraw` — Initiate Withdrawal

Start the 2-step async withdrawal process. Redeems LRT shares and queues them for processing (~14 days).

**Usage:**
```bash
mellow-lrt --chain 1 withdraw --vault steakLRT --all
mellow-lrt --chain 1 withdraw --vault steakLRT --amount 0.5
mellow-lrt --chain 1 withdraw --vault steakLRT --all --dry-run
```

**Parameters:**
- `--vault`: Vault symbol or address
- `--amount`: Shares to redeem (human-readable)
- `--all`: Redeem all shares
- `--dry-run`: Preview calldata without broadcasting

**Trigger phrases:** "withdraw from mellow", "redeem steakLRT shares", "exit mellow vault", "start mellow withdrawal", "unstake from mellow LRT"

**Note:** Withdrawal is async through Symbiotic queue. After initiating, wait ~14 days then call `claim`. **Always ask user to confirm before submitting.**

---

### `claim` — Claim Unlocked Withdrawals

Claim wstETH from completed withdrawal queue entries. Only works after the queue period (~14 days).

**Usage:**
```bash
mellow-lrt --chain 1 claim --vault steakLRT
mellow-lrt --chain 1 claim --vault steakLRT --dry-run
```

**Parameters:**
- `--vault`: Vault symbol or address
- `--dry-run`: Preview calldata without broadcasting

**Trigger phrases:** "claim mellow withdrawal", "collect mellow LRT assets", "finish mellow withdrawal", "claim wstETH from mellow"

**Note:** Checks `claimableAssetsOf` first. Reports "nothing to claim" if queue hasn't processed yet. **Always ask user to confirm before submitting.**

---

## Known Vaults (Ethereum Mainnet)

| Symbol | Name | Base Token | Vault Address |
|--------|------|------------|---------------|
| steakLRT | Steakhouse Resteaking Vault | wstETH | `0xBEEF69Ac7870777598A04B2bd4771c71212E6aBc` |
| Re7LRT | Re7 Labs LRT Vault | wstETH | `0x84631c0d0081FDe56DeB72F6DE77abBbF6A9f93a` |
| amphrETH | Amphor ETH LRT Vault | wstETH | `0x5fD13359Ba15A84B76f7F87568309040176167cd` |
| rstETH | rstETH Vault | wstETH | `0x7a4EffD87C2f3C55CA251080b1343b605f327E3a` |
| pzETH | Renzo pzETH Vault | wstETH | `0x8c9532a60E0E7C6BbD2B2c1303F63aCE1c3E9811` |

Run `mellow-lrt --chain 1 vaults` for the full list.

---

## Key Addresses

- **EthWrapper**: `0x83F6c979ce7a52C7027F08119222825E5bd50351` — converts ETH/WETH/stETH to wstETH and deposits
- **wstETH**: `0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0`

## Withdrawal Timeline

1. Call `withdraw --all` → redeems shares, starts queue
2. Wait ~14 days (Symbiotic epoch processing)
3. Call `claim` → receive wstETH

For urgent withdrawals, check if liquid buffer is available (claimable immediately if vault has liquid reserves).
