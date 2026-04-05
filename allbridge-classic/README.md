# Allbridge Classic Plugin

Cross-chain stablecoin bridge plugin for onchainos. Bridge USDT, USDC, BUSD between Ethereum, BSC, Polygon, Avalanche, and Solana using the Allbridge Classic protocol.

## Commands

- `get-tokens` — List supported tokens and bridge fees per chain
- `bridge` — Bridge ERC-20 tokens cross-chain (EVM source)
- `get-tx-status` — Check bridge transaction status by lock ID
- `check-address` — Validate recipient address for destination chain

## Supported Chains

| Chain | ID |
|---|---|
| Ethereum | 1 |
| BSC | 56 |
| Polygon | 137 |
| Avalanche | 43114 |
| Solana | destination only |

## Usage

```bash
# List available tokens
allbridge-classic get-tokens

# Bridge 10 USDT from Ethereum to Solana (dry run)
allbridge-classic bridge --chain 1 --token USDT --amount 10.0 \
  --dest-chain SOL --recipient <SOLANA_ADDRESS> --dry-run

# Check bridge status
allbridge-classic get-tx-status --lock-id <LOCK_ID>
```

## Note

Allbridge Classic is in maintenance mode and scheduled to be discontinued in mid-2026. For new integrations, consider using Allbridge Core.
