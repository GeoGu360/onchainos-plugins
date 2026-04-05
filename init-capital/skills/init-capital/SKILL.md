---
name: init-capital
display_name: INIT Capital
version: "0.1.0"
description: "INIT Capital isolated lending protocol: supply assets, borrow against collateral, manage multi-silo positions with health factor monitoring. Deployed on Blast."
author: GeoGu360
category: defi-protocol
tags:
  - lending
  - borrowing
  - defi
  - init-capital
  - blast
  - isolated-positions
---

# INIT Capital Lending Protocol

INIT Capital is a non-custodial decentralized lending protocol with multi-silo isolated positions. Each position is independently risk-managed. Users supply assets to earn yield and borrow against collateral. This plugin targets the **Blast** deployment (chain 81457).

## Supported Commands

### `pools` - View lending pools

Lists all INIT Capital lending pools on Blast with supply/borrow rates and total assets.

**Trigger phrases:**
- "show INIT Capital pools"
- "INIT Capital lending rates on Blast"
- "what assets can I supply to INIT Capital?"
- "INIT Capital supply APY"

**Example:**
```bash
init-capital pools --chain 81457
```

**Output:** JSON array of pools with symbol, total supplied, supply APY%, borrow APY%.

---

### `positions` - View your positions

Shows all your INIT Capital positions with collateral, debt, and health factor.

**Trigger phrases:**
- "show my INIT Capital positions"
- "what have I supplied to INIT Capital on Blast?"
- "my INIT Capital borrow balance"
- "INIT Capital health factor"
- "my INIT Capital collateral"

**Example:**
```bash
init-capital positions --chain 81457
init-capital positions --chain 81457 --wallet 0xYourAddress
```

**Output:** JSON with all positions, collateral amounts, debt amounts, and health factors.

---

### `health-factor` - Check position health

Gets the health factor for a specific INIT Capital position.

**Trigger phrases:**
- "check health factor for INIT Capital position 1"
- "is my INIT Capital position healthy?"
- "INIT Capital position 2 health"

**Parameters:**
- `--pos-id`: Position ID to check

**Example:**
```bash
init-capital health-factor --pos-id 1 --chain 81457
```

**Output:** Health factor value (>1.0 = healthy, <1.0 = liquidatable).

---

### `supply` - Supply assets to earn interest

Supplies tokens to INIT Capital. Creates a new position (pos-id=0) or adds to an existing one.

**Trigger phrases:**
- "supply WETH to INIT Capital on Blast"
- "deposit USDB into INIT Capital"
- "add collateral to INIT Capital"
- "lend WETH on INIT Capital"

**Parameters:**
- `--asset`: WETH, USDB
- `--amount`: human-readable amount (e.g. 0.01)
- `--pos-id`: position ID (0 = create new position, default: 0)
- `--from`: wallet address (optional, defaults to logged-in wallet)

**Example:**
```bash
# Dry-run (no on-chain tx)
init-capital supply --asset WETH --amount 0.01 --chain 81457 --dry-run

# Live supply - ask user to confirm before running
init-capital supply --asset WETH --amount 0.01 --chain 81457
```

**Flow:**
1. `WETH.approve(MoneyMarketHook, amount)` (ERC-20 approve)
2. `MoneyMarketHook.execute(OperationParams{depositParams})` (creates/updates position)

**Important:** Ask user to confirm before executing on-chain supply transactions via `wallet contract-call`.

---

### `withdraw` - Withdraw collateral

Withdraws supplied assets from an INIT Capital position.

**Trigger phrases:**
- "withdraw WETH from INIT Capital position 1"
- "remove collateral from INIT Capital"
- "take out my WETH from INIT Capital"

**Parameters:**
- `--asset`: WETH, USDB
- `--amount`: amount to withdraw
- `--pos-id`: position ID
- `--to`: recipient address (optional, defaults to sender)
- `--from`: wallet address (optional)

**Example:**
```bash
init-capital withdraw --asset WETH --amount 0.01 --pos-id 1 --chain 81457 --dry-run
init-capital withdraw --asset WETH --amount 0.01 --pos-id 1 --chain 81457
```

**Important:** Ask user to confirm before broadcasting on-chain withdraw transaction via `wallet contract-call`. Ensure health factor remains above 1.0 after withdrawal.

---

### `borrow` - Borrow against collateral

Borrows assets from an INIT Capital position. DRY-RUN strongly recommended.

**Trigger phrases:**
- "borrow USDB from INIT Capital position 1"
- "take out a loan on INIT Capital Blast"
- "borrow against my WETH on INIT Capital"

**Parameters:**
- `--asset`: WETH, USDB
- `--amount`: amount to borrow
- `--pos-id`: position ID
- `--to`: recipient address (optional, defaults to sender)
- `--from`: wallet address (optional)

**Example:**
```bash
# Always dry-run first
init-capital borrow --asset USDB --amount 1.0 --pos-id 1 --chain 81457 --dry-run

# Live borrow - ask user to confirm before running
init-capital borrow --asset USDB --amount 1.0 --pos-id 1 --chain 81457
```

**Important:** Ask user to confirm before broadcasting on-chain borrow transaction via `wallet contract-call`. Borrowing creates liquidation risk - always verify health factor first.

---

### `repay` - Repay debt

Repays outstanding debt in an INIT Capital position. DRY-RUN recommended.

**Trigger phrases:**
- "repay my INIT Capital loan"
- "pay back USDB on INIT Capital"
- "close my INIT Capital borrow"
- "repay debt in INIT Capital position 1"

**Parameters:**
- `--asset`: WETH, USDB
- `--amount`: amount to repay
- `--pos-id`: position ID
- `--from`: wallet address (optional)

**Example:**
```bash
init-capital repay --asset USDB --amount 1.0 --pos-id 1 --chain 81457 --dry-run
init-capital repay --asset USDB --amount 1.0 --pos-id 1 --chain 81457
```

**Important:** Ask user to confirm before broadcasting on-chain repay transaction via `wallet contract-call`.

---

## Chain Support

| Chain | Chain ID | Status |
|-------|----------|--------|
| Blast | 81457 | Supported |

Note: INIT Capital is also deployed on Mantle (chain 5000), but onchainos does not support Mantle. Use Blast deployment.

## Key Contracts (Blast)

| Contract | Address |
|----------|---------|
| INIT_CORE | `0xa7d36f2106b5a5D528a7e2e7a3f436d703113A10` |
| POS_MANAGER | `0xA0e172f8BdC18854903959b8f7f73F0D332633fe` |
| MONEY_MARKET_HOOK | `0xC02819a157320Ba2859951A1dfc1a5E76c424dD4` |
| POOL_WWETH | `0xD20989EB39348994AA99F686bb4554090d0C09F3` |
| POOL_WUSDB | `0xc5EaC92633aF47c0023Afa0116500ab86FAB430F` |

## Isolated Position Model

INIT Capital uses NFT-based isolated positions:
- Each position is identified by a `posId`
- Positions are independent - risk is siloed per position
- Supply first to create a position (`--pos-id 0`)
- Use `positions` command to list your position IDs
