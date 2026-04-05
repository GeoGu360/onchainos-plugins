# INIT Capital Plugin

INIT Capital is a non-custodial decentralized lending protocol with multi-silo isolated positions on Blast.

## Supported Operations

- `pools` — List lending pools with rates and TVL
- `positions` — View your isolated positions with health factor
- `health-factor` — Check health factor for a specific position
- `supply` — Supply assets to earn interest
- `withdraw` — Withdraw collateral
- `borrow` — Borrow against collateral (dry-run recommended)
- `repay` — Repay debt (dry-run recommended)

## Supported Chains

| Chain | Chain ID |
|-------|----------|
| Blast | 81457 |

## Key Contracts (Blast)

| Contract | Address |
|----------|---------|
| INIT_CORE | `0xa7d36f2106b5a5D528a7e2e7a3f436d703113A10` |
| MONEY_MARKET_HOOK | `0xC02819a157320Ba2859951A1dfc1a5E76c424dD4` |
| POS_MANAGER | `0xA0e172f8BdC18854903959b8f7f73F0D332633fe` |

## Usage

```bash
# List pools
init-capital pools --chain 81457

# Check positions
init-capital positions --chain 81457

# Supply WETH
init-capital supply --asset WETH --amount 0.01 --chain 81457 --dry-run

# Borrow USDB
init-capital borrow --asset USDB --amount 1.0 --pos-id 1 --chain 81457 --dry-run
```
