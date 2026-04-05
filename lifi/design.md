# LI.FI/Jumper Plugin Design

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `lifi` |
| dapp_name | LI.FI/Jumper |
| target_chains | EVM (Ethereum, Arbitrum, Base, Polygon, Optimism, etc.) |
| target_protocols | Cross-chain bridge/swap aggregation |
| bitable_record_id | recvfIUuJYSiXF |

---

## §1 Feasibility Research

| Check | Result |
|-------|--------|
| Rust SDK? | No official Rust SDK. JS/TS SDK only (`@lifi/sdk`). Using REST API directly. |
| SDK tech stacks | TypeScript/JavaScript only |
| REST API? | Yes — https://li.quest/v1/ (no auth required for basic use; optional `x-lifi-api-key` for higher rate limits) |
| Official Skill? | None found |
| Community Skill? | None found |
| Supported chains | 79 EVM chains (Ethereum, Arbitrum, Base, Optimism, Polygon, BSC, Avalanche, zkSync, Linea, Scroll, etc.) |
| Needs onchainos broadcast? | Yes — quote returns `transactionRequest.data` (pre-built calldata) + `transactionRequest.to` (LiFiDiamond: `0x1231DEB6f5749EF6cE6943a275A1D3E7486F4EaE`) + `transactionRequest.value`. Submit via `onchainos wallet contract-call`. |

**Onboarding path: API** — LI.FI provides a REST API that returns pre-built `transactionRequest` objects. The plugin calls the API, extracts the calldata, and submits via `onchainos wallet contract-call`. No manual ABI encoding needed.

**Key insight:** LI.FI's API returns the complete calldata in `transactionRequest.data`. The plugin:
1. Calls GET /quote (or POST /advanced/routes)
2. Gets `transactionRequest.to`, `transactionRequest.data`, `transactionRequest.value`
3. If ERC-20 approval needed (value=0, non-native token), sends approve tx first
4. Submits the bridge/swap tx via `onchainos wallet contract-call`

---

## §2 Interface Mapping

### Operations

| Operation | Type | Description |
|-----------|------|-------------|
| `get-chains` | Off-chain read | List all supported chains |
| `get-tokens` | Off-chain read | List tokens on a chain |
| `get-quote` | Off-chain read | Get bridge/swap quote |
| `get-status` | Off-chain read | Check cross-chain transfer status |
| `get-tools` | Off-chain read | List available bridges and DEXes |
| `swap` | On-chain write | Execute a cross-chain swap/bridge |

### Off-chain queries

| Operation | Endpoint | Key Params | Key Response Fields |
|-----------|---------|------------|---------------------|
| get-chains | GET https://li.quest/v1/chains | none | chains[].id, chains[].name, chains[].key, chains[].chainType, chains[].diamondAddress |
| get-tokens | GET https://li.quest/v1/tokens | chains (comma-separated chainIds) | tokens.{chainId}[].address, .symbol, .decimals, .priceUSD |
| get-quote | GET https://li.quest/v1/quote | fromChain, toChain, fromToken, toToken, fromAmount (raw), fromAddress, slippage | transactionRequest.{to,data,value,chainId}, estimate.{fromAmount,toAmount,feeCosts,gasCosts}, toolDetails.key |
| get-status | GET https://li.quest/v1/status | txHash, bridge (optional), fromChain, toChain | status ("DONE"/"PENDING"/"FAILED"), substatus, sending.txHash, receiving.txHash |
| get-tools | GET https://li.quest/v1/tools | chains (optional) | bridges[].key, exchanges[].key |

### On-chain write operations (EVM)

| Operation | Contract Address | Pre-built calldata source | Value |
|-----------|-----------------|--------------------------|-------|
| swap/bridge | `0x1231DEB6f5749EF6cE6943a275A1D3E7486F4EaE` (LiFiDiamond, same on all chains) | `transactionRequest.data` from GET /quote response | `transactionRequest.value` (hex, may be 0 for ERC-20 or non-zero for native ETH) |

> **Note:** LI.FI Diamond address is `0x1231DEB6f5749EF6cE6943a275A1D3E7486F4EaE` on all supported EVM chains.
> The function selector varies by bridge route (e.g., `0x1794958f` for startBridgeTokensViaBridge, `0x9e75aa95` for swapAndStartBridgeTokensVia*).
> These are fully encoded in `transactionRequest.data` — no manual encoding needed.

**ERC-20 Approve (before bridge, if token is not native):**
- Contract: token address (from `action.fromToken.address`)  
- Calldata: `approve(address spender, uint256 amount)` = `0x095ea7b3` + LiFiDiamond padded + amount padded
- Check allowance first — skip approve if already sufficient

---

## §3 User Scenarios

### Scenario 1: Bridge USDC from Base to Arbitrum

User says: "Bridge 5 USDC from Base to Arbitrum using LI.FI"

Agent actions:
1. [Off-chain] Call GET /quote with fromChain=8453, toChain=42161, fromToken=USDC, toToken=USDC, fromAmount=5000000 (6 decimals), fromAddress=wallet
2. Display quote: bridge=AcrossV4, receive≈4.99 USDC, fees, estimated 4 minutes
3. Ask user to confirm
4. [On-chain] If USDC allowance < fromAmount: send ERC-20 approve tx via `onchainos wallet contract-call --chain 8453 --to 0xUSDC --input-data 0x095ea7b3...`
5. Wait 15s for approve to confirm
6. [On-chain] Submit bridge tx: `onchainos wallet contract-call --chain 8453 --to 0x1231DEB6f5749EF6cE6943a275A1D3E7486F4EaE --input-data <transactionRequest.data> --force`
7. Return txHash, link to li.fi/scan

### Scenario 2: Get supported chains and tokens

User says: "What chains does LI.FI support?" / "Show me USDC on Ethereum"

Agent actions:
1. [Off-chain] Call GET /chains
2. Return list: 79 chains including Ethereum(1), Base(8453), Arbitrum(42161), Polygon(137), etc.

OR

1. [Off-chain] Call GET /tokens?chains=1
2. Filter by symbol="USDC", return address, decimals, priceUSD

### Scenario 3: Check cross-chain transfer status

User says: "Check status of my LI.FI transfer txHash 0xabc..."

Agent actions:
1. [Off-chain] Call GET /status?txHash=0xabc&fromChain=1&toChain=8453
2. Return: status=DONE/PENDING/FAILED, source txHash, destination txHash, substatus message

### Scenario 4: Swap ETH to USDC on Base (same-chain)

User says: "Swap 0.00005 ETH to USDC on Base via LI.FI"

Agent actions:
1. [Off-chain] GET /quote with fromChain=8453, toChain=8453, fromToken=ETH, toToken=USDC, fromAmount=50000000000000 (wei), fromAddress=wallet
2. Display: receive≈X USDC, fee, tool=some DEX
3. Ask user to confirm
4. [On-chain] Submit swap tx: `onchainos wallet contract-call --chain 8453 --to 0x1231DEB6f5749EF6cE6943a275A1D3E7486F4EaE --input-data <transactionRequest.data> --amt <transactionRequest.value in decimal> --force`

---

## §4 External API Dependencies

| API | URL | Auth | Purpose |
|-----|-----|------|---------|
| LI.FI API | https://li.quest/v1/ | None (optional `x-lifi-api-key` header) | Chains, tokens, quotes, status, tools |

---

## §5 Config Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| chain | 8453 (Base) | Source chain ID |
| slippage | 0.005 (0.5%) | Slippage tolerance |
| dry_run | false | Preview without broadcasting |
| from | (resolve from onchainos) | Source wallet address |

---

## Notes

- LI.FI supports EVM only (no native Solana/SVM); plugin targets EVM chains
- The API is open (no API key required), so no config secrets needed
- ERC-20 approve must check existing allowance to avoid redundant txs
- After bridging, user can track via https://scan.li.fi/tx/{txHash}
- DEX swaps on same chain also go through LiFiDiamond (`--force` required)
