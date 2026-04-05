# Benqi Lending Plugin

Benqi Lending plugin for the OKX Plugin Store. Benqi is a Compound V2 fork on Avalanche C-Chain.

## Supported Operations

- **markets** - List qiToken markets with supply/borrow APR
- **positions** - View your supplied and borrowed positions
- **supply** - Supply assets to earn interest (mints qiTokens)
- **redeem** - Redeem qiTokens to get back underlying assets
- **borrow** - Preview borrowing (dry-run only)
- **repay** - Preview repaying (dry-run only)
- **claim-rewards** - Claim QI token or AVAX rewards

## Supported Chain

- Avalanche C-Chain (chain ID: 43114)

## Supported Assets

AVAX, USDC, USDT, ETH, BTC, LINK, DAI, QI

## Usage

```bash
benqi markets
benqi positions --wallet 0x...
benqi --dry-run supply --asset USDC --amount 0.01
benqi supply --asset USDC --amount 0.01
benqi redeem --asset USDC --amount 0.01
benqi claim-rewards --reward-type 0
```
