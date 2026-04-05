# LayerBank Plugin

LayerBank lending protocol integration for the onchainos Plugin Store.

## Overview

[LayerBank](https://layerbank.finance) is an omni-chain over-collateralized lending protocol. This plugin supports the **Scroll** deployment (chain 534352).

## Supported Commands

| Command | Description |
|---------|-------------|
| `markets` | List all lToken markets with TVL, prices, and utilization |
| `positions` | View supplied/borrowed positions and health factor |
| `supply` | Supply assets to earn interest (mints lTokens) |
| `withdraw` | Redeem lTokens for underlying assets |
| `borrow` | Borrow against collateral (dry-run recommended) |
| `repay` | Repay a borrow position |

## Usage

```bash
# View all markets
layer-bank markets --chain 534352

# Check positions
layer-bank positions --chain 534352

# Supply 0.01 USDC (dry-run)
layer-bank supply --asset USDC --amount 0.01 --chain 534352 --dry-run

# Borrow (always dry-run first)
layer-bank borrow --asset ETH --amount 0.001 --chain 534352 --dry-run
```

## Chain Support

- **Scroll** (chain 534352) — primary deployment

## Contract Addresses (Scroll)

- Core: `0xEC53c830f4444a8A56455c6836b5D2aA794289Aa`
- PriceCalculator: `0xe3168c8D1Bcf6aaF5E090F61be619c060F3aD508`
