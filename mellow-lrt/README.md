# mellow-lrt

Mellow LRT plugin for onchainos — deposit ETH or wstETH into liquid restaking vaults and earn EigenLayer/Symbiotic yield.

## Features

- **vaults** — List all Mellow LRT vaults with APR, TVL, and deposit options
- **positions** — Check your LRT holdings and withdrawal status
- **deposit** — Deposit ETH/wstETH/WETH/stETH into any vault
- **withdraw** — Initiate async withdrawal (2-step process)
- **claim** — Collect processed withdrawals after the queue period

## Supported Chain

Ethereum mainnet (chain ID 1)

## Usage

```bash
# List top vaults by TVL
mellow-lrt --chain 1 vaults

# Check positions
mellow-lrt --chain 1 positions

# Deposit 0.00005 ETH into steakLRT
mellow-lrt --chain 1 deposit --vault steakLRT --token ETH --amount 0.00005

# Preview withdraw (dry run)
mellow-lrt --chain 1 withdraw --vault steakLRT --all --dry-run

# Claim unlocked withdrawal
mellow-lrt --chain 1 claim --vault steakLRT
```

## Withdrawal Flow

Mellow withdrawals are asynchronous (Symbiotic protocol requirement):
1. `withdraw` → redeems shares, enters queue
2. Wait ~14 days for Symbiotic epoch processing
3. `claim` → receive wstETH

## License

MIT
