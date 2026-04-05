---
name: layer-bank
display_name: LayerBank
version: "0.1.0"
description: "LayerBank omni-chain lending protocol: supply assets to earn interest, borrow against collateral, view positions and health factor. Deployed on Scroll."
author: GeoGu360
category: defi-protocol
tags:
  - lending
  - borrowing
  - defi
  - layerbank
  - scroll
  - ltoken
---

# LayerBank Lending Protocol

LayerBank is an omni-chain over-collateralized lending protocol. Users supply assets to earn yield (receiving lTokens) and can borrow against their collateral. This plugin targets the **Scroll** deployment (chain 534352).

## Supported Commands

### `markets` — View all lending markets

Lists all LayerBank lToken markets with TVL, utilization rate, and asset prices.

**Trigger phrases:**
- "show LayerBank markets"
- "what assets can I lend on LayerBank?"
- "list LayerBank pools"
- "LayerBank supply rates"

**Example:**
```bash
layer-bank markets --chain 534352
```

**Output:** JSON array of markets with symbol, TVL, borrow/supply balances, utilization %, price in USD.

---

### `positions` — View your supplied and borrowed positions

Shows lToken balances, underlying supply/borrow amounts, and overall health factor for a wallet.

**Trigger phrases:**
- "show my LayerBank positions"
- "what have I supplied to LayerBank?"
- "my LayerBank borrow balance"
- "LayerBank health factor"

**Example:**
```bash
layer-bank positions --chain 534352
layer-bank positions --chain 534352 --wallet 0xYourAddress
```

**Output:** JSON with summary (total collateral, supply, borrow in USD, health factor) and per-asset positions.

---

### `supply` — Supply an asset to earn interest

Supplies an ERC-20 or ETH to LayerBank. Mints lTokens representing your deposit. Requires user confirmation before on-chain execution.

**Trigger phrases:**
- "supply 0.01 ETH to LayerBank"
- "deposit USDC into LayerBank"
- "lend wstETH on LayerBank Scroll"
- "add collateral to LayerBank"

**Parameters:**
- `--asset`: ETH, USDC, USDT, wstETH, WBTC
- `--amount`: human-readable amount (e.g. 0.01)
- `--from`: wallet address (optional, defaults to logged-in wallet)

**Example:**
```bash
# Dry-run (no on-chain tx)
layer-bank supply --asset USDC --amount 0.01 --chain 534352 --dry-run

# Live supply — ask user to confirm before running
layer-bank supply --asset ETH --amount 0.001 --chain 534352
```

**Flow:**
- ETH: `Core.supply(lETH, 0)` with `msg.value`
- ERC-20: `Token.approve(Core, amount)` → wait 3s → `Core.supply(lToken, amount)`

**Important:** Ask user to confirm before executing on-chain supply transactions.

---

### `withdraw` — Withdraw a supplied asset

Redeems underlying tokens by burning lTokens via `Core.redeemUnderlying`. Requires user confirmation.

**Trigger phrases:**
- "withdraw 0.01 USDC from LayerBank"
- "redeem my LayerBank position"
- "remove collateral from LayerBank"

**Parameters:**
- `--asset`: ETH, USDC, USDT, wstETH, WBTC
- `--amount`: underlying amount to withdraw
- `--from`: wallet address (optional)

**Example:**
```bash
layer-bank withdraw --asset USDC --amount 0.01 --chain 534352 --dry-run
layer-bank withdraw --asset ETH --amount 0.001 --chain 534352
```

**Important:** Ask user to confirm before broadcasting on-chain withdraw transactions. Ensure health factor stays above 1.0 after withdrawal.

---

### `borrow` — Borrow against collateral

Borrows an asset using supplied collateral. Creates debt with liquidation risk. DRY-RUN strongly recommended.

**Trigger phrases:**
- "borrow USDC from LayerBank"
- "take out a loan on LayerBank"
- "borrow against my ETH on LayerBank"

**Parameters:**
- `--asset`: ETH, USDC, USDT, wstETH, WBTC
- `--amount`: amount to borrow
- `--from`: wallet address (optional)

**Example:**
```bash
# Always dry-run first to see calldata
layer-bank borrow --asset USDC --amount 1.0 --chain 534352 --dry-run

# Live borrow — ask user to confirm before running
layer-bank borrow --asset USDC --amount 1.0 --chain 534352
```

**Important:** Ask user to confirm before broadcasting on-chain borrow transactions. Borrowing creates liquidation risk — always verify health factor first.

---

### `repay` — Repay a borrow position

Repays outstanding debt. For ERC-20: approve Core first, then call `Core.repayBorrow`. Requires user confirmation.

**Trigger phrases:**
- "repay my LayerBank loan"
- "pay back USDC on LayerBank"
- "close my LayerBank borrow"

**Parameters:**
- `--asset`: ETH, USDC, USDT, wstETH, WBTC
- `--amount`: amount to repay
- `--from`: wallet address (optional)

**Example:**
```bash
layer-bank repay --asset USDC --amount 1.0 --chain 534352 --dry-run
layer-bank repay --asset USDC --amount 1.0 --chain 534352
```

**Important:** Ask user to confirm before broadcasting on-chain repay transactions.

---

## Chain Support

| Chain | Chain ID | Status |
|-------|----------|--------|
| Scroll | 534352 | ✅ Supported |

LayerBank is NOT deployed on Base (chain 8453).

## Key Contracts (Scroll)

| Contract | Address |
|----------|---------|
| Core | `0xEC53c830f4444a8A56455c6836b5D2aA794289Aa` |
| PriceCalculator | `0xe3168c8D1Bcf6aaF5E090F61be619c060F3aD508` |
| lETH | `0x274C3795dadfEbf562932992bF241ae087e0a98C` |
| lUSDC | `0x0D8F8e271DD3f2fC58e5716d3Ff7041dBe3F0688` |
| lUSDT | `0xE0Cee49cC3C9d047C0B175943ab6FCC3c4F40fB0` |
| lwstETH | `0xB6966083c7b68175B4BF77511608AEe9A80d2Ca4` |
| lWBTC | `0xc40D6957B8110eC55f0F1A20d7D3430e1d8Aa4cf` |
