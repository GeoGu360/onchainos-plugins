# sanctum-infinity

Sanctum Infinity LST pool plugin for onchainos. Swap between Solana LSTs, deposit to earn staking yields + trading fees, and query pool stats.

## Commands

- `pools` — Pool stats, APY, LST allocations
- `quote` — Get swap quote
- `swap` — Execute LST→LST swap
- `deposit` — Add liquidity (earn INF)
- `withdraw` — Remove liquidity (burn INF)
- `positions` — View INF holdings

## Usage

```bash
sanctum-infinity pools
sanctum-infinity quote --from jitoSOL --to INF --amount 0.005
sanctum-infinity swap --from jitoSOL --to INF --amount 0.001 --dry-run
sanctum-infinity deposit --lst jitoSOL --amount 0.005 --dry-run
sanctum-infinity withdraw --lst jitoSOL --amount 0.001 --dry-run
sanctum-infinity positions
```
