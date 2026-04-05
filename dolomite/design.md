# Dolomite Plugin Design

## Overview
Dolomite is an isolated lending protocol built on a DolomiteMargin core contract (forked from dYdX Solo). All deposits, withdrawals, and borrows flow through a single `operate()` call on the DolomiteMargin contract.

## Chains
- **Arbitrum One** (42161) — primary, test chain
- Mantle (5000)
- Berachain (80094)

## Key Contracts (Arbitrum 42161)

| Contract | Address |
|---|---|
| DolomiteMargin (core) | `0x6Bd780E7fDf01D77e4d475c821f1e7AE05409072` |
| DepositWithdrawalProxy | `0xAdB9D68c613df4AA363B42161E1282117C7B9594` |

## Core Functions

### Read
- `getNumMarkets() → uint256` — selector `0x295c39a5`
- `getMarketTokenAddress(uint256 marketId) → address` — selector `0x062bd3e9`
- `getMarketTotalPar(uint256 marketId) → TotalPar` — selector `0xcb04a34c`
- `getMarketInterestRate(uint256 marketId) → Rate` — selector `0xfd47eda6`
- `getAccountBalances(Account.Info account) → (uint[], address[], Par[], Wei[])` — selector `0x6a8194e7`

### Write
- `operate(AccountInfo[] accounts, ActionArgs[] actions)` — selector `0xa67a6a45`
  All state-changing operations (deposit, withdraw, borrow, repay) go through this single entry point.

## ActionType Enum
```
Deposit  = 0
Withdraw = 1
Transfer = 2
Buy      = 3
Sell     = 4
Trade    = 5
Liquidate= 6
Vaporize = 7
Call     = 8
```

## ActionArgs Struct
```solidity
struct ActionArgs {
    ActionType actionType;   // 0=deposit, 1=withdraw
    uint256 accountId;       // index into accounts array (0)
    AssetAmount amount;      // {sign, denomination, ref, value}
    uint256 primaryMarketId; // market ID of token
    uint256 secondaryMarketId; // 0
    address otherAddress;    // from/to address
    uint256 otherAccountId;  // 0
    bytes data;              // empty
}

struct AssetAmount {
    bool sign;              // true=positive
    AssetDenomination denomination; // 0=Wei, 1=Par
    AssetReference ref;    // 0=Delta, 1=Target
    uint256 value;          // amount
}
```

## ABI Encoding for operate()

The `operate()` call encodes:
1. `AccountInfo[]` — array of `(address owner, uint256 number)` — typically just `[(walletAddress, 0)]`
2. `ActionArgs[]` — array of actions

For deposit (ActionType=0):
- accountId = 0
- amount = {sign: true, denomination: Wei(0), ref: Delta(0), value: rawAmount}
- primaryMarketId = marketId of token
- otherAddress = from address (wallet)

For withdraw (ActionType=1):
- accountId = 0
- amount = {sign: false, denomination: Wei(0), ref: Delta(0), value: rawAmount} (or Target=1 for max)
- primaryMarketId = marketId
- otherAddress = to address (wallet)

For borrow (ActionType=1 but with negative balance intent — borrow against collateral):
- Similar to withdraw but creates negative balance

For repay (ActionType=0 but paying back debt):
- Similar to deposit but reduces negative balance

## Supported Operations

| Operation | RW | Notes |
|---|---|---|
| markets | R | List all markets with symbol, TVL, rates |
| positions | R | User's balance per market |
| deposit | W | Supply tokens to earn yield |
| withdraw | W | Retrieve supplied tokens |
| borrow | W (dry-run) | Borrow (liquidation risk) |
| repay | W (dry-run) | Repay debt |

## Market ID Mapping (Arbitrum)
Markets are identified by integer IDs. Common ones:
- 0: USDC
- 2: WETH
- 17: USDT
(Fetched dynamically via getNumMarkets + getMarketTokenAddress)

## ERC-20 Approve
Before deposit/repay, must approve DolomiteMargin to spend tokens.
Selector: `0x095ea7b3`

## Known Tokens (Arbitrum 42161)
- USDC: `0xaf88d065e77c8cC2239327C5EDb3A432268e5831` (6 decimals)
- WETH: `0x82aF49447D8a07e3bd95BD0d56f35241523fBab1` (18 decimals)
- USDT: `0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9` (6 decimals)
