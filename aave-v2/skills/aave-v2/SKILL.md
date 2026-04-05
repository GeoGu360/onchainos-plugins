---
name: aave-v2
description: "Aave V2 classic lending pool on Ethereum. Trigger phrases: deposit to aave v2, aave v2 reserves, my aave v2 positions, aave v2 interest rates, withdraw from aave v2, borrow aave v2, repay aave v2. Chinese: Aave V2存款, Aave V2借款, Aave V2还款, 我的Aave V2仓位, Aave V2利率, Aave经典池"
license: MIT
metadata:
  author: GeoGu360
  version: "0.1.0"
---

# Aave V2 Skill

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
