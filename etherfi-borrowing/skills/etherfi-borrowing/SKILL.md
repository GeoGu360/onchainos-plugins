---
name: etherfi-borrowing
version: "0.1.0"
description: "EtherFi Cash borrowing protocol on Scroll - supply USDC liquidity, view rates and positions, repay debt"
author:
  name: "GeoGu360"
  github: "GeoGu360"
binary: etherfi-borrowing
---

# EtherFi Borrowing (Cash) Skill

EtherFi Cash is a credit-card-backed borrowing protocol on Scroll. Users can deposit weETH/SCR as collateral in their UserSafe smart wallet and borrow USDC. This skill supports querying rates, checking positions, supplying USDC liquidity to earn yield, and repaying debt.

**Network:** Scroll (chain ID 534352)
**Borrow token:** USDC
**Collateral tokens:** weETH, USDC, SCR

> **Note:** Direct borrowing requires a UserSafe smart wallet created via the EtherFi Cash app (app.ether.fi). This skill supports liquidity supply, position queries, and repayment.

---

## Commands

### markets

Show supported borrow and collateral tokens with LTV and liquidation parameters.

**Trigger phrases:**
- "show EtherFi Cash markets"
- "what collateral can I use on EtherFi Cash?"
- "what tokens does EtherFi borrowing support?"
- "EtherFi Cash supported tokens"

**Usage:**
```
etherfi-borrowing markets [--chain 534352] [--rpc-url <URL>]
```

**Example output:**
```json
{
  "ok": true,
  "chain": "Scroll (534352)",
  "borrow_markets": [
    {
      "token": "USDC",
      "total_supply": "5.0050",
      "total_borrow": "10.3332",
      "borrow_apy_pct": "0.000000",
      "utilization_pct": "206.44"
    }
  ],
  "collateral_markets": [
    {
      "token": "weETH",
      "ltv_pct": "50.0",
      "liquidation_threshold_pct": "75.0",
      "liquidation_bonus_pct": "1.00"
    }
  ]
}
```

---

### rates

Show current borrowing APY and protocol liquidity statistics.

**Trigger phrases:**
- "what are EtherFi Cash borrowing rates?"
- "EtherFi Cash interest rate"
- "show EtherFi borrowing APY"
- "how much does it cost to borrow on EtherFi Cash?"

**Usage:**
```
etherfi-borrowing rates [--chain 534352] [--rpc-url <URL>]
```

---

### position

Show a UserSafe's current collateral, debt, and borrowing capacity.

**Trigger phrases:**
- "show my EtherFi Cash position"
- "check EtherFi Cash debt for [address]"
- "how much can I still borrow on EtherFi Cash?"
- "is my EtherFi Cash position healthy?"

**Usage:**
```
etherfi-borrowing position --user-safe <ADDRESS> [--chain 534352]
```

**Parameters:**
- `--user-safe` — UserSafe contract address

---

### supply-liquidity

Supply USDC to the EtherFi Cash debt manager to earn yield from borrowers.

**Trigger phrases:**
- "supply USDC to EtherFi Cash"
- "provide liquidity to EtherFi Cash"
- "deposit USDC to earn yield on EtherFi Cash"
- "lend USDC on EtherFi Cash"

**Usage:**
```
etherfi-borrowing supply-liquidity --amount <AMOUNT> [--chain 534352] [--dry-run]
```

**Parameters:**
- `--amount` — Amount of USDC to supply (e.g. `0.01`)
- `--dry-run` — Simulate without broadcasting

**On-chain operations (requires user confirmation):**
1. ERC-20 approve USDC to DebtManager (if needed)
   - `onchainos wallet contract-call --chain 534352 --to <USDC> --input-data <approve_calldata>`
   - Please confirm this approval transaction.
2. supply() to DebtManager
   - `onchainos wallet contract-call --chain 534352 --to <DEBT_MANAGER> --input-data <supply_calldata>`
   - Please confirm this supply transaction.

---

### withdraw-liquidity

Withdraw previously supplied USDC liquidity from EtherFi Cash.

**Trigger phrases:**
- "withdraw USDC from EtherFi Cash"
- "remove liquidity from EtherFi Cash"
- "get back my USDC from EtherFi Cash"

**Usage:**
```
etherfi-borrowing withdraw-liquidity --amount <AMOUNT> [--chain 534352] [--dry-run]
```

**Parameters:**
- `--amount` — Amount of USDC to withdraw (e.g. `0.01`)
- `--dry-run` — Simulate without broadcasting

**On-chain operation (requires user confirmation):**
- `onchainos wallet contract-call --chain 534352 --to <DEBT_MANAGER> --input-data <withdraw_calldata>`
- Please confirm this withdrawal transaction.

---

### repay

Repay USDC debt on behalf of a UserSafe.

**Trigger phrases:**
- "repay EtherFi Cash debt"
- "pay back USDC on EtherFi Cash"
- "repay my EtherFi borrowing"
- "reduce my EtherFi Cash debt"

**Usage:**
```
etherfi-borrowing repay --user-safe <ADDRESS> --amount <AMOUNT> [--chain 534352] [--dry-run]
```

**Parameters:**
- `--user-safe` — UserSafe contract address with the debt
- `--amount` — Amount of USDC to repay (e.g. `0.01`)
- `--dry-run` — Simulate without broadcasting

**On-chain operations (requires user confirmation):**
1. ERC-20 approve USDC to DebtManager (if needed)
   - `onchainos wallet contract-call --chain 534352 --to <USDC> --input-data <approve_calldata>`
   - Please confirm this approval transaction.
2. repay() to DebtManager
   - `onchainos wallet contract-call --chain 534352 --to <DEBT_MANAGER> --input-data <repay_calldata>`
   - Please confirm this repay transaction.

---

## Contract Addresses (Scroll Mainnet)

| Contract | Address |
|---|---|
| DebtManagerProxy | `0x8f9d2Cd33551CE06dD0564Ba147513F715c2F4a0` |
| USDC (Scroll) | `0x06eFdBFf2a14a7c8E15944D1F4A48F9F95F663A4` |
| weETH (Scroll) | `0x01f0a31698C4d065659b9bdC21B3610292a1c506` |

---

## Notes

- All operations run on **Scroll** (chain ID 534352), not Ethereum mainnet
- Borrowing directly requires a UserSafe smart wallet; create one at app.ether.fi
- Repay can be called by any EOA on behalf of a UserSafe
- Supply/withdraw work directly with EOA wallets (no UserSafe needed)
