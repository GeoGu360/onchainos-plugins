# Velodrome V2 Plugin — Design Document

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `velodrome-v2` |
| dapp_name | Velodrome V2 |
| version | 0.1.0 |
| target_chains | Optimism (chain ID 10) |
| category | defi-protocol |
| tags | dex, amm, velodrome, classic-amm, stable, volatile, optimism |

---

## §1 Feasibility Research

| Check | Result |
|-------|--------|
| Rust SDK? | No official Rust SDK |
| SDK tech stack | N/A |
| REST API? | No public REST API — direct on-chain via eth_call |
| Official Skill? | None |
| Community Skill? | Aerodrome AMM plugin (Base) — same architecture, adapt for Optimism |
| Supported chains | Optimism mainnet (chain ID 10) |
| Needs onchainos broadcast? | Yes — swaps and liquidity ops require wallet tx |

**Onboarding path**: Reference `aerodrome-amm` plugin (same codebase, different chain + contracts)

---

## §2 Interface Mapping

### Operations

| Operation | Type | Description |
|-----------|------|-------------|
| quote | Read (eth_call) | Get swap quote from Router.getAmountsOut |
| swap | Write (on-chain) | Execute swap via Router.swapExactTokensForTokens |
| pools | Read (eth_call) | Get pool info from Factory.getPool + pool reserves |
| positions | Read (eth_call) | View LP token balances for wallet |
| add-liquidity | Write (on-chain) | Add liquidity via Router.addLiquidity |
| remove-liquidity | Write (on-chain) | Remove liquidity via Router.removeLiquidity |
| claim-rewards | Write (on-chain) | Claim VELO emissions from gauge |

### Read Operations (eth_call)

| Operation | Contract | Function | Selector | Parameters |
|-----------|----------|----------|----------|------------|
| getPool | Factory `0xF1046053aa5682b4F9a81b5481394DA16BE5FF5a` | `getPool(address,address,bool)` | `0x79bc57d5` ✅ | tokenA, tokenB, stable |
| getAmountsOut | Router `0xa062aE8A9c5e11aaA026fc2670B0D65cCc8B2858` | `getAmountsOut(uint256,(address,address,bool,address)[])` | `0x5509a1ac` ✅ | amountIn, routes[] |
| quoteAddLiquidity | Router | `quoteAddLiquidity(address,address,bool,address,uint256,uint256)` | `0xce700c29` ✅ | tokenA, tokenB, stable, factory, amountADesired, amountBDesired |
| getReserves | Pool contract | `getReserves()` | `0x0902f1ac` ✅ | — |
| balanceOf | LP token / Pool | `balanceOf(address)` | `0x70a08231` ✅ | account |
| allowance | ERC-20 token | `allowance(address,address)` | `0xdd62ed3e` ✅ | owner, spender |
| earned | Gauge | `earned(address)` | `0x008cc262` ✅ | account |
| gauges | Voter `0x41C914ee0c7E1A5edCD0295623e6dC557B5aBf3C` | `gauges(address)` | `0xb9a09fd5` ✅ | pool |

### Write Operations (on-chain via onchainos)

| Operation | Contract | Function Signature | Selector | ABI Param Order |
|-----------|----------|--------------------|----------|-----------------|
| swap | Router `0xa062aE8A9c5e11aaA026fc2670B0D65cCc8B2858` | `swapExactTokensForTokens(uint256,uint256,(address,address,bool,address)[],address,uint256)` | `0xcac88ea9` ✅ | amountIn, amountOutMin, routes[], to, deadline |
| add-liquidity | Router | `addLiquidity(address,address,bool,uint256,uint256,uint256,uint256,address,uint256)` | `0x5a47ddc3` ✅ | tokenA, tokenB, stable, amountADesired, amountBDesired, amountAMin, amountBMin, to, deadline |
| remove-liquidity | Router | `removeLiquidity(address,address,bool,uint256,uint256,uint256,address,uint256)` | `0x0dede6c4` ✅ | tokenA, tokenB, stable, liquidity, amountAMin, amountBMin, to, deadline |
| claim-rewards | Gauge (dynamic) | `getReward(address)` | `0xc00007b0` ✅ | account |
| approve | ERC-20 token | `approve(address,uint256)` | `0x095ea7b3` ✅ | spender, amount |

All write ops use: `onchainos wallet contract-call --chain 10 --to <addr> --input-data <hex> --force`

---

## §3 User Scenarios

### Scenario 1: Query Swap Quote
User: "How much USDC will I get for swapping 0.00005 ETH on Velodrome?"
1. Resolve WETH address → `0x4200000000000000000000000000000000000006`
2. Resolve USDC address → `0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85`
3. eth_call `getPool(WETH, USDC, false)` on Factory → pool address
4. eth_call `getAmountsOut(50000000000000, [Route{WETH, USDC, false, factory}])` → amountOut
5. Return formatted quote

### Scenario 2: Swap Tokens
User: "Swap 0.00005 WETH for USDC on Velodrome V2"
1. Quote via getAmountsOut
2. Ask user to confirm amounts and slippage
3. Check WETH allowance → approve Router if needed
4. Build swap calldata (selector 0xcac88ea9)
5. `onchainos wallet contract-call --chain 10 --to Router --input-data <calldata> --force`
6. Return txHash

### Scenario 3: View LP Positions
User: "Show my Velodrome LP positions"
1. Resolve wallet via `onchainos wallet balance --chain 10`
2. For common pairs: check Factory.getPool() → check balanceOf(wallet)
3. For each pool with balance: fetch reserves, compute share %, estimate token amounts
4. Return list of positions

### Scenario 4: Add Liquidity
User: "Add 0.00005 WETH and USDC liquidity to Velodrome volatile pool"
1. Verify pool exists via Factory.getPool()
2. Auto-quote amountBDesired via quoteAddLiquidity if not provided
3. Ask user to confirm token amounts
4. Approve tokenA → Router (if needed)
5. Approve tokenB → Router (if needed)
6. Build addLiquidity calldata (selector 0x5a47ddc3)
7. `onchainos wallet contract-call --chain 10 --to Router --input-data <calldata> --force`

### Scenario 5: Remove Liquidity
User: "Remove my WETH/USDC Velodrome LP position"
1. Lookup pool address from Factory
2. Check LP token balance for wallet
3. Ask user to confirm LP amount
4. Approve LP token → Router
5. Build removeLiquidity calldata (selector 0x0dede6c4)
6. `onchainos wallet contract-call --chain 10 --to Router --input-data <calldata> --force`

---

## §4 External API Dependencies

| API | URL | Purpose |
|-----|-----|---------|
| Optimism RPC | `https://optimism-rpc.publicnode.com` | eth_call for all read ops |

---

## §5 Config Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| chain_id | 10 | Optimism mainnet |
| rpc_url | `https://optimism-rpc.publicnode.com` | JSON-RPC endpoint |
| router | `0xa062aE8A9c5e11aaA026fc2670B0D65cCc8B2858` | Classic AMM Router |
| factory | `0xF1046053aa5682b4F9a81b5481394DA16BE5FF5a` | Pool Factory |
| voter | `0x41C914ee0c7E1A5edCD0295623e6dC557B5aBf3C` | Voter (gauges) |
| dry_run | false | Skip on-chain broadcast, return simulated response |
| slippage | 0.5% | Default slippage tolerance |

### Token Addresses (Optimism mainnet)

| Symbol | Address |
|--------|---------|
| WETH / ETH | `0x4200000000000000000000000000000000000006` |
| USDC | `0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85` |
| USDT | `0x94b008aA00579c1307B0EF2c499aD98a8ce58e58` |
| DAI | `0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1` |
| VELO | `0x9560e827aF36c94D2Ac33a39bCE1Fe78631088Db` |
| WBTC | `0x68f180fcCe6836688e9084f035309E29Bf0A2095` |
| OP | `0x4200000000000000000000000000000000000042` |
| WSTETH | `0x1F32b1c2345538c0c6f582fCB022739c4A194Ebb` |
| SNX | `0x8700dAec35aF8Ff88c16BdF0418774CB3D7599B4` |
