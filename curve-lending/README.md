# curve-lending

Curve Lending (LlamaLend) plugin for onchainos Plugin Store.

## Overview

Curve Lending is an isolated lending product from Curve Finance. Users can:
- Deposit ETH, wstETH, tBTC, or other collateral to borrow crvUSD
- Lend crvUSD to earn yield
- Monitor positions with LLAMMA band pricing and health factors

Each market is isolated with its own Controller and LLAMMA AMM.

## Commands

| Command | Description |
|---------|-------------|
| `markets` | List all lending markets with TVL and loan activity |
| `rates` | View borrow/lend APY for markets |
| `positions` | Show active borrowing positions for a wallet |
| `deposit-collateral` | Deposit collateral (supports --dry-run) |
| `borrow` | Borrow crvUSD (dry-run in test env) |
| `repay` | Repay crvUSD debt (dry-run in test env) |

## Chain Support

- Ethereum mainnet (chain ID: 1) — primary

## Build

```bash
cargo build --release
./target/release/curve-lending markets --chain 1
```

## Key Contracts (Ethereum)

- OneWayLendingFactory: `0xeA6876DDE9e3467564acBeE1Ed5bac88783205E0`
- crvUSD: `0xf939E0A03FB07F59A73314E73794Be0E57ac1b4e`
