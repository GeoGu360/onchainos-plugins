---
name: aave-v2
description: "Aave V2 classic lending pool on Ethereum. Trigger phrases: deposit to aave v2, aave v2 reserves, my aave v2 positions, aave v2 interest rates, withdraw from aave v2, borrow aave v2, repay aave v2. Chinese: Aave V2 cun kuan, Aave V2 jie kuan, Aave V2 huan kuan, wo de Aave V2 positions, Aave V2 li lv, Aave jing dian chi. Do NOT use for Aave V3 operations (supply, e-mode, multi-chain) -- use aave-v3 instead. Do NOT use for any chain other than Ethereum mainnet (chain 1). Do NOT use for swap, bridge, or token price queries."
license: MIT
metadata:
  author: GeoGu360
  version: "0.1.0"
---

# Aave V2 Skill

## ⚠️ Protocol Status: Deprecated

Aave V2 has been **officially deprecated** by Aave governance. All reserves are **frozen** — new deposits and borrows are rejected at the contract level.

**What still works:**
- ✅ `reserves` / `positions` — read market data and view existing positions
- ✅ `withdraw` — existing suppliers can withdraw their funds
- ❌ `deposit` — will fail on-chain (reserves frozen)
- ❌ `borrow` / `repay` — dry-run only regardless

**Recommendation:** For active lending/borrowing, use **Aave V3** (`aave-v3`) instead.

---

Interact with the Aave V2 classic LendingPool on Ethereum mainnet.

## Commands

### Read Commands (safe, no wallet needed)

#### `reserves`
List all Aave V2 reserves with supply and borrow APYs.

```
aave-v2 reserves --chain 1
aave-v2 reserves --chain 1 --asset 0xdAC17F958D2ee523a2206206994597C13D831ec7
```

#### `positions`
View your aToken deposits and debt positions.

```
aave-v2 positions --chain 1
aave-v2 positions --chain 1 --from 0xYourAddress
```

### Write Commands (require wallet confirmation)

> **IMPORTANT**: Before executing deposit or withdraw, always ask the user to confirm
> the transaction details — asset, amount, and chain. These operations move real funds.

#### `deposit`
Deposit an asset to earn interest (you receive aTokens).

```
aave-v2 deposit --asset USDT --amount 0.01 --chain 1
aave-v2 deposit --asset USDT --amount 0.01 --chain 1 --dry-run  # simulate first
```

**Steps**: (1) approve LendingPool for ERC-20 spend → (2) LendingPool.deposit()

#### `withdraw`
Withdraw a previously deposited asset.

```
aave-v2 withdraw --asset USDT --amount 0.01 --chain 1
aave-v2 withdraw --asset USDT --all --chain 1   # withdraw everything
```

### Dry-Run Only Commands (liquidation risk)

> **borrow** and **repay** are restricted to `--dry-run` mode to prevent accidental
> liquidation. Always simulate before executing any borrow/repay.

#### `borrow` (dry-run only)
Borrow an asset against posted collateral.

```
aave-v2 borrow --asset USDT --amount 1.0 --chain 1 --dry-run
aave-v2 borrow --asset USDT --amount 1.0 --rate-mode 2 --chain 1 --dry-run
```

Rate modes: `1` = stable, `2` = variable (default)

#### `repay` (dry-run only)
Repay borrowed debt.

```
aave-v2 repay --asset USDT --amount 1.0 --chain 1 --dry-run
aave-v2 repay --asset USDT --all --chain 1 --dry-run   # repay everything
```

## Notes

- Aave V2 uses `deposit()` (not `supply()` like V3) — different function selector
- Only Ethereum mainnet (chain 1) is supported for Aave V2
- Health factor < 1.0 triggers liquidation — monitor your positions
- aTokens accrue interest automatically, no claiming needed
- V2 still supports stable borrow rate (deprecated in V3)
