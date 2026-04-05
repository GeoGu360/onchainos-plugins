---
name: etherfi-liquid
description: "ether.fi Liquid multi-strategy yield vaults — deposit weETH/WETH/USDC/WBTC into automated DeFi yield strategies. Commands: vaults, positions, deposit, withdraw, rates. Trigger phrases: ether.fi liquid, etherfi liquid vault, liquid eth vault, deposit weETH ether.fi, withdraw liquid, LIQUIDETH, LIQUIDUSD, LIQUIDBTC. Chinese: 以太坊ether.fi流动性金库,存款weETH,提取流动性"
license: MIT
metadata:
  author: GeoGu360
  version: "0.1.0"
---

## Overview

ether.fi Liquid provides multi-strategy ERC-4626-compatible yield vaults built on Veda's BoringVault architecture. Users deposit ETH-correlated (weETH, WETH, eETH) or stablecoin (USDC, USDT, DAI) tokens and receive vault shares that auto-compound across Pendle, Aave, Morpho, Balancer, Uniswap V3, Convex, and other DeFi protocols.

**Supported Vaults (Ethereum mainnet):**
- **LIQUIDETH** — ETH Yield Vault (weETH/WETH/eETH deposits, ~3% APY)
- **LIQUIDUSD** — USD Yield Vault (USDC/USDT/DAI deposits, ~4% APY)
- **LIQUIDBTC** — BTC Yield Vault (WBTC/eBTC/LBTC/cbBTC deposits, ~2% APY)

## Architecture

- Read ops (vaults, positions, rates) → direct `eth_call` via public RPC; no confirmation needed
- Write ops (deposit, withdraw) → **after user confirmation**, submits via `onchainos wallet contract-call`
- All on-chain ops use `--force` flag for broadcasting

## Commands

### vaults — List available vaults

Fetches APY from DefiLlama and share prices from on-chain Accountant contracts.

```
etherfi-liquid vaults [--chain 1] [--rpc-url <url>]
```

**Trigger phrases:** "show ether.fi liquid vaults", "list etherfi liquid vaults", "ether.fi liquid APY", "etherfi liquid rates overview"

**Output:** JSON with vault name, symbol, accepted tokens, APY, TVL, share price.

---

### positions — Show your vault positions

Reads LIQUIDETH/LIQUIDUSD/LIQUIDBTC share balances and calculates value.

```
etherfi-liquid positions [--chain 1] [--wallet <addr>] [--rpc-url <url>]
```

**Trigger phrases:** "show my ether.fi liquid positions", "my etherfi liquid balance", "how much LIQUIDETH do I have", "etherfi liquid portfolio"

**Output:** JSON with shares held and estimated value in deposit token per vault.

---

### rates — Show current exchange rates

Shows current share price for each vault from Accountant contract.

```
etherfi-liquid rates [--chain 1] [--rpc-url <url>]
```

**Trigger phrases:** "etherfi liquid share price", "LIQUIDETH rate", "ether.fi liquid exchange rate"

**Output:** JSON with share price (deposit token per vault share) and 7-day APY.

---

### deposit — Deposit tokens into a vault

**IMPORTANT: Ask user to confirm the transaction details before executing on-chain.**

Two-step flow (if approval needed):
1. ERC-20 approve deposit token to Teller — **ask user to confirm**
2. Teller.deposit(...) — **ask user to confirm** before proceeding

Run with `--dry-run` first to preview calldata without broadcasting.

```
etherfi-liquid deposit \
  --vault LIQUIDETH \
  --token weETH \
  --amount 0.001 \
  [--chain 1] \
  [--dry-run]
```

**Parameters:**
- `--vault`: LIQUIDETH (default), LIQUIDUSD, or LIQUIDBTC
- `--token`: deposit token symbol (weETH default for ETH vault, USDC for USD vault, WBTC for BTC vault)
- `--amount`: amount in human-readable units (e.g. 0.001 for 0.001 weETH)
- `--dry-run`: preview calldata without broadcasting

**Trigger phrases:** "deposit 0.1 weETH into ether.fi liquid", "add weETH to etherfi liquid vault", "deposit into LIQUIDETH", "stake weETH ether.fi liquid"

**Execution flow:**
1. Checks allowance; approves Teller if needed
2. **Ask user to confirm** before approve tx
3. Calls `onchainos wallet contract-call --chain 1 --to <teller> --input-data <approve_calldata> --force`
4. **Ask user to confirm** before deposit tx
5. Calls `onchainos wallet contract-call --chain 1 --to <teller> --input-data <deposit_calldata> --force`
6. Returns txHash and shares received

**Output:** JSON with approve_tx_hash (if needed), deposit_tx_hash, vault, amount.

---

### withdraw — Withdraw from a vault

**IMPORTANT: Ask user to confirm the transaction details before executing on-chain.**

Run with `--dry-run` first to preview calldata.

```
etherfi-liquid withdraw \
  --vault LIQUIDETH \
  --shares 0.001 \
  [--all] \
  [--chain 1] \
  [--dry-run]
```

**Parameters:**
- `--vault`: LIQUIDETH (default), LIQUIDUSD, or LIQUIDBTC
- `--shares`: number of vault shares to redeem in human-readable units
- `--all`: withdraw entire share balance
- `--dry-run`: preview calldata without broadcasting

**Trigger phrases:** "withdraw from ether.fi liquid", "redeem LIQUIDETH", "exit ether.fi liquid vault", "withdraw my weETH from etherfi liquid"

**Execution flow:**
1. Reads share balance from vault contract
2. Calculates expected weETH output using current rate
3. **Ask user to confirm** before executing
4. Calls `onchainos wallet contract-call --chain 1 --to <teller> --input-data <bulkWithdraw_calldata> --force`
5. Returns txHash and estimated assets received

**Output:** JSON with tx_hash, shares_withdrawn, expected_out_human, vault.

---

## Important Notes

- **Network:** Ethereum mainnet only (chain ID 1)
- **We already have weETH** from ether.fi Stake testing — use that for ETH vault deposits
- **Withdrawal is instant** via Teller.bulkWithdraw for supported assets (weETH supports withdrawals)
- **No fees** on deposit/withdraw (platform fees are within vault yield calculation)
- **Share price appreciates** over time as DeFi strategies generate yield
- **BoringVault architecture:** NOT standard ERC-4626; Teller is the entry point, not the vault itself
- **⚠️ Vault deposits require authorization:** The Teller uses `requiresAuth` (Veda RolesAuthority). Direct EOA contract calls from arbitrary wallets will revert. Deposits go through ether.fi's app infrastructure (ERC-4337 smart accounts). The plugin correctly builds calldata but on-chain execution requires an authorized caller. Ask user to confirm before proceeding.
