---
name: notional-v3
description: "Notional Finance leveraged yield (Exponent) on Ethereum mainnet. Trigger phrases: notional vaults, notional positions, enter notional vault, exit notional vault, notional leveraged yield, claim notional rewards, initiate notional withdraw, notional fixed rate yield"
license: MIT
metadata:
  author: GeoGu360
  version: "0.1.0"
do_not_use_for: "general DeFi queries, Notional V1/V2 legacy, non-Ethereum chains, token swaps, liquidity provision, staking on Aave/Compound/Morpho directly"

---

# Notional V3 Skill (Notional Exponent)

## Protocol Status

Notional V3 legacy contracts are fully paused on-chain. This plugin targets **Notional Exponent** (V4), the active successor protocol, deployed on **Ethereum mainnet (chain 1) only**.

- **MorphoLendingRouter**: `0x9a0c630C310030C4602d1A76583a3b16972ecAa0`
- **Architecture**: Leveraged yield vaults backed by Morpho protocol
- **TVL**: ~$3.3M (Ethereum mainnet)

---

## Commands

### Read Commands (safe, no wallet needed)

#### `get-vaults`
List available leveraged yield vaults on Notional Exponent.

```
notional-v3 get-vaults
notional-v3 get-vaults --asset USDC
notional-v3 get-vaults --asset WETH
```

#### `get-positions`
View current vault positions for a wallet.

```
notional-v3 get-positions
notional-v3 get-positions --wallet 0xYourAddress
```

Returns: token type, vault address, current balance, health factor (for leveraged positions), PnL.

---

### Write Commands (require wallet confirmation)

> **IMPORTANT**: Before executing any transaction, always ask the user to confirm
> the transaction details — vault address, amount, and chain. These operations move real funds.

#### `enter-position`
Deposit into a leveraged yield vault (optionally with borrowed leverage).

```
notional-v3 enter-position --vault 0xVaultAddress --amount 0.01 --asset USDC
notional-v3 enter-position --vault 0xVaultAddress --amount 0.01 --asset USDC --borrow-amount 0
notional-v3 enter-position --vault 0xVaultAddress --amount 0.01 --dry-run
```

**Steps**: (1) ERC-20 approve MorphoLendingRouter → (2) `enterPosition()` (3s delay between steps)

**Default**: `--borrow-amount 0` (no leverage). Leverage is dry-run only per guardrails.

#### `exit-position`
Redeem vault shares to withdraw assets.

```
notional-v3 exit-position --vault 0xVaultAddress --shares all
notional-v3 exit-position --vault 0xVaultAddress --shares 1000000000000000000
notional-v3 exit-position --vault 0xVaultAddress --shares all --dry-run
```

Use `--shares all` to exit the full position. Always confirm with the user before executing.

#### `initiate-withdraw`
For staking strategies (e.g. sUSDe vaults): starts the unstaking queue. Assets become claimable after the unstaking period.

```
notional-v3 initiate-withdraw --vault 0xVaultAddress --shares all
notional-v3 initiate-withdraw --vault 0xVaultAddress --shares 1000000000000000000
notional-v3 initiate-withdraw --vault 0xVaultAddress --shares all --dry-run
```

Always confirm with the user before executing. This starts an irreversible unbonding period.

#### `claim-rewards`
Claim pending rewards from a vault.

```
notional-v3 claim-rewards --vault 0xVaultAddress
notional-v3 claim-rewards --vault 0xVaultAddress --wallet 0xYourAddress
notional-v3 claim-rewards --vault 0xVaultAddress --dry-run
```

Always confirm with the user before executing.

---

## Known Vault Addresses (Ethereum mainnet)

| Vault | Address | Asset |
|---|---|---|
| sUSDe Staking | `0xaf14d06a65c91541a5b2db627ecd1c92d7d9c48b` | USDC |
| mAPOLLO Leveraged | `0x091356e6793a0d960174eaab4d470e39a99dd673` | USDC |
| mHYPER Leveraged | `0x2a5c94fe8fa6c0c8d2a87e5c71ad628caa092ce4` | USDC |
| weETH Leveraged | `0x7f723fee1e65a7d26be51a05af0b5efee4a7d5ae` | WETH |
| Pendle PT-sUSDE | `0x0e61e810f0918081cbfd2ac8c97e5866daf3f622` | USDC |
| liUSD-4w Leveraged | `0x9fb57943926749b49a644f237a28b491c9b465e0` | USDC |
| Convex OETH/WETH | `0x2716561755154eef59bc48eb13712510b27f167f` | WETH |
| mHYPER Leveraged 2 | `0x94f6cb4fae0eb3fa74e9847dff2ff52fd5ec7e6e` | USDC |

---

## Notes

- Only Ethereum mainnet (chain 1) is supported
- `--borrow-amount` > 0 introduces liquidation risk — use dry-run only
- Health factor < 1.0 triggers liquidation — monitor positions regularly
- `initiate-withdraw` starts an unstaking queue; final withdrawal requires a separate step after the unbonding period
- Subgraph: `https://api.studio.thegraph.com/query/60626/notional-exponent/version/latest`
