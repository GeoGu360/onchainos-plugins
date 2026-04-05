# Benqi Lending — Design Document

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | benqi |
| dapp_name | Benqi Lending |
| version | 0.1.0 |
| target_chains | Avalanche C-Chain (43114) |
| category | defi-protocol |
| tags | lending, borrowing, defi, benqi, avalanche, qitoken |

## §1 Feasibility Research

| Check | Result |
|-------|--------|
| Rust SDK? | No official Rust SDK |
| SDK tech stacks? | JavaScript SDK (benqi-js), no Rust |
| REST API? | https://api.benqi.fi (limited, mostly UI data) |
| Official Skill? | None |
| Community Skill? | None found |
| Supported chains? | Avalanche C-Chain (43114) only |
| Requires onchainos broadcast? | Yes — all write ops via onchainos wallet contract-call |

**Integration path:** Direct eth_call for reads (Compound V2 ABI); manual ABI encoding for writes via onchainos.

## §2 Interface Mapping

### Operations

| Operation | Type | Contract |
|-----------|------|---------|
| markets | off-chain read | qiToken eth_call |
| positions | off-chain read | qiToken + Comptroller eth_call |
| supply | on-chain write | qiToken.mint |
| redeem | on-chain write | qiToken.redeemUnderlying |
| borrow | dry-run only | qiToken.borrow |
| repay | dry-run only | qiToken.repayBorrow |
| claim-rewards | on-chain write | Comptroller.claimReward |

### Key Contracts

| Contract | Address |
|----------|---------|
| Comptroller | `0x486Af39519B4Dc9a7fCcd318217352830E8AD9b4` |
| qiAVAX | `0x5C0401e81Bc07Ca70fAD469b451682c0d747Ef1c` |
| qiUSDC | `0xBEb5d47A3f720Ec0a390d04b4d41ED7d9688bC7F` |
| qiUSDT | `0xc9e5999b8e75C3fEB117F6f73E664b9f3C8ca65C` |
| qiETH | `0x334AD834Cd4481BB02d09615E7c11a00579A7909` |
| qiBTC | `0xe194c4c5aC32a3C9ffDb358d9Bfd523a0B6d1568` |
| qiLINK | `0x4e9f683A27a6BdAD3FC2764003759277e93696e6` |
| qiDAI | `0x835866d37AFB8CB8F8334dCCdaf66cf01832Ff5D` |
| qiQI | `0x35Bd6aedA81a7E5FC7A7832490e71F757b0cD9Ce` |
| QI Token | `0x8729438EB15e2C8B576fCc6AeCdA6A148776C0F5` |

### On-Chain Write Operations

| Operation | Contract | Function Signature | Selector | ABI Params |
|-----------|---------|-------------------|---------|-----------|
| supply AVAX | qiAVAX | `mint()` | `0x1249c58b` | (none, send AVAX as value) |
| supply ERC20 approve | ERC20 | `approve(address,uint256)` | `0x095ea7b3` | spender=qiToken, amount |
| supply ERC20 mint | qiToken | `mint(uint256)` | `0xa0712d68` | mintAmount |
| redeem | qiToken | `redeemUnderlying(uint256)` | `0x852a12e3` | redeemAmount (underlying units) |
| borrow (dry-run) | qiToken | `borrow(uint256)` | `0xc5ebeaec` | borrowAmount |
| repay AVAX (dry-run) | qiAVAX | `repayBorrow()` | `0x4e4d9fea` | (none, send AVAX as value) |
| repay ERC20 approve (dry-run) | ERC20 | `approve(address,uint256)` | `0x095ea7b3` | spender=qiToken, amount |
| repay ERC20 (dry-run) | qiToken | `repayBorrow(uint256)` | `0x0e752702` | repayAmount |
| claim rewards | Comptroller | `claimReward(uint8,address)` | `0x0952c563` | rewardType (0=QI, 1=AVAX), holder |

### Key Difference from Compound V2

Benqi uses **per-timestamp** interest rate functions:
- `supplyRatePerTimestamp()` selector: `0xd3bd2c72`
- `borrowRatePerTimestamp()` selector: `0xcd91801c`
- APR = rate_per_second * 31,536,000 * 100

## §3 User Scenarios

### Scenario 1: Check Benqi markets
User: "Show me Benqi lending rates on Avalanche"
1. [off-chain] `benqi --chain 43114 markets` — eth_call to each qiToken for rates
2. Return JSON with supply APR, borrow APR, exchange rate per market

### Scenario 2: Supply USDC to earn interest
User: "Supply 10 USDC to Benqi on Avalanche"
1. [dry-run] `benqi --chain 43114 --dry-run supply --asset USDC --amount 10`
2. Show preview: approve calldata + mint calldata
3. Ask user to confirm
4. [on-chain] ERC20.approve(qiUSDC, 10e6) via onchainos wallet contract-call
5. [on-chain] qiUSDC.mint(10e6) via onchainos wallet contract-call
6. Report approveTxHash + mintTxHash + new qiUSDC balance

### Scenario 3: Check positions
User: "What are my positions on Benqi?"
1. [off-chain] `benqi --chain 43114 positions` — eth_call for balances + getAccountLiquidity
2. Return supplied amounts, borrowed amounts, account liquidity

### Scenario 4: Claim QI rewards
User: "Claim my QI rewards from Benqi"
1. [dry-run] `benqi --chain 43114 --dry-run claim-rewards --reward-type 0`
2. Ask user to confirm
3. [on-chain] Comptroller.claimReward(0, walletAddress) via onchainos wallet contract-call
4. Report txHash

## §4 External API Dependencies

| API | URL | Purpose |
|-----|-----|---------|
| Avalanche C-Chain RPC | https://avalanche-c-chain-rpc.publicnode.com | eth_call for reads and submitting txs |

## §5 Configuration Parameters

| Parameter | Default | Notes |
|-----------|---------|-------|
| chain_id | 43114 | Avalanche C-Chain only |
| dry_run | false | Preview mode, no broadcast |
| rpc_url | https://avalanche-c-chain-rpc.publicnode.com | Public node RPC |
