# Euler V2 Plugin Design

## Overview

Euler V2 is a modular lending protocol built around the Ethereum Vault Connector (EVC) and EVault architecture. Each market is an EVault — an ERC-4626-compatible vault with added borrowing functionality. Vaults are permissionlessly deployable, but "governed" vaults are verified by the GovernedPerspective contract.

## Architecture

- **EVC (Ethereum Vault Connector)**: Central coordinator for account management, batch operations, and cross-vault interactions
- **EVault**: ERC-4626-compatible vault per asset. Supports deposit, withdraw (assets), mint, redeem (shares), borrow, repay
- **GenericFactory (eVaultFactory)**: Registry of all deployed EVaults — `getProxyListLength()`, `getProxyListSlice(start, end)`
- **Lens Contracts**: Read-only helpers — VaultLens, AccountLens, UtilsLens, OracleLens, IRMLens
- **GovernedPerspective**: Contract that verifies "official" (governed) vaults

## Supported Chains

| Chain       | Chain ID | EVC                                         | eVaultFactory                               |
|-------------|----------|---------------------------------------------|---------------------------------------------|
| Ethereum    | 1        | 0x0C9a3dd6b8F28529d72d7f9cE918D493519EE383  | 0x29a56a1b8214D9Cf7c5561811750D5cBDb45CC8e  |
| Base        | 8453     | 0x5301c7dD20bD945D2013b48ed0DEE3A284ca8989  | 0x7F321498A801A191a93C840750ed637149dDf8D0  |
| Arbitrum    | 42161    | 0x6302ef0F34100CDDFb5489fbcB6eE1AA95CD1066  | 0x78Df1CF5bf06a7f27f2ACc580B934238C1b80D50  |
| Avalanche   | 43114    | 0xddcbe30A761Edd2e19bba930A977475265F36Fa1  | 0xaf4B4c18B17F6a2B32F6c398a3910bdCD7f26181  |
| BSC         | 56       | 0xb2E5a73CeE08593d1a076a2AE7A6e02925a640ea  | 0x7F53E2755eB3c43824E162F7F6F087832B9C9Df6  |

## Lens Contracts (Base 8453)

| Lens         | Address                                     |
|--------------|---------------------------------------------|
| accountLens  | 0xe6b05A38D6a29D2C8277fA1A8BA069F1693b780C  |
| vaultLens    | 0x601F023CD063324DdbCADa69460e969fb97e98b9  |
| utilsLens    | 0x506F74991664dA79b4f407e80adF118c76307c8E  |
| oracleLens   | 0xCE85cC424d12B8074bacB81c84dA7C6DA317c4D3  |
| irmLens      | 0xc159d463E7Cdb2C4bA8D4C0C877127A1fCdf33dC  |

## Known Vaults on Base (8453)

| Vault Name          | Vault Address                               | Asset Symbol | Asset Address                               | Decimals | TVL (approx)   |
|---------------------|---------------------------------------------|--------------|---------------------------------------------|----------|----------------|
| Euler WETH Vault    | 0x859160db5841e5cfb8d3f144c6b3381a85a4b410  | WETH         | 0x4200000000000000000000000000000000000006  | 18       | varies         |
| Euler USDC Vault    | 0x0a1a3b5f2041f33522c4efc754a7d096f880ee16  | USDC         | 0x833589fcd6edb6e08f4c7c32d4f71b54bda02913  | 6        | ~620K USDC     |
| Euler cbBTC Vault   | 0x7b181d6509deabfbd1a23af1e65fd46e89572609  | cbBTC        | 0xcbB7C0000aB88B473b1f5aFd9ef808440eed33Bf  | 8        | varies         |
| Euler cbETH Vault   | 0x358f25f82644eabb441d0df4af8746614fb9ea49  | cbETH        | 0x2ae3f1ec7f1f5012cfeab0185bfc7aa3cf0dec22  | 18       | varies         |

Factory has 257 vaults deployed on Base (4 Apr 2026). Markets command queries first 20 with live data.

## Key Contract Functions

### EVault (ERC-4626 + borrowing extensions)

```solidity
// Read
asset() -> address                               // 0x38d52e0f
totalAssets() -> uint256                         // 0x01e1d114
balanceOf(address owner) -> uint256              // 0x70a08231
convertToAssets(uint256 shares) -> uint256       // 0x07a2d13a
convertToShares(uint256 assets) -> uint256       // 0xc6e6f592
debtOf(address account) -> uint256               // 0xd283e75f
maxDeposit(address receiver) -> uint256          // 0x402d267d
maxWithdraw(address owner) -> uint256            // 0xce96cb77
interestRate() -> uint256                        // borrow APR, 1e27 scale
name() -> string                                 // 0x06fdde03
decimals() -> uint8                              // 0x313ce567

// Write
deposit(uint256 assets, address receiver) -> uint256 shares   // 0x6e553f65
redeem(uint256 shares, address receiver, address owner) -> uint256 assets // 0xba087652
withdraw(uint256 assets, address receiver, address owner) -> uint256 shares // 0xb460af94
borrow(uint256 amount, address receiver) -> uint256           // 0x4b3fd148
repay(uint256 amount, address receiver) -> uint256            // 0xacb70815
```

### GenericFactory

```solidity
getProxyListLength() -> uint256                  // 0x0a68b7ba
getProxyListSlice(uint256 start, uint256 end) -> address[] // 0xc0e96df6
```

## Operations

### markets (read)
- Query eVaultFactory.getProxyListSlice(0, 20) for first 20 vaults on Base
- For each: call asset(), name(), totalAssets(), interestRate()
- Display as table with asset symbol, TVL, supply/borrow rates
- Support --asset filter and --chain

### positions (read)
- Resolve wallet address
- Query accountLens.getAccountInfo(wallet, evc) for position summary
- For known vaults: check balanceOf(wallet), debtOf(wallet)
- Display supplied amounts, borrow balances

### supply (write)
- Resolve vault by asset symbol or address
- Step 1: ERC-20 approve(vault, amount) on underlying token
- Step 2: EVault.deposit(amount, receiver) — selector 0x6e553f65
- Confirm before executing (SKILL.md warns)

### withdraw (write)
- Resolve vault
- EVault.redeem(shares, receiver, owner) to withdraw all: selector 0xba087652
- Or EVault.withdraw(assets, receiver, owner) for exact amount: selector 0xb460af94

### borrow (dry-run only)
- EVault.borrow(amount, receiver) — selector 0x4b3fd148
- Always dry-run, document calldata

### repay (dry-run only)
- ERC-20 approve(vault, amount)
- EVault.repay(amount, receiver) — selector 0xacb70815
- Always dry-run

## RPC Strategy
- Base (8453): `https://mainnet.base.org` (primary), fallback `https://base-rpc.publicnode.com`
- Ethereum (1): `https://eth.llamarpc.com`
- Arbitrum (42161): `https://arbitrum-one-rpc.publicnode.com`
- Avalanche (43114): `https://api.avax.network/ext/bc/C/rpc`
- BSC (56): `https://bsc-rpc.publicnode.com`

## Interest Rate Decoding
- interestRate() returns per-second borrow rate in 1e27 (ray) units
- APR = rate * 365 * 24 * 3600 / 1e27
- Supply APY ≈ borrow_APR * utilization * (1 - fee)

## Notes
- borrow/repay are **dry-run only** due to liquidation risk
- EVC enableCollateral/enableController required before borrow — not implemented in plugin (informational only)
- Max 0.01 USDT or 0.00005 ETH per test tx; reserve ≥ 0.001 ETH
