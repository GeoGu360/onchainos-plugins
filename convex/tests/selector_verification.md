# Selector Verification Checklist

All selectors verified with `cast sig` (Foundry).

| Function Signature | cast sig Result | Code Value | Status |
|-------------------|----------------|------------|--------|
| `approve(address,uint256)` | 0x095ea7b3 | 0x095ea7b3 | ✅ |
| `stake(uint256)` | 0xa694fc3a | 0xa694fc3a (alloy sol! macro) | ✅ |
| `withdraw(uint256,address,bool)` | 0x00ebf5dd | 0x00ebf5dd (alloy sol! macro) | ✅ |
| `lock(uint256,uint256)` | 0x1338736f | 0x1338736f (alloy sol! macro) | ✅ |
| `processExpiredLocks(bool)` | 0x312ff839 | 0x312ff839 (alloy sol! macro) | ✅ |
| `getReward(address,bool)` | 0x7050ccd9 | 0x7050ccd9 (hardcoded in claim_rewards.rs) | ✅ |
| `getReward()` | 0x3d18b912 | 0x3d18b912 (hardcoded in claim_rewards.rs) | ✅ |
| `balanceOf(address)` | 0x70a08231 | 0x70a08231 (hardcoded in rpc.rs) | ✅ |
| `allowance(address,address)` | 0xdd62ed3e | 0xdd62ed3e (hardcoded in rpc.rs) | ✅ |
| `earned(address)` | 0x008cc262 | 0x008cc262 (hardcoded in rpc.rs) | ✅ |
