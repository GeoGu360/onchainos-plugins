# Spark Savings Plugin — Design Document

## Overview

Spark Savings (by SparkFi / MakerDAO/Sky ecosystem) allows users to deposit USDS (and DAI on Ethereum) into savings vaults to earn the Sky Savings Rate (SSR) or DAI Savings Rate (DSR). On L2 chains (Base, Arbitrum, Optimism), deposits/withdrawals flow through the **Spark PSM3** contract, which swaps USDS ↔ sUSDS. On Ethereum, sUSDS is an ERC-4626 vault and sDAI is a separate ERC-4626 vault for DAI.

Current rates (as of research):
- SSR (sUSDS): **3.75% APY**
- DSR (sDAI): **1.25% APY**

## Architecture

### On L2s (Base, Arbitrum, Optimism)
- sUSDS is a **bridged ERC-20 token** (not an ERC-4626 vault locally)
- Deposit: `USDS.approve(PSM3, amount)` → `PSM3.swapExactIn(USDS, sUSDS, amount, minOut, receiver, 0)`
- Withdraw: `PSM3.swapExactIn(sUSDS, USDS, amount, minOut, receiver, 0)`
- APY from: Ethereum SSR oracle forwarded to L2 via SSR_AUTH_ORACLE

### On Ethereum (L1)
- **sUSDS**: ERC-4626 vault, `deposit(uint256 assets, address receiver)` → shares
- **sDAI**: ERC-4626 vault (legacy), uses DAI
- APY from: `sUSDS.ssr()` → per-second rate in ray → compute APY

## Contract Addresses

### Ethereum (1)
| Contract | Address |
|----------|---------|
| sUSDS (ERC-4626) | `0xa3931d71877C0E7a3148CB7Eb4463524FEc27fbD` |
| sDAI (ERC-4626)  | `0x83F20F44975D03b1b09e64809B757c47f942BEeA` |
| USDS             | `0xdC035D45d973E3EC169d2276DDab16f1e407384F` |
| DAI              | `0x6B175474E89094C44Da98b954EedeAC495271d0F` |
| MakerDAO Pot     | `0x197E90f9FAD81970bA7976f33CbD77088E5D7cf7` |

### Base (8453) — Primary test chain
| Contract | Address |
|----------|---------|
| PSM3              | `0x1601843c5E9bC251A3272907010AFa41Fa18347E` |
| sUSDS (bridged)   | `0x5875eEE11Cf8398102FdAd704C9E96607675467a` |
| USDS              | `0x820C137fa70C8691f0e44Dc420a5e53c168921Dc` |
| SSR_AUTH_ORACLE   | `0x65d946e533748A998B1f0E430803e39A6388f7a1` |
| SSR_CHAINLINK     | `0x026a5B6114431d8F3eF2fA0E1B2EDdDccA9c540E` |

### Arbitrum (42161)
| Contract | Address |
|----------|---------|
| PSM3              | `0x2B05F8e1cACC6974fD79A673a341Fe1f58d27266` |
| sUSDS (bridged)   | `0xdDb46999F8891663a8F2828d25298f70416d7610` |
| USDS              | (not listed in registry — check on-chain) |
| SSR_AUTH_ORACLE   | `0xEE2816c1E1eed14d444552654Ed3027abC033A36` |

### Optimism (10)
| Contract | Address |
|----------|---------|
| PSM3              | `0xe0F9978b907853F354d79188A3dEfbD41978af62` |
| sUSDS (bridged)   | `0xb5B2dc7fd34C249F4be7fB1fCea07950784229e0` |
| USDS              | `0x4F13a96EC5C4Cf34e442b46Bbd98a0791F20edC3` |
| SSR_AUTH_ORACLE   | `0x6E53585449142A5E6D5fC918AE6BEa341dC81C68` |

## Operations

### `apy` — Current savings APY (read)
- On Ethereum: call `sUSDS.ssr()` → per-second rate in ray → APY = (rate/1e27)^(365*24*3600) - 1
- On L2: call `SSR_AUTH_ORACLE.getConversionRate()` to get accumulator; also read Ethereum SSR for APY display
- Also show: current chi (accumulator) = conversion rate of sUSDS → USDS

### `balance` — User sUSDS balance (read)
- Call `sUSDS.balanceOf(user)` → shares
- Convert to USDS equivalent: shares × getConversionRate / 1e27

### `deposit` — Deposit USDS → sUSDS (write)
- On L2: `USDS.approve(PSM3)` + `PSM3.swapExactIn(USDS, sUSDS, amount, 0, receiver, 0)`
- On Ethereum: `USDS.approve(sUSDS)` + `sUSDS.deposit(amount, receiver)`
- Selector `swapExactIn(address,address,uint256,uint256,address,uint256)`: `0x1a019e37`
- Selector `deposit(uint256,address)`: `0x6e553f65`

### `withdraw` — Withdraw sUSDS → USDS (write)
- On L2: `PSM3.swapExactIn(sUSDS, USDS, sUSDS_amount, 0, receiver, 0)` (no approve needed for PSM3 with sUSDS)
- Wait: actually need `sUSDS.approve(PSM3)` before withdraw swap
- On Ethereum: `sUSDS.redeem(shares, receiver, owner)`
- Selector `redeem(uint256,address,address)`: `0xba087652`

### `markets` — Market info (read)
- PSM3 totalAssets (TVL)
- Current SSR/DSR rates
- sUSDS/USDS conversion rate

## Key Selectors (verified)
```
deposit(uint256,address)                            = 0x6e553f65
redeem(uint256,address,address)                     = 0xba087652
withdraw(uint256,address,address)                   = 0xb460af94
previewDeposit(uint256)                             = 0xef8b30f7
convertToAssets(uint256)                            = 0x07a2d13a
balanceOf(address)                                  = 0x70a08231
totalAssets()                                       = 0x01e1d114
asset()                                             = 0x38d52e0f
approve(address,uint256)                            = 0x095ea7b3
allowance(address,address)                          = 0xdd62ed3e
swapExactIn(address,address,uint256,uint256,address,uint256) = 0x1a019e37
swapExactOut(address,address,uint256,uint256,address,uint256) = 0x051f86b5
previewSwapExactIn(address,address,uint256)         = 0x00d8088a
deposit(address,address,uint256) [PSM3]             = 0x8340f549
withdraw(address,address,uint256) [PSM3]            = 0xd9caed12
getConversionRate() [oracle]                        = 0xf36089ec
ssr() [sUSDS Ethereum]                              = 0x03607ceb
dsr() [Pot]                                         = 0x487bf082
```

## APY Computation
```
ssr_ray = sUSDS.ssr()  // per-second rate, 1e27 precision
ssr_normalized = ssr_ray / 1e27
apy = ssr_normalized^(365*24*3600) - 1
```
Example: ssr_ray = 1000000001167363430498603315
→ ssr_normalized = 1.000000001167363...
→ APY ≈ 3.75%

## Rust Plugin Structure
```
spark-savings/
├── plugin.yaml
├── Cargo.toml
├── LICENSE
├── README.md
├── skills/spark-savings/SKILL.md
└── src/
    ├── main.rs         — CLI entry point, clap subcommands
    ├── config.rs       — chain configs, contract addresses
    ├── rpc.rs          — eth_call helpers, decode utils
    ├── onchainos.rs    — wallet_contract_call, resolve_wallet
    └── commands/
        ├── mod.rs
        ├── apy.rs      — read SSR/DSR from oracle/contract
        ├── balance.rs  — balanceOf + convertToAssets
        ├── deposit.rs  — approve + swapExactIn (L2) or ERC4626 deposit (L1)
        ├── withdraw.rs — approve + swapExactIn reverse (L2) or ERC4626 redeem (L1)
        └── markets.rs  — PSM3 totalAssets + rates
```
