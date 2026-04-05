---
name: synthetix-v3
description: "Synthetix V3 perps market queries and collateral management on Base. Trigger phrases: synthetix markets, synthetix perps, synthetix positions, deposit collateral synthetix, withdraw collateral synthetix, synthetix v3. Chinese: Synthetix市场, Synthetix持仓, Synthetix存款, Synthetix抵押品"
license: MIT
metadata:
  author: GeoGu360
  version: "0.1.0"
---

# Synthetix V3 Skill

Interact with Synthetix V3 on Base (chain 8453) — query Perps markets, manage sUSDC collateral, and view account positions.

## Architecture

- **Read ops** (markets, positions, collateral) → direct `eth_call` via `https://base-rpc.publicnode.com`; no wallet confirmation needed
- **Write ops** (deposit-collateral, withdraw-collateral) → after user confirmation, submits via `onchainos wallet contract-call --chain 8453 --to <contract> --input-data <calldata>`
- All on-chain operations route through the onchainos CLI; no private keys are handled by this skill

## Commands

### markets — List Perps Markets

List all Synthetix V3 Perps markets with funding rates and open interest.

```
synthetix-v3 markets [--market-id <ID>]
```

**Parameters:**
- `--market-id` (optional): Query a specific market by ID (e.g. 100 for ETH, 200 for BTC)

**Example output:**
```json
{
  "ok": true,
  "chain": 8453,
  "protocol": "Synthetix V3 Perps",
  "total_markets": 100,
  "showing": 20,
  "markets": [
    {
      "market_id": 100,
      "symbol": "ETH",
      "skew": "12.5000",
      "size": "5000.0000",
      "max_open_interest": "30000.0000",
      "current_funding_rate": "0.00001234",
      "current_funding_velocity": "0.00000012"
    }
  ]
}
```

**Trigger phrases:**
- "Show me Synthetix perps markets"
- "What markets are available on Synthetix V3?"
- "Synthetix ETH funding rate"
- "Synthetix BTC market info"

---

### positions — Query Account Positions

Query open Perps positions for a Synthetix V3 account.

```
synthetix-v3 positions --account-id <ACCOUNT_ID>
```

**Parameters:**
- `--account-id` (required): Synthetix V3 account ID (uint128, e.g. 1234567890)

**Example output:**
```json
{
  "ok": true,
  "chain": 8453,
  "account_id": 1234567890,
  "available_margin": "1000.500000",
  "open_positions": [
    {
      "market_id": 100,
      "symbol": "ETH",
      "total_pnl": "12.500000",
      "accrued_funding": "-0.123456",
      "position_size": "0.500000"
    }
  ]
}
```

**Trigger phrases:**
- "Check my Synthetix positions for account 12345"
- "Show Synthetix V3 perps positions"
- "What are my open positions on Synthetix?"

---

### collateral — Query Collateral Balances

Query sUSDC collateral deposits and available-to-withdraw balance for an account.

```
synthetix-v3 collateral --account-id <ACCOUNT_ID>
```

**Parameters:**
- `--account-id` (required): Synthetix V3 account ID

**Example output:**
```json
{
  "ok": true,
  "chain": 8453,
  "account_id": 1234567890,
  "collaterals": [
    {
      "token": "sUSDC",
      "address": "0xC74eA762cF06c9151cE074E6a569a5945b6302E7",
      "total_deposited": 100.0,
      "total_assigned": 80.0,
      "total_locked": 0.0,
      "available_to_withdraw": 20.0
    }
  ]
}
```

**Trigger phrases:**
- "How much collateral do I have on Synthetix?"
- "Show Synthetix collateral for account 12345"
- "Synthetix V3 account balance"

---

### deposit-collateral — Deposit sUSDC Collateral

Deposit sUSDC as collateral into Synthetix V3 Core. This is a 2-step operation: ERC-20 approve + deposit.

```
synthetix-v3 deposit-collateral --account-id <ACCOUNT_ID> --amount <AMOUNT> [--from <ADDRESS>] [--dry-run]
```

**Parameters:**
- `--account-id` (required): Synthetix V3 account ID
- `--amount` (required): Amount of sUSDC to deposit (human-readable, e.g. `10.0`)
- `--from` (optional): Sender address (resolved from onchainos wallet if omitted)
- `--dry-run` (optional): Preview calldata without broadcasting

**Important: Ask user to confirm before proceeding with on-chain transaction.**

Run `--dry-run` first to preview the calldata, then **ask user to confirm** before executing the deposit.

**On-chain flow:**
1. Step 1: ERC-20 `approve(CoreProxy, amount)` on sUSDC token
2. Step 2: `deposit(accountId, sUSDC, amount)` on CoreProxy
3. Returns txHash for both transactions

**Example output:**
```json
{
  "ok": true,
  "action": "deposit-collateral",
  "account_id": 1234567890,
  "collateral": "sUSDC",
  "amount": 10.0,
  "approve_tx": "0xabc...",
  "tx_hash": "0xdef...",
  "explorer": "basescan.org/tx/<txHash>"
}
```

**Trigger phrases:**
- "Deposit 10 sUSDC to Synthetix account 12345"
- "Add collateral to my Synthetix V3 account"
- "Deposit collateral on Synthetix"

---

### withdraw-collateral — Withdraw sUSDC Collateral

Withdraw available sUSDC collateral from Synthetix V3 Core.

```
synthetix-v3 withdraw-collateral --account-id <ACCOUNT_ID> --amount <AMOUNT> [--from <ADDRESS>] [--dry-run]
```

**Parameters:**
- `--account-id` (required): Synthetix V3 account ID
- `--amount` (required): Amount of sUSDC to withdraw
- `--from` (optional): Sender address
- `--dry-run` (optional): Preview without broadcasting

**Important: Ask user to confirm before proceeding with on-chain transaction.**

Run `--dry-run` first to preview, then **ask user to confirm** before executing the withdrawal.

**Note:** Only unassigned (not delegated to a pool) collateral can be withdrawn. Check available balance with the `collateral` command first.

**On-chain flow:**
1. Checks `getAccountAvailableCollateral` — fails if amount exceeds available
2. Calls `withdraw(accountId, sUSDC, amount)` on CoreProxy
3. Returns txHash

**Example output:**
```json
{
  "ok": true,
  "action": "withdraw-collateral",
  "account_id": 1234567890,
  "collateral": "sUSDC",
  "amount": 5.0,
  "tx_hash": "0xabc...",
  "explorer": "basescan.org/tx/<txHash>"
}
```

**Trigger phrases:**
- "Withdraw 5 sUSDC from Synthetix account 12345"
- "Remove collateral from Synthetix V3"
- "Take out my Synthetix collateral"

---

## Key Contract Addresses (Base 8453)

| Contract | Address |
|----------|---------|
| CoreProxy | `0x32C222A9A159782aFD7529c87FA34b96CA72C696` |
| PerpsMarketProxy | `0x0A2AF931eFFd34b81ebcc57E3d3c9B1E1dE1C9Ce` |
| sUSDC Token | `0xC74eA762cF06c9151cE074E6a569a5945b6302E7` |

## Error Handling

- Missing account ID: Returns descriptive error message
- Insufficient collateral: Pre-flight check returns error before broadcasting
- Wallet not logged in: Returns "Cannot resolve wallet address" with instructions
