# Aave V2 Plugin Design

## Overview

Aave V2 is the classic lending pool on Ethereum mainnet. This plugin provides direct
contract interaction with the Aave V2 LendingPool and ProtocolDataProvider contracts.

**Key difference from Aave V3**: V2 uses `deposit()` instead of `supply()`, and the
LendingPool (not Pool) is the main contract. Function signatures differ from V3.

## Chain Support

- **Ethereum Mainnet** (chain ID: 1)
  - Test chain for L3/L4 tests

## Contract Addresses (Ethereum Mainnet)

| Contract                    | Address                                      |
|-----------------------------|----------------------------------------------|
| LendingPool (proxy)         | `0x7d2768dE32b0b80b7a3454c06BdAc94A69DDc7A9` |
| LendingPoolAddressesProvider| `0xB53C1a33016B2DC2fF3653530bfF1848a515c8c5` |
| ProtocolDataProvider        | `0x057835Ad21a177dbdd3090bB1CAE03EaCF78Fc6d` |

LendingPool address is resolved at runtime via LendingPoolAddressesProvider.getLendingPool()
for safety (the proxy is stable, but using the provider is best practice).

## Aave V2 vs V3 Function Signature Differences

| Operation | Aave V2                                                          | Aave V3                                           |
|-----------|------------------------------------------------------------------|---------------------------------------------------|
| Deposit   | `deposit(address,uint256,address,uint16)` → `0xe8eda9df`        | `supply(address,uint256,address,uint16)` → `0x617ba037` |
| Withdraw  | `withdraw(address,uint256,address)` → `0x69328dec`              | Same selector → `0x69328dec`                     |
| Borrow    | `borrow(address,uint256,uint256,uint16,address)` → `0xa415bcad` | Same selector → `0xa415bcad`                     |
| Repay     | `repay(address,uint256,uint256,address)` → `0x573ade81`         | Same selector → `0x573ade81`                     |

Note: The main difference is `deposit` vs `supply`. V2 doesn't have E-Mode or set-collateral via separate function.

## Selector Verification (via Python keccak256)

```
deposit(address,uint256,address,uint16)      => 0xe8eda9df
withdraw(address,uint256,address)            => 0x69328dec
borrow(address,uint256,uint256,uint16,address) => 0xa415bcad
repay(address,uint256,uint256,address)       => 0x573ade81
getReservesList()                            => 0xd1946dbc
getReserveData(address)                      => 0x35ea6a75
getUserAccountData(address)                  => 0xbf92857c
getLendingPool()                             => 0x0261bf8b (LendingPoolAddressesProvider)
```

## Operations

### Read Operations
1. **reserves** — List all Aave V2 reserves with supply/borrow APYs
   - Calls LendingPool.getReservesList() then LendingPool.getReserveData(asset) per reserve
   - Returns: asset, supplyApy, variableBorrowApy, stableBorrowApy

2. **positions** — Show user's aToken and debt token balances
   - Calls LendingPool.getUserAccountData(user) for health factor
   - Uses onchainos defi positions for enriched data

### Write Operations
3. **deposit** — Supply assets to Aave V2 LendingPool
   - Signature: `deposit(address asset, uint256 amount, address onBehalfOf, uint16 referralCode)`
   - Requires ERC-20 approve before deposit
   - Selector: `0xe8eda9df`

4. **withdraw** — Withdraw supplied assets
   - Signature: `withdraw(address asset, uint256 amount, address to)`
   - Selector: `0x69328dec`

5. **borrow** (dry-run only) — Borrow against collateral
   - Signature: `borrow(address asset, uint256 amount, uint256 interestRateMode, uint16 referralCode, address onBehalfOf)`
   - Selector: `0xa415bcad`
   - V2 supports both stable (1) and variable (2) rate modes

6. **repay** (dry-run only) — Repay borrowed debt
   - Signature: `repay(address asset, uint256 amount, uint256 rateMode, address onBehalfOf)`
   - Selector: `0x573ade81`

## ReserveData Struct Slot Layout (V2 LendingPool.getReserveData)

The V2 DataTypes.ReserveData struct differs from V3:
- Slot 0: configuration (packed bitmask)
- Slot 1: liquidityIndex (ray)
- Slot 2: variableBorrowIndex (ray)
- Slot 3: currentLiquidityRate (supply APY, ray = 1e27)
- Slot 4: currentVariableBorrowRate (variable APY, ray = 1e27)
- Slot 5: currentStableBorrowRate (stable APY, ray = 1e27)
- Slot 6: lastUpdateTimestamp (uint40 packed)
- Slot 7: aTokenAddress
- Slot 8: stableDebtTokenAddress
- Slot 9: variableDebtTokenAddress
- Slot 10: interestRateStrategyAddress
- Slot 11: id (uint8)

## Safety

- borrow and repay: **dry-run only** (liquidation risk)
- deposit and withdraw: live on L4 with small USDT test amount (0.01 USDT max)
- Reserve ≥ 0.001 ETH for gas

## USDT Address (Ethereum Mainnet)

`0xdAC17F958D2ee523a2206206994597C13D831ec7` (6 decimals)
