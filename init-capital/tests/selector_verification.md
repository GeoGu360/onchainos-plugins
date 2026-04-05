# Selector Verification — INIT Capital

All selectors verified with `cast sig` (Foundry).

## MoneyMarketHook

| Function | Signature | Selector | Verified |
|----------|-----------|----------|---------|
| execute | `execute((uint256,address,uint16,(address,uint256,(address,address))[],(address,uint256,(address,address),address)[],(address,uint256,address)[],(address,uint256)[],uint256,bool))` | `0x247d4981` | ✅ |

## ERC-20

| Function | Signature | Selector | Verified |
|----------|-----------|----------|---------|
| approve | `approve(address,uint256)` | `0x095ea7b3` | ✅ |

## LendingPool

| Function | Signature | Selector | Verified |
|----------|-----------|----------|---------|
| totalAssets | `totalAssets()` | `0x01e1d114` | ✅ |
| getSupplyRate_e18 | `getSupplyRate_e18()` | `0xbea205a2` | ✅ |
| getBorrowRate_e18 | `getBorrowRate_e18()` | `0x8d80c344` | ✅ |
| toShares | `toShares(uint256)` | `0x9e57c975` | ✅ |
| toAmt | `toAmt(uint256)` | `0x183c268e` | ✅ |
| debtShareToAmtCurrent | `debtShareToAmtCurrent(uint256)` | `0x31a86fe1` | ✅ |

## POS_MANAGER

| Function | Signature | Selector | Verified |
|----------|-----------|----------|---------|
| getViewerPosIdsLength | `getViewerPosIdsLength(address)` | `0x0c4478c7` | ✅ |
| getViewerPosIdsAt | `getViewerPosIdsAt(address,uint256)` | `0xfd379b17` | ✅ |
| getPosMode | `getPosMode(uint256)` | `0xf92c4d4c` | ✅ |
| getCollAmt | `getCollAmt(uint256,address)` | `0x402414b3` | ✅ |
| getPosDebtShares | `getPosDebtShares(uint256,address)` | `0x10e28e71` | ✅ |

## INIT_CORE

| Function | Signature | Selector | Verified |
|----------|-----------|----------|---------|
| getPosHealthCurrent_e18 | `getPosHealthCurrent_e18(uint256)` | `0xa72ca39b` | ✅ |
