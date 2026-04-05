# Selector Verification

All selectors verified via Python eth_utils keccak256.

| Function Signature | Expected Selector | Method | Status |
|-------------------|-------------------|--------|--------|
| supply(address,uint256) | 0xf2b9fdb8 | eth_utils keccak + live allMarkets() call | ✅ |
| borrow(address,uint256) | 0x4b8a3529 | eth_utils keccak | ✅ |
| redeemUnderlying(address,uint256) | 0x96294178 | eth_utils keccak | ✅ |
| redeemToken(address,uint256) | 0x830cbbbd | eth_utils keccak | ✅ |
| repayBorrow(address,uint256) | 0xabdb5ea8 | eth_utils keccak | ✅ |
| allMarkets() | 0x375a7cba | eth_utils keccak + live RPC test | ✅ |
| accountLiquidityOf(address) | 0xf8982e7a | eth_utils keccak + live RPC test | ✅ |
| marketInfoOf(address) | 0x6e8584fd | eth_utils keccak + live RPC test | ✅ |
| getUnderlyingPrice(address) | 0xfc57d4df | eth_utils keccak + live RPC test ($2033 ETH) | ✅ |
| exchangeRate() | 0x3ba0b9a9 | eth_utils keccak + live RPC test (1.02) | ✅ |
| totalBorrow() | 0x8285ef40 | eth_utils keccak + live RPC test (46 ETH) | ✅ |
| getCash() | 0x3b1d21a2 | eth_utils keccak + live RPC test (52 ETH) | ✅ |
| totalSupply() | 0x18160ddd | eth_utils keccak + live RPC test | ✅ |
| accountSnapshot(address) | 0x014a296f | eth_utils keccak + live RPC test | ✅ |
| borrowBalanceOf(address) | 0x374c49b4 | eth_utils keccak + live RPC test (0 for wallet) | ✅ |
| balanceOf(address) | 0x70a08231 | eth_utils keccak + live RPC test | ✅ |
| approve(address,uint256) | 0x095ea7b3 | standard ERC-20, well-known | ✅ |
