# Selector Verification — StakeStone

All selectors verified with `cast sig`.

| Operation | Function Signature | Selector | Status |
|-----------|-------------------|---------|--------|
| stake | `deposit()` | `0xd0e30db0` | ✅ verified |
| request-withdraw | `requestWithdraw(uint256)` | `0x745400c9` | ✅ verified |
| cancel-withdraw | `cancelWithdraw(uint256)` | `0x9f01f7ba` | ✅ verified |
| get-rate: share price | `currentSharePrice()` | `0x28a79576` | ✅ verified |
| get-rate: round ID | `latestRoundID()` | `0xf76339dc` | ✅ verified |
| get-rate: fee rate | `withdrawFeeRate()` | `0xea99e689` | ✅ verified |
| get-position: STONE balance | `balanceOf(address)` | `0x70a08231` | ✅ verified |
| get-position: receipts | `userReceipts(address)` | `0xa4786f3d` | ✅ verified |
| get-rate: vault amounts | `getVaultAvailableAmount()` | `0x82f1631f` | ✅ verified |
