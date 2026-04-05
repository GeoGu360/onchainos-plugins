---
name: convex
description: "Convex Finance plugin for staking cvxCRV, locking CVX as vlCVX, and claiming boosted Curve rewards on Ethereum. Trigger phrases: stake cvxCRV, lock CVX, vlCVX, claim convex rewards, convex positions, convex pools, unstake cvxCRV, unlock CVX. Chinese: 质押cvxCRV, 锁定CVX, 领取Convex奖励, 查询Convex持仓"
license: MIT
metadata:
  author: GeoGu360
  version: "0.1.0"
---

## Architecture

Convex Finance allows CRV stakers and Curve LP providers to earn boosted rewards.
This plugin supports:
- Staking cvxCRV in CvxCrvStaking to earn CRV + CVX rewards
- Locking CVX as vlCVX (16-week lock) for governance and rewards
- Unlocking expired vlCVX locks
- Claiming all pending rewards
- Querying pools and positions

- Write ops (stake-cvxcrv, unstake-cvxcrv, lock-cvx, unlock-cvx, claim-rewards) -> after user confirmation, submits via `onchainos wallet contract-call`
- Read ops (get-pools, get-positions) -> direct eth_call via public RPC; no confirmation needed
- Chain: Ethereum mainnet (chain ID 1) only

## Execution Flow for Write Operations

1. Run with `--dry-run` first to preview calldata
2. **Ask user to confirm** before executing on-chain
3. Execute only after explicit user approval
4. Report transaction hash with Etherscan link

---

## Commands

### get-pools — List Convex Curve Pools

Lists Curve pools accessible through Convex Finance with APY data.

```
convex get-pools [--limit <n>] [--registry <main|factory|all>]
```

**Parameters:**
- `--limit` (optional, default 10): Number of pools to return
- `--registry` (optional, default all): Which Curve registry to query

**Example:** "Show me the top Convex pools"
```
convex get-pools --limit 10
```

**Output:**
```json
{
  "ok": true,
  "data": {
    "total_found": 450,
    "shown": 10,
    "pools": [
      {
        "name": "3pool",
        "address": "0xbebc44782c7db0a1a60cb6fe97d0b483032ff1c7",
        "coins": ["DAI", "USDC", "USDT"],
        "tvl_usd": "$500000000",
        "apy_pct": "3.5%",
        "registry": "main"
      }
    ]
  }
}
```

---

### get-positions — Query Convex Positions

Queries all Convex positions for a wallet.

```
convex get-positions [--address <wallet>] [--chain 1]
```

**Parameters:**
- `--address` (optional): Wallet to query (defaults to onchainos logged-in wallet)

**Example:** "What are my Convex positions?"
```
convex get-positions --chain 1
```

**Output:**
```json
{
  "ok": true,
  "data": {
    "wallet": "0x...",
    "positions": {
      "cvxCRV_staked": {
        "balance": "100.5",
        "pending_crv_rewards": "2.34"
      },
      "vlCVX_locked": {
        "balance": "500",
        "note": "16-week lock period"
      }
    },
    "liquid_balances": {
      "CVX": "10.0",
      "cvxCRV": "5.0",
      "CRV": "0.0"
    }
  }
}
```

---

### stake-cvxcrv — Stake cvxCRV

Stakes cvxCRV tokens to earn boosted CRV and CVX rewards.

**Ask user to confirm** before submitting the approve and/or stake transactions.

```
convex stake-cvxcrv --amount <amount> [--from <wallet>] [--chain 1] [--dry-run]
```

**Parameters:**
- `--amount` (required): Amount of cvxCRV to stake (e.g., 10.5)
- `--from` (optional): Wallet address override
- `--dry-run` (optional): Preview calldata without broadcasting

**Execution steps:**
1. Check cvxCRV balance
2. If needed: approve cvxCRV spending via `onchainos wallet contract-call` -> **ask user to confirm**
3. Stake via `onchainos wallet contract-call` -> **ask user to confirm**

**Example:** "Stake 10 cvxCRV on Convex"
```
convex stake-cvxcrv --amount 10 --chain 1
```

---

### unstake-cvxcrv — Unstake cvxCRV

Withdraws staked cvxCRV from the CvxCrvStaking contract.

**Ask user to confirm** before submitting the transaction.

```
convex unstake-cvxcrv --amount <amount> [--to <recipient>] [--claim] [--from <wallet>] [--chain 1] [--dry-run]
```

**Parameters:**
- `--amount` (required): Amount of cvxCRV to unstake
- `--to` (optional): Recipient address (defaults to calling wallet)
- `--claim` (optional): Also claim pending rewards
- `--dry-run` (optional): Preview calldata

**Example:** "Unstake 5 cvxCRV from Convex"
```
convex unstake-cvxcrv --amount 5 --chain 1
```

---

### lock-cvx — Lock CVX as vlCVX

Locks CVX tokens as vote-locked CVX (vlCVX) for 16 weeks to earn rewards and participate in governance.

**Warning: CVX is locked for 16 weeks and cannot be withdrawn early.**
**Ask user to confirm** before submitting.

```
convex lock-cvx --amount <amount> [--from <wallet>] [--chain 1] [--dry-run]
```

**Parameters:**
- `--amount` (required): Amount of CVX to lock (e.g., 100)
- `--from` (optional): Wallet address override
- `--dry-run` (optional): Preview calldata

**Execution steps:**
1. Check CVX balance
2. If needed: approve CVX spending -> `onchainos wallet contract-call` -> **ask user to confirm**
3. Lock CVX -> `onchainos wallet contract-call` -> **ask user to confirm**

**Example:** "Lock 50 CVX as vlCVX"
```
convex lock-cvx --amount 50 --chain 1
```

---

### unlock-cvx — Unlock Expired vlCVX

Processes expired vlCVX locks to receive CVX back (or re-lock).

**Ask user to confirm** before submitting.

```
convex unlock-cvx [--relock] [--from <wallet>] [--chain 1] [--dry-run]
```

**Parameters:**
- `--relock` (optional): Re-lock the CVX instead of withdrawing
- `--dry-run` (optional): Preview calldata

**Note:** This will revert if there are no expired locks.

**Example:** "Unlock my expired vlCVX"
```
convex unlock-cvx --chain 1
```

---

### claim-rewards — Claim Convex Rewards

Claims pending rewards from cvxCRV staking and/or vlCVX.

**Ask user to confirm** before submitting.

```
convex claim-rewards [--cvxcrv] [--vlcvx] [--from <wallet>] [--chain 1] [--dry-run]
```

**Parameters:**
- `--cvxcrv` (optional, default true): Claim from cvxCRV staking
- `--vlcvx` (optional, default true): Claim from vlCVX
- `--dry-run` (optional): Preview calldata

**Example:** "Claim all my Convex rewards"
```
convex claim-rewards --chain 1
```

**Output:**
```json
{
  "ok": true,
  "data": {
    "action": "claim-rewards",
    "claims": [
      {
        "source": "cvxCRV_staking",
        "pending_crv": "2.34",
        "txHash": "0x..."
      }
    ]
  }
}
```
