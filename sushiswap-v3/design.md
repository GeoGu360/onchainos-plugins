# SushiSwap V3 Plugin Design

## §0 Overview

**Plugin Name**: sushiswap-v3  
**Type**: Skill + Binary (Rust)  
**Protocol**: SushiSwap V3 — Concentrated Liquidity Market Maker (CLMM), Uniswap V3-style  
**Primary Chain**: Base (8453)  
**Supported Chains**: EVM 38+ chains (Ethereum 1, Base 8453, Arbitrum 42161, BSC 56, Polygon 137, Optimism 10, Avalanche 43114, etc.)  
**SDK**: REST API approach (no official Rust SDK; uses `https://api.sushi.com/swap/v7/{chainId}` for routing + direct contract calls for write ops)

---

## §1 Contract Addresses (Base Chain 8453)

| Contract | Address |
|---|---|
| Factory (UniswapV3Factory) | `0xc35DADB65012eC5796536bD9864eD8773aBc74C4` |
| SwapRouter | `0xFB7eF66a7e61224DD6FcD0D7d9C3be5C8B049b9f` |
| QuoterV2 | `0xb1E835Dc2785b52265711e17fCCb0fd018226a6e` |
| NonfungiblePositionManager (NFT) | `0x80C7DD17B01855a6D2347444a0FCC36136a314de` |
| RouteProcessor4 (used by Swap API) | `0xac4c6e212A361c968F1725b4d055b47E63F80b75` |

**Key Tokens on Base**:
- WETH: `0x4200000000000000000000000000000000000006`
- USDC: `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913`

**Fee Tiers**: 100 (0.01%), 500 (0.05%), 3000 (0.3%), 10000 (1%)

**Source**: `https://github.com/sushiswap/v3-periphery/tree/master/deployments/base`

---

## §2 REST API Endpoints

### Swap/Quote API
- **Base URL**: `https://api.sushi.com/swap/v7/{chainId}`
- **Method**: GET
- **Query Parameters**:
  - `tokenIn`: input token address
  - `tokenOut`: output token address
  - `amount`: amount in base units (wei/smallest unit)
  - `maxSlippage`: e.g. `0.005` for 0.5%
  - `sender`: user wallet address
- **Response**: JSON with `status`, `amountIn`, `assumedAmountOut`, `tx` (from, to, data, value, gasPrice)
- **Example**: `https://api.sushi.com/swap/v7/8453?tokenIn=0x4200...&tokenOut=0x8335...&amount=1000000000000000&maxSlippage=0.005&sender=0x87fb...`

### Pool Info
- No stable public REST API for pools; use on-chain contract calls to Factory + QuoterV2
- Optionally query The Graph decentralized network for SushiSwap V3 Base subgraph

---

## §3 Supported Operations

### Read Operations (no wallet required)
1. **quote** — Get swap quote via Sushi API (`api.sushi.com/swap/v7/{chainId}`)
2. **pools** — List top pools (from QuoterV2 + known pool addresses or subgraph)
3. **positions** — List user LP positions via NonfungiblePositionManager (tokenOfOwnerByIndex + positions)

### Write Operations (require wallet)
4. **swap** — Execute swap via SwapRouter `exactInputSingle` (with ERC20 approve if needed)
5. **add-liquidity** — Add concentrated liquidity via NonfungiblePositionManager `mint`
6. **remove-liquidity** — Remove liquidity via NonfungiblePositionManager `decreaseLiquidity` + `collect`
7. **collect-fees** — Collect accumulated fees via NonfungiblePositionManager `collect`

---

## §4 Function Selectors (Verified)

All selectors computed with `python3 -c "from eth_utils import keccak; print('0x' + keccak(text=sig).hex()[:8])"`.

| Function | Signature | Selector |
|---|---|---|
| `exactInputSingle` | `exactInputSingle((address,address,uint24,address,uint256,uint256,uint256,uint160))` | `0x414bf389` |
| `quoteExactInputSingle` | `quoteExactInputSingle((address,address,uint256,uint24,uint160))` | `0xc6a5026a` |
| `mint` | `mint((address,address,uint24,int24,int24,uint256,uint256,uint256,uint256,address,uint256))` | `0x88316456` |
| `increaseLiquidity` | `increaseLiquidity((uint256,uint256,uint256,uint256,uint256,uint256))` | `0x219f5d17` |
| `decreaseLiquidity` | `decreaseLiquidity((uint256,uint128,uint256,uint256,uint256))` | `0x0c49ccbe` |
| `collect` | `collect((uint256,address,uint128,uint128))` | `0xfc6f7865` |
| `burn` | `burn(uint256)` | `0x42966c68` |
| `positions` | `positions(uint256)` | `0x99fbab88` |
| `tokenOfOwnerByIndex` | `tokenOfOwnerByIndex(address,uint256)` | `0x2f745c59` |
| `balanceOf` | `balanceOf(address)` | `0x70a08231` |
| `approve` | `approve(address,uint256)` | `0x095ea7b3` |

**Note on exactInputSingle**: The SushiSwap V3 SwapRouter uses the same interface as Uniswap V3 with `deadline` parameter included:
- Struct: `(tokenIn, tokenOut, fee, recipient, deadline, amountIn, amountOutMinimum, sqrtPriceLimitX96)`
- Selector: `0x414bf389`

---

## §5 ABI Encoding Notes

### exactInputSingle params struct (SwapRouter)
```
tokenIn:          address (32 bytes)
tokenOut:         address (32 bytes)
fee:              uint24  (32 bytes, padded)
recipient:        address (32 bytes)
deadline:         uint256 (32 bytes)
amountIn:         uint256 (32 bytes)
amountOutMinimum: uint256 (32 bytes)
sqrtPriceLimitX96:uint160 (32 bytes)
```

### quoteExactInputSingle params struct (QuoterV2)
```
tokenIn:            address (32 bytes)
tokenOut:           address (32 bytes)
amountIn:           uint256 (32 bytes)
fee:                uint24  (32 bytes)
sqrtPriceLimitX96:  uint160 (32 bytes)
```

### mint params struct (NonfungiblePositionManager)
```
token0:             address (32 bytes)
token1:             address (32 bytes)
fee:                uint24  (32 bytes)
tickLower:          int24   (32 bytes, signed)
tickUpper:          int24   (32 bytes, signed)
amount0Desired:     uint256 (32 bytes)
amount1Desired:     uint256 (32 bytes)
amount0Min:         uint256 (32 bytes)
amount1Min:         uint256 (32 bytes)
recipient:          address (32 bytes)
deadline:           uint256 (32 bytes)
```

### collect params struct (NonfungiblePositionManager)
```
tokenId:            uint256 (32 bytes)
recipient:          address (32 bytes)
amount0Max:         uint128 (32 bytes) — use u128::MAX = 0xffffffffffffffffffffffffffffffff
amount1Max:         uint128 (32 bytes) — same
```

### decreaseLiquidity params struct
```
tokenId:            uint256 (32 bytes)
liquidity:          uint128 (32 bytes)
amount0Min:         uint256 (32 bytes)
amount1Min:         uint256 (32 bytes)
deadline:           uint256 (32 bytes)
```

---

## §6 Architecture

```
sushiswap-v3/
├── plugin.yaml              ← plugin metadata
├── skills/sushiswap-v3/
│   └── SKILL.md             ← skill documentation
├── src/
│   ├── main.rs              ← CLI entrypoint (clap)
│   ├── config.rs            ← contract addresses, chain configs
│   ├── onchainos.rs         ← resolve_wallet, contract_call helper
│   ├── rpc.rs               ← eth_call, read-only RPC queries
│   └── commands/
│       ├── mod.rs
│       ├── quote.rs         ← GET api.sushi.com/swap/v7
│       ├── swap.rs          ← approve + exactInputSingle
│       ├── pools.rs         ← read pool info
│       ├── positions.rs     ← tokenOfOwnerByIndex + positions()
│       ├── add_liquidity.rs ← approve + mint
│       ├── remove_liquidity.rs ← decreaseLiquidity + collect
│       └── collect_fees.rs  ← collect
├── tests/
│   └── selector_verification.md
├── Cargo.toml
├── .gitignore
├── LICENSE
└── README.md
```

---

## §7 Multi-Chain Support

The same contract addresses (Factory, SwapRouter, QuoterV2, NonfungiblePositionManager) are deployed at identical addresses across all supported chains (SushiSwap uses deterministic CREATE2 deployment).

**Confirmed identical addresses on all supported chains** via v3-periphery deployments repo.

RPC endpoints per chain are provided by onchainos CLI or can be specified via environment variables.
