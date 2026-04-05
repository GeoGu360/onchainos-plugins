# Relay Plugin Design

## Overview

Relay is a cross-chain bridge and swap protocol supporting 74+ EVM chains, using a solver network for instant, low-cost settlement. The plugin enables users to bridge assets across chains, check quotes, list supported chains/currencies, and monitor bridge status.

## API Base URL

`https://api.relay.link/`

## Key Endpoints

### GET /chains
Returns supported chains with metadata.

**Response:**
```json
{
  "chains": [
    {
      "id": 1,
      "name": "ethereum",
      "displayName": "Ethereum",
      "httpRpcUrl": "...",
      "depositEnabled": true,
      "currency": { "symbol": "ETH", "address": "0x0000...", "decimals": 18 }
    }
  ]
}
```

### POST /currencies/v1
Returns supported tokens (grouped by token type) for given chain IDs.

**Request:**
```json
{
  "chainIds": [8453],
  "defaultList": true,
  "limit": 20
}
```

**Response:** Array of arrays (grouped token objects), each with `chainId`, `address`, `symbol`, `name`, `decimals`, `metadata`.

### POST /quote
Get bridge/swap quote with execution steps.

**Request:**
```json
{
  "user": "0x...",
  "originChainId": 8453,
  "destinationChainId": 1,
  "originCurrency": "0x0000000000000000000000000000000000000000",
  "destinationCurrency": "0x0000000000000000000000000000000000000000",
  "amount": "50000000000000",
  "tradeType": "EXACT_INPUT"
}
```

**Response:**
```json
{
  "steps": [
    {
      "id": "deposit",
      "action": "Confirm transaction in your wallet",
      "kind": "transaction",
      "items": [
        {
          "status": "incomplete",
          "data": {
            "from": "0x...",
            "to": "0x4cd00e387622c35bddb9b4c962c136462338bc31",
            "data": "0x49290c1c...",
            "value": "50000000000000",
            "chainId": 8453,
            "gas": "32713"
          },
          "check": {
            "endpoint": "/intents/status?requestId=0x...",
            "method": "GET"
          }
        }
      ],
      "requestId": "0x..."
    }
  ],
  "fees": {
    "gas": { "amount": "165013139645", "amountFormatted": "...", "amountUsd": "0.000338" },
    "relayer": { "amount": "12680145505879", "amountFormatted": "...", "amountUsd": "0.025939" },
    "relayerGas": { ... },
    "relayerService": { ... }
  },
  "details": {
    "operation": "swap",
    "sender": "0x...",
    "recipient": "0x...",
    "currencyIn": { "currency": {...}, "amount": "...", "amountUsd": "..." },
    "currencyOut": { "currency": {...}, "amount": "...", "amountUsd": "..." },
    "totalImpact": { "usd": "...", "percent": "..." },
    "timeEstimate": 30
  }
}
```

### GET /intents/status
Check bridge transaction status.

**Request:** `GET /intents/status?requestId=0x...`

**Response:** `{ "status": "waiting" | "pending" | "success" | "failed" | "refunded" | "unknown" }`

### GET /requests/v2
Get cross-chain request history for a user.

**Request:** `GET /requests/v2?originChainId=8453&user=0x...&limit=10`

## Execution Model

Relay uses a **steps model**. The `/quote` endpoint returns an array of `steps`, each with:
- `id`: step type (deposit, approve, authorize, swap, send)
- `kind`: "transaction" or "signature"
- `items[].data`: transaction data `{ to, data, value, chainId, gas }`
- `items[].check.endpoint`: polling URL to check step completion

For bridge operations (ETH to ETH across chains), there is typically one step: `deposit`. This deposits ETH to the Relay relayer contract which then handles the cross-chain fill.

## Operations

| Operation | Type | Description |
|-----------|------|-------------|
| `chains` | read | List all supported chains |
| `currencies` | read | List supported tokens on a given chain |
| `quote` | read | Get bridge quote with fees and execution steps |
| `bridge` | write | Execute bridge by submitting the deposit step tx |
| `status` | read | Check bridge transaction status by requestId |

## Bridge Flow

1. Call `POST /quote` with user, chains, currencies, amount
2. Parse `steps[0].items[0].data` for the tx to submit
3. Call `onchainos wallet contract-call --chain <src> --to <step.to> --input-data <step.data> --amt <step.value> --force`
4. Extract `requestId` from steps response
5. Poll `GET /intents/status?requestId=...` to monitor progress

## Chains Supported

74 chains confirmed including: Ethereum (1), Base (8453), Arbitrum (42161), Optimism (10), Polygon (137), BSC (56), Avalanche (43114), Linea, Scroll, zkSync, Mantle, Blast, Mode, and more.

## Design Decisions

- **Primary test chain:** Base (8453) → Ethereum (1)
- **ETH address:** `0x0000000000000000000000000000000000000000`
- **No auth required** for any API calls
- **Amount in wei** for all API calls
- **Relay relayer contract** on Base: `0x4cd00e387622c35bddb9b4c962c136462338bc31`
- L4 test: quote only (actual bridge fee ~0.026 USD but gas + relayer total is ~50000000000000 wei = 0.00005 ETH which is right at the limit — skip actual bridge to be safe)
