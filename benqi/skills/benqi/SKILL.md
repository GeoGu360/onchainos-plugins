---
name: benqi
description: "Benqi Lending on Avalanche C-Chain: supply assets to earn interest, redeem qiTokens, view positions, borrow (dry-run), repay (dry-run), claim QI and AVAX rewards. Supports AVAX, USDC, USDT, ETH, BTC, LINK, DAI. Trigger phrases: benqi supply, benqi lend, benqi redeem, benqi borrow, benqi repay, benqi positions, benqi markets, claim QI rewards, qiToken, supply AVAX, lend on Avalanche, Benqi lending"
license: MIT
metadata:
  author: GeoGu360
  version: "0.1.0"
---

## Overview

Benqi Lending is a Compound V2 fork on Avalanche C-Chain. Supply assets to earn interest (and receive qiTokens), use supplied assets as collateral to borrow, or claim QI and AVAX rewards.

- Read ops (`markets`, `positions`) use direct RPC calls; no wallet needed
- Write ops (`supply`, `redeem`, `claim-rewards`) require user confirmation and submit via `onchainos wallet contract-call`
- `borrow` and `repay` are always dry-run only for safety

## Architecture

| Operation | Type | Contract |
|-----------|------|---------|
| markets | read | qiToken contracts (eth_call) |
| positions | read | qiToken + Comptroller (eth_call) |
| supply AVAX | write | qiAVAX.mint() payable |
| supply ERC20 | write | ERC20.approve + qiToken.mint(uint256) |
| redeem | write | qiToken.redeemUnderlying(uint256) |
| borrow | dry-run | qiToken.borrow(uint256) |
| repay | dry-run | ERC20.approve + qiToken.repayBorrow(uint256) |
| claim-rewards | write | Comptroller.claimReward(uint8, address) |

## Supported Chain

| Chain | Chain ID | Protocol |
|-------|----------|---------|
| Avalanche C-Chain | 43114 | Benqi Lending (Compound V2 fork) |

## Supported Assets

| Symbol | qiToken Address | Underlying |
|--------|----------------|-----------|
| AVAX | `0x5C0401e81Bc07Ca70fAD469b451682c0d747Ef1c` | Native AVAX |
| USDC | `0xBEb5d47A3f720Ec0a390d04b4d41ED7d9688bC7F` | `0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E` |
| USDT | `0xc9e5999b8e75C3fEB117F6f73E664b9f3C8ca65C` | `0x9702230A8Ea53601f5cD2dc00fDBc13d4dF4A8c7` |
| ETH | `0x334AD834Cd4481BB02d09615E7c11a00579A7909` | `0x49D5c2BdFfac6CE2BFdB6640F4F80f226bc10bAB` |
| BTC | `0xe194c4c5aC32a3C9ffDb358d9Bfd523a0B6d1568` | `0x50b7545627a5162F82A992c33b87aDc75187B218` |
| LINK | `0x4e9f683A27a6BdAD3FC2764003759277e93696e6` | `0x5947BB275c521040051D82396192181b413227A3` |
| DAI | `0x835866d37AFB8CB8F8334dCCdaf66cf01832Ff5D` | `0xd586E7F844cEa2F87f50152665BCbc2C279D8d70` |
| QI | `0x35Bd6aedA81a7E5FC7A7832490e71F757b0cD9Ce` | `0x8729438EB15e2C8B576fCc6AeCdA6A148776C0F5` |

## Commands

### markets - List qiToken markets

```bash
benqi [--chain 43114] markets
```

Returns supply APR, borrow APR, and exchange rate for each qiToken market. Read-only; no wallet needed.

---

### positions - View your positions

```bash
benqi [--chain 43114] positions [--wallet 0x...]
```

Returns supplied balance (in underlying) and borrowed amount per market, plus account liquidity from the Comptroller. Read-only.

---

### supply - Supply an asset to earn interest

```bash
# Preview (dry-run)
benqi --chain 43114 --dry-run supply --asset USDC --amount 0.01

# Execute
benqi --chain 43114 supply --asset USDC --amount 0.01
```

**Execution flow:**
1. Run with `--dry-run` to preview the calldata and steps
2. **Ask user to confirm** the asset, amount, and that they will receive qiTokens in return
3. For ERC20 assets: execute `ERC20.approve(qiToken, amount)`, wait 3 seconds, then `qiToken.mint(amount)`
4. For AVAX: execute `qiAVAX.mint()` as a payable call with AVAX value
5. Report approve txHash (ERC20 only), mint txHash, and updated qiToken balance

After confirming with the user, submit via `onchainos wallet contract-call --chain 43114`.

---

### redeem - Redeem qiTokens to get back underlying

```bash
# Preview (dry-run)
benqi --chain 43114 --dry-run redeem --asset USDC --amount 0.01

# Execute
benqi --chain 43114 redeem --asset USDC --amount 0.01
```

**Execution flow:**
1. Run with `--dry-run` to preview
2. **Ask user to confirm** the amount of underlying to receive
3. Execute `qiToken.redeemUnderlying(amount)`
4. Report txHash

After confirming with the user, submit via `onchainos wallet contract-call --chain 43114`.

---

### borrow - Preview borrowing (DRY-RUN ONLY)

```bash
benqi --chain 43114 borrow --asset USDC --amount 1.0
```

**Note:** Borrow is dry-run only for safety. Shows the calldata and steps. Requires collateral to be supplied first. Never executes on-chain.

---

### repay - Preview repaying borrow (DRY-RUN ONLY)

```bash
benqi --chain 43114 repay --asset USDC --amount 1.0
```

**Note:** Repay is dry-run only for safety. Shows approve + repayBorrow steps. Never executes on-chain.

---

### claim-rewards - Claim QI or AVAX rewards

```bash
# Preview (dry-run)
benqi --chain 43114 --dry-run claim-rewards [--reward-type 0]

# Claim QI rewards (reward-type 0)
benqi --chain 43114 claim-rewards --reward-type 0

# Claim AVAX rewards (reward-type 1)
benqi --chain 43114 claim-rewards --reward-type 1
```

Reward types: `0` = QI governance token, `1` = native AVAX.

**Execution flow:**
1. Run with `--dry-run` to preview
2. **Ask user to confirm** which reward type to claim
3. Execute `Comptroller.claimReward(rewardType, walletAddress)`
4. Report txHash

After confirming with the user, submit via `onchainos wallet contract-call --chain 43114`.

---

## Key Concepts

**qiTokens represent your supply position**
When you supply assets, you receive qiTokens. The exchange rate increases over time as interest accrues. To get assets back, redeem qiTokens via `redeem`.

**Timestamp-based interest rates**
Unlike Compound V2 (which uses per-block rates), Benqi uses per-timestamp rates (`supplyRatePerTimestamp`, `borrowRatePerTimestamp`). APR = rate_per_second * 31,536,000 * 100%.

**Borrow requires collateral**
Supply collateral first, then borrow. Your total borrow must not exceed your borrowing capacity (based on collateral factor). Monitor health factor via `positions`.

**Dual rewards**
Benqi distributes both QI governance tokens (type 0) and native AVAX (type 1) as liquidity mining rewards.

## Dry-Run Mode

All write operations support `--dry-run`. In dry-run mode:
- No transactions are broadcast
- Returns expected calldata, steps, and amounts as JSON
- Use to preview before asking for user confirmation

## Error Responses

All commands return structured JSON:
```json
{"ok": false, "error": "human-readable error message"}
```
