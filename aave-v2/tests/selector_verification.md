# Selector Verification for Aave V2 Plugin

## Method

Selectors computed using Python `eth_utils.keccak`:
```python
from eth_utils import keccak
selector = keccak(text="function_signature")[:4].hex()
```

## Aave V2 LendingPool Selectors

| Function                                              | Selector     | Notes                              |
|-------------------------------------------------------|--------------|------------------------------------|
| `deposit(address,uint256,address,uint16)`             | `0xe8eda9df` | V2 ONLY — V3 uses `supply()`       |
| `withdraw(address,uint256,address)`                   | `0x69328dec` | Same as V3                         |
| `borrow(address,uint256,uint256,uint16,address)`      | `0xa415bcad` | Same as V3                         |
| `repay(address,uint256,uint256,address)`              | `0x573ade81` | Same as V3                         |
| `getReservesList()`                                   | `0xd1946dbc` | Same as V3                         |
| `getReserveData(address)`                             | `0x35ea6a75` | Same selector, different slots     |
| `getUserAccountData(address)`                         | `0xbf92857c` | Same as V3                         |

## LendingPoolAddressesProvider Selectors

| Function             | Selector     |
|----------------------|--------------|
| `getLendingPool()`   | `0x0261bf8b` |

## ERC-20 Selectors

| Function                      | Selector     |
|-------------------------------|--------------|
| `approve(address,uint256)`    | `0x095ea7b3` |
| `balanceOf(address)`          | `0x70a08231` |
| `allowance(address,address)`  | `0xdd62ed3e` |

## Key Aave V2 vs V3 Difference

The most critical difference: Aave V2 uses `deposit()` with selector `0xe8eda9df`,
while Aave V3 uses `supply()` with selector `0x617ba037`.

Using the wrong selector would cause the transaction to fail silently or call
a different function.

## ReserveData Struct Slot Layout (V2 vs V3)

V2 `DataTypes.ReserveData` slots:
- Slot 3: `currentLiquidityRate` (supply APY)
- Slot 4: `currentVariableBorrowRate`
- Slot 5: `currentStableBorrowRate`

V3 `DataTypes.ReserveData` slots:
- Slot 2: `currentLiquidityRate` (supply APY)  ← one slot earlier
- Slot 4: `currentVariableBorrowRate`

Using wrong slot indices would return incorrect APY values.

## getUserAccountData Return Values

V2 returns ETH-denominated values:
- totalCollateralETH (1e18)
- totalDebtETH (1e18)
- availableBorrowsETH (1e18)

V3 returns USD-denominated values:
- totalCollateralBase (1e8)
- totalDebtBase (1e8)
- availableBorrowsBase (1e8)
