# etherfi-liquid

ether.fi Liquid multi-strategy yield vaults plugin for onchainos.

## Supported Vaults

| Vault | Symbol | Deposit Token | APY (approx) |
|-------|--------|--------------|--------------|
| ETH Yield Vault | LIQUIDETH | weETH / WETH | ~3% |
| USD Yield Vault | LIQUIDUSD | USDC / USDT | ~4% |
| BTC Yield Vault | LIQUIDBTC | WBTC / eBTC | ~2% |

## Commands

- `vaults` — list available vaults with APY and TVL
- `positions` — show your current share balances
- `rates` — show current share prices
- `deposit --vault LIQUIDETH --token weETH --amount 0.001` — deposit into a vault
- `withdraw --vault LIQUIDETH --shares 0.001` or `--all` — withdraw from a vault

## Chain

Ethereum mainnet (chain ID 1) only.

## Build

```bash
cargo build --release
```
