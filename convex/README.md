# Convex Finance Plugin

Convex Finance integration for onchainos. Allows you to:
- Stake cvxCRV for boosted CRV + CVX rewards
- Lock CVX as vlCVX (vote-locked, 16-week period)
- Unlock expired vlCVX positions
- Claim all pending rewards
- Query Curve pools and your positions

## Supported Chain

- Ethereum mainnet (chain ID 1)

## Usage

```bash
# List top Convex pools
convex get-pools --limit 10

# Check your positions
convex get-positions --chain 1

# Stake cvxCRV
convex stake-cvxcrv --amount 10 --chain 1 --dry-run
convex stake-cvxcrv --amount 10 --chain 1

# Lock CVX
convex lock-cvx --amount 50 --chain 1 --dry-run
convex lock-cvx --amount 50 --chain 1

# Claim rewards
convex claim-rewards --chain 1
```

## Contracts

| Contract | Address |
|----------|---------|
| Booster | 0xF403C135812408BFbE8713b5A23a04b3D48AAE31 |
| CvxCrvStaking | 0x3Fe65692bfCD0e6CF84cB1E7d24108E434A7587e |
| vlCVX | 0x72a19342e8F1838460eBFCCEf09F6585e32db86E |
| CVX Token | 0x4e3FBD56CD56c3e72c1403e103b45Db9da5B9D2B |
| cvxCRV Token | 0x62b9c7356a2dc64a1969e19c23e4f579f9810aa7 |
