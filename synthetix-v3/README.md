# Synthetix V3 Plugin

Synthetix V3 integration for the OKX Plugin Store. Supports Perps market queries and sUSDC collateral management on Base (chain 8453).

## Commands

| Command | Description |
|---------|-------------|
| `markets` | List Perps markets with funding rates |
| `positions` | Query open positions for an account |
| `collateral` | Query collateral balances |
| `deposit-collateral` | Deposit sUSDC as collateral |
| `withdraw-collateral` | Withdraw available sUSDC collateral |

## Usage

```bash
# List all perps markets
synthetix-v3 markets

# Query a specific market
synthetix-v3 markets --market-id 100

# Check positions for account
synthetix-v3 positions --account-id 1234567890

# Check collateral balances
synthetix-v3 collateral --account-id 1234567890

# Deposit sUSDC (dry-run first)
synthetix-v3 --dry-run deposit-collateral --account-id 1234567890 --amount 10.0

# Withdraw sUSDC
synthetix-v3 withdraw-collateral --account-id 1234567890 --amount 5.0
```

## Contracts (Base 8453, Andromeda)

- CoreProxy: `0x32C222A9A159782aFD7529c87FA34b96CA72C696`
- PerpsMarketProxy: `0x0A2AF931eFFd34b81ebcc57E3d3c9B1E1dE1C9Ce`
- sUSDC: `0xC74eA762cF06c9151cE074E6a569a5945b6302E7`
