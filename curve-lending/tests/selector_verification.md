# Curve Lending — Function Selector Verification

All selectors verified via `cast sig` (Foundry). Final verified set below.

## OneWayLendingFactory

| Function | Selector | Method | Status |
|----------|---------|--------|--------|
| `market_count()` | `0xfd775c78` | cast sig | ✅ |
| `names(uint256)` | `0x4622ab03` | cast sig | ✅ |
| `controllers(uint256)` | `0xe94b0dd2` | cast sig | ✅ |
| `vaults(uint256)` | `0x8c64ea4a` | cast sig | ✅ |
| `collateral_tokens(uint256)` | `0x49b89984` | cast sig | ✅ |
| `borrowed_tokens(uint256)` | `0x6fe4501f` | cast sig | ✅ |
| `monetary_policies(uint256)` | `0x762e7b92` | cast sig | ✅ |
| `amms(uint256)` | `0x86a8cdbc` | cast sig | ✅ |

## Controller

| Function | Selector | Method | Status |
|----------|---------|--------|--------|
| `n_loans()` | `0x6cce39be` | cast sig + live | ✅ |
| `total_debt()` | `0x31dc3ca8` | cast sig + live | ✅ |
| `loan_exists(address)` | `0xa21adb9e` | cast sig + live | ✅ |
| `debt(address)` | `0x9b6c56ec` | cast sig + live | ✅ |
| `user_state(address)` | `0xec74d0a8` | cast sig + live | ✅ |
| `user_prices(address)` | `0x2c5089c3` | cast sig + live | ✅ |
| `health(address,bool)` | `0x8908ea82` | cast sig + live | ✅ |
| `max_borrowable(uint256,uint256)` | `0x9a497196` | cast sig + live | ✅ |
| `min_collateral(uint256,uint256)` | `0xa7573206` | cast sig | ✅ |
| `create_loan(uint256,uint256,uint256)` | `0x23cfed03` | cast sig | ✅ |
| `add_collateral(uint256,address)` | `0x24049e57` | cast sig | ✅ |
| `remove_collateral(uint256,bool)` | `0x2e4af52a` | cast sig | ✅ |
| `borrow_more(uint256,uint256)` | `0xdd171e7c` | cast sig | ✅ |
| `repay(uint256,address,int256,bool)` | `0x37671f93` | cast sig | ✅ |
| `repay(uint256)` | `0x371fd8e6` | cast sig + live (revert "Loan doesn't exist") | ✅ |

## Vault (ERC-4626)

| Function | Selector | Method | Status |
|----------|---------|--------|--------|
| `totalAssets()` | `0x01e1d114` | cast sig + live | ✅ |
| `convertToAssets(uint256)` | `0x07a2d13a` | cast sig + live | ✅ |
| `deposit(uint256,address)` | `0x6e553f65` | cast sig | ✅ |
| `redeem(uint256,address,address)` | `0xba087652` | cast sig | ✅ |
| `lend_apy()` | `0x1eb25c42` | cast sig (returns 0 live) | ✅ |
| `borrow_apy()` | `0x3ca97d20` | cast sig (returns 0 live) | ✅ |
| `totalSupply()` | `0x18160ddd` | cast sig + live | ✅ |
| `asset()` | `0x38d52e0f` | cast sig + live | ✅ |

## MonetaryPolicy

| Function | Selector | Method | Status |
|----------|---------|--------|--------|
| `rate(address)` | `0x0ba9d8ca` | cast sig + live (326278881/s) | ✅ |
| `min_rate()` | `0x5d786401` | cast sig + live (31709791/s = 0.1% APY) | ✅ FIXED (was 0xd22565a8) |
| `max_rate()` | `0x536e4ec4` | cast sig + live (22196854388/s = 101% APY) | ✅ FIXED (was 0x2c4e722e) |

## ERC-20

| Function | Selector | Method | Status |
|----------|---------|--------|--------|
| `approve(address,uint256)` | `0x095ea7b3` | cast sig | ✅ |
| `symbol()` | `0x95d89b41` | cast sig + live | ✅ |
| `decimals()` | `0x313ce567` | cast sig + live | ✅ |
| `balanceOf(address)` | `0x70a08231` | cast sig | ✅ |
