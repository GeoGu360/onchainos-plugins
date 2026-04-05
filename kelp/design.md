# Kelp DAO Plugin Design

## Overview

Kelp DAO is a liquid restaking protocol built on top of EigenLayer. Users deposit ETH or liquid staking tokens (LSTs) to receive rsETH — a Liquid Restaking Token (LRT) that represents a user's restaked position. rsETH accrues both staking and restaking rewards.

- **Website**: https://kelpdao.xyz/ (now redirects to kerneldao.com/kelp/)
- **Docs**: https://docs.kelpdao.xyz/
- **GitHub**: https://github.com/Kelp-DAO/LRT-rsETH
- **Token**: rsETH (`0xA1290d69c65A6Fe4DF752f95823fae25cB99e5A7`)
- **TVL**: ~$1.27B (578,512 rsETH in circulation)
- **Price**: ~1.076 ETH per rsETH

## Supported Assets

Kelp accepts:
1. **ETH** — via `depositETH(uint256 minRSETHAmountExpected, string referralId)` (payable)
2. **stETH** (Lido) — `0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84`
3. **ETHx** (Stader) — held in deposit pool
4. **sfrxETH** (Frax) — supported via `depositAsset`

## Contract Addresses (Ethereum Mainnet, Chain ID 1)

| Contract | Address |
|---|---|
| RSETH Token | `0xA1290d69c65A6Fe4DF752f95823fae25cB99e5A7` |
| LRTDepositPool | `0x036676389e48133B63a802f8635AD39E752D375D` |
| LRTOracle | `0x349A73444b1a310BAe67ef67973022020d70020d` |
| LRTConfig | `0x947Cb49334e6571ccBFEF1f1f1178d8469D65ec7` |
| LRTWithdrawalManager | `0x62De59c08eB5dAE4b7E6F7a8cAd3006d6965ec16` |
| LRTUnstakingVault | `0xc66830E2667bc740c0BED9A71F18B14B8c8184bA` |
| LRTConverter | `0x598dbcb99711E5577fF76ef4577417197B939Dfa` |
| ProxyAdmin | `0xb61e0E39b6d4030C36A176f576aaBE44BF59Dc78` |
| TimelockController | `0x49bD9989E31aD35B0A62c20BE86335196A3135B1` |

## Key Function Signatures

### LRTDepositPool

```solidity
// Deposit ETH → rsETH (payable)
function depositETH(uint256 minRSETHAmountExpected, string calldata referralId) external payable;
// Selector: 0x72c51c0b

// Deposit LST → rsETH (requires prior ERC-20 approve)
function depositAsset(address asset, uint256 depositAmount, uint256 minRSETHAmountExpected, string calldata referralId) external;
// Selector: 0xc3ae1766

// Query: how much rsETH will I get?
function getRsETHAmountToMint(address asset, uint256 amount) external view returns (uint256);
// Selector: 0xba5bb442

// ETH address sentinel used by protocol for ETH deposits
// address(0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE)
```

### LRTOracle

```solidity
// Get rsETH price in ETH (18 decimals)
function rsETHPrice() external view returns (uint256);
// Selector: 0xb4b46434

// Get asset price in ETH
function getAssetCurrentPrice(address asset) external view returns (uint256);
// Selector: 0x7a95e516
```

### LRTWithdrawalManager

```solidity
// Initiate unstaking (burn rsETH, queue withdrawal)
function initiateWithdrawal(address asset, uint256 rsEthAmount) external;
// Selector: 0xc8393ba9

// Complete withdrawal (claim ETH/LST after unbonding)
function completeWithdrawal(address asset) external;
// Selector: 0x6dbaf9ee
```

### rsETH Token (ERC-20)

```solidity
function balanceOf(address) external view returns (uint256);
// Selector: 0x70a08231

function totalSupply() external view returns (uint256);
// Selector: 0x18160ddd
```

## Operations

### `apy` (read)
Fetch current rsETH APY. Sources:
1. CoinGecko API or Kelp API for APY data
2. Fall back: compute from oracle price movement
- Endpoint: `https://api.coingecko.com/api/v3/simple/price?ids=kelp-dao-restaked-eth&vs_currencies=eth,usd`

### `rates` (read)
Get rsETH/ETH exchange rate from LRTOracle.
- Call `rsETHPrice()` on LRTOracle → returns price in wei (1e18 = 1 ETH)
- Display: rsETH price in ETH and USD equivalent

### `positions` (read)
Get user's rsETH balance and underlying ETH value.
- Call `balanceOf(address)` on rsETH token
- Multiply by `rsETHPrice()` to get ETH value
- Also query pending withdrawals from LRTWithdrawalManager

### `stake` (write)
Deposit ETH → rsETH via LRTDepositPool.depositETH.
- Amount: ETH value passed as `--amt` (msg.value)
- minRSETHAmountExpected: 0 (or compute 1% slippage from oracle)
- referralId: "" (empty string)
- Calldata: `0x72c51c0b` + ABI-encoded params
- **User confirmation required**

### `unstake` (write)
Initiate rsETH withdrawal via LRTWithdrawalManager.initiateWithdrawal.
- asset: ETH address (`0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE`)
- rsEthAmount: amount to redeem
- **User confirmation required**
- Note: Withdrawal has a queue delay (typically several days)

## ABI Encoding Notes

### depositETH calldata
```
selector: 72c51c0b
param1 (uint256 minRSETHAmountExpected): 32 bytes, big-endian (0 for no minimum)
param2 (string referralId): ABI dynamic type
  - offset: 0x40 (64 bytes from start of params)
  - length: 0 (empty string)
  - data: (none)

Full calldata for depositETH(0, ""):
0x72c51c0b
0000000000000000000000000000000000000000000000000000000000000000  // minRSETH = 0
0000000000000000000000000000000000000000000000000000000000000040  // offset to string = 64
0000000000000000000000000000000000000000000000000000000000000000  // string length = 0
```

## Chains

| Chain | Chain ID | Status |
|---|---|---|
| Ethereum | 1 | Primary (full support) |
| Base | 8453 | rsETH bridged, limited |
| Arbitrum | 42161 | rsETH bridged, limited |
| Optimism | 10 | rsETH bridged, limited |

Primary chain for testing: **Ethereum (1)**

## Minimum Deposit

The LRTDepositPool may enforce a minimum deposit amount. Based on protocol documentation, minimum ETH deposit is likely around 0.001 ETH. This will be verified during Phase 3 testing. If minimum > 0.00005 ETH, L4 will remain dry-run only.

## Data Flow

```
User → kelp stake --amount 0.1 --chain 1
  → resolve_wallet(1) → "0x87fb..."
  → eth_call: rsETHPrice() on LRTOracle → 1.076 ETH/rsETH
  → eth_call: getRsETHAmountToMint(ETH_ADDR, 0.1e18) → expected rsETH
  → build calldata: depositETH(0, "")
  → [confirm] "Stake 0.1 ETH → ~0.0929 rsETH?"
  → wallet_contract_call(1, DEPOSIT_POOL, calldata, wallet, 0.1e18, dry_run)
  → print txHash
```

## References

- LRT-rsETH GitHub: https://github.com/Kelp-DAO/LRT-rsETH
- Etherscan (LRTDepositPool): https://etherscan.io/address/0x036676389e48133B63a802f8635AD39E752D375D
- Etherscan (rsETH): https://etherscan.io/token/0xa1290d69c65a6fe4df752f95823fae25cb99e5a7
- Etherscan (LRTOracle): https://etherscan.io/address/0x349A73444b1a310BAe67ef67973022020d70020d
- EigenLayer: https://docs.eigenlayer.xyz/
