---
name: allbridge-classic
description: Bridge stablecoins cross-chain using Allbridge Classic. Supports USDT, USDC, BUSD transfers between Ethereum, BSC, Polygon, Avalanche, and Solana. Provides token info, transaction status, and address validation.
---

# Allbridge Classic Bridge Plugin

## Overview

This plugin enables cross-chain stablecoin bridging via Allbridge Classic. Users can transfer USDT, USDC, BUSD, and other stablecoins between EVM chains (Ethereum, BSC, Polygon, Avalanche) and Solana.

**Key facts:**
- Allbridge Classic uses a lock-and-unlock model: tokens are locked on source chain, unlocked on destination
- Bridge fee: typically 0.3% (Ethereum charges a flat $1 minimum fee)
- Bridge confirmation takes 5-15 minutes after source lock is confirmed
- All EVM write operations require user confirmation before submission
- Source chains supported: Ethereum (1), BSC (56), Polygon (137), Avalanche (43114)
- Note: Allbridge Classic is in maintenance mode; migrating to Allbridge Core is recommended for new integrations

## Architecture

- Read ops (token list, tx status, address check) -> Allbridge REST API (no wallet required)
- Write ops (bridge/lock) -> after user confirmation, submits via `onchainos wallet contract-call`

## Pre-flight Checks

Before running any write command:
1. Verify `onchainos` is installed: `onchainos --version`
2. Check wallet is logged in: `onchainos wallet balance --chain <chain_id>`
3. Confirm user has approved the transaction before proceeding

## Contract Addresses

| Chain | Chain ID | Bridge Contract |
|---|---|---|
| Ethereum | 1 | `0xBBbD1BbB4f9b936C3604906D7592A644071dE884` |
| BSC | 56 | `0xBBbD1BbB4f9b936C3604906D7592A644071dE884` |
| Polygon | 137 | `0xBBbD1BbB4f9b936C3604906D7592A644071dE884` |
| Avalanche | 43114 | `0xBBbD1BbB4f9b936C3604906D7592A644071dE884` |

---

## Commands

### `get-tokens` - List Supported Tokens

List all tokens available for bridging on each chain.

**Usage:**
```
allbridge-classic get-tokens
```

**Example:**
```bash
allbridge-classic get-tokens
```

**Output:** JSON with chains (ETH, BSC, POL, AVA, SOL) and their supported tokens, fees, and decimals.

---

### `get-tx-status` - Check Bridge Transaction Status

Check if a bridge transfer has been confirmed by the Allbridge network.

**Usage:**
```
allbridge-classic get-tx-status --lock-id <LOCK_ID>
```

**Parameters:**
| Parameter | Required | Description |
|---|---|---|
| `--lock-id` | Yes | The lock ID decimal (from bridge transaction, saved after `bridge` command) |

**Example:**
```bash
allbridge-classic get-tx-status --lock-id 199936896233321369426533897768825075
```

**Output:** Bridge confirmation details including source, destination, amount, and recipient.

**Notes:**
- Returns error if lock not yet confirmed - wait a few minutes and try again
- Lock ID is logged by the `bridge` command on success

---

### `bridge` - Bridge Tokens Cross-Chain

Transfer ERC-20 tokens from an EVM chain to any supported destination chain.

**Usage:**
```
allbridge-classic bridge --chain <CHAIN_ID> --token <SYMBOL> --amount <AMOUNT> --dest-chain <DEST> --recipient <ADDR> [--dry-run]
```

**Parameters:**
| Parameter | Required | Description |
|---|---|---|
| `--chain` | Yes | Source chain ID (1=Ethereum, 56=BSC, 137=Polygon, 43114=Avalanche) |
| `--token` | Yes | Token symbol (USDT, USDC, BUSD, etc.) |
| `--amount` | Yes | Amount in token units (e.g. `10.0` for 10 USDT) |
| `--dest-chain` | Yes | Destination chain (ETH, BSC, POL, AVA, SOL, FTM) |
| `--recipient` | Yes | Recipient address on destination chain |
| `--dry-run` | No | Show calldata without broadcasting |

**Bridge flow (2 transactions):**

This command will execute 2 on-chain transactions. **Ask the user to confirm before proceeding.**

1. **Approve** - ERC-20 approve for bridge contract amount
   - `onchainos wallet contract-call --chain <ID> --to <TOKEN_ADDR> --input-data <approve_calldata>`
2. **Lock** - Call bridge contract to lock tokens
   - `onchainos wallet contract-call --chain <ID> --to 0xBBbD1BbB4f9b936C3604906D7592A644071dE884 --input-data <lock_calldata>`

After lock is confirmed, use `get-tx-status --lock-id <lockId>` to track bridge progress.

**Examples:**
```bash
# Dry run: bridge 10 USDT from Ethereum to Solana
allbridge-classic bridge --chain 1 --token USDT --amount 10.0 --dest-chain SOL \
  --recipient DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE --dry-run

# Bridge 10 USDT from BSC to Polygon
allbridge-classic bridge --chain 56 --token USDT --amount 10.0 --dest-chain POL \
  --recipient 0x87fb0647faabea33113eaf1d80d67acb1c491b90
```

**Important warnings:**
- Bridge fee is deducted from the transferred amount
- Minimum bridge amount applies (check `get-tokens` for `minFee`)
- Cross-chain transfers take 5-15 minutes; do not retry if pending
- Save the `lockId` from output to track status with `get-tx-status`

---

### `check-address` - Validate Recipient Address

Validate that a recipient address is supported for a destination chain.

**Usage:**
```
allbridge-classic check-address --chain <CHAIN_ID> --address <ADDRESS>
```

**Parameters:**
| Parameter | Required | Description |
|---|---|---|
| `--chain` | Yes | Chain identifier (SOL, ETH, BSC, POL, AVA) |
| `--address` | Yes | Address to validate |

**Example:**
```bash
allbridge-classic check-address --chain SOL --address DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE
allbridge-classic check-address --chain ETH --address 0x87fb0647faabea33113eaf1d80d67acb1c491b90
```

---

## Trigger Examples

Use these natural language phrases to invoke the plugin:

- "Bridge 10 USDT from Ethereum to Solana"
- "Transfer USDC from BSC to Polygon using Allbridge"
- "Check my Allbridge bridge transaction status for lock ID 1234..."
- "What tokens can I bridge with Allbridge Classic?"
- "Is this Solana address valid for Allbridge? [address]"
- "Show me Allbridge Classic supported chains"
