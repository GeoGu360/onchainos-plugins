# Allbridge Classic — Plugin Design

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `allbridge-classic` |
| dapp_name | Allbridge Classic |
| target_chains | EVM (Ethereum 1, BSC 56, Polygon 137) + Solana (501) |
| target_protocols | Cross-chain stablecoin bridge |
| bitable_record_id | recvfIVNIZdjc6 |

---

## §1 Feasibility Research

| Check | Result |
|-------|--------|
| Rust SDK? | No. No official Rust SDK exists for Allbridge Classic. |
| SDK tech stack? | JavaScript/TypeScript only (allbridge-sdk on npm, but for Core not Classic) |
| REST API? | Yes. `https://allbridgeapi.net` — token-info, sign, check endpoints |
| Official Skill? | No official skill found |
| Community Skill? | No community skill found on GitHub |
| Supported chains | EVM (ETH, BSC, POL, AVA, FTM, CELO) + Solana + NEAR + Stellar + XRPL |
| Needs onchainos broadcast? | Yes — EVM lock/unlock calls via `wallet contract-call`; Solana bridge requires manual instruction encoding (complex) — focus on EVM + API-based Solana status queries |

**Integration path:** API (Rust calling REST API) + onchainos `wallet contract-call` for EVM on-chain operations.

**Note on Solana:** Solana bridge lock/unlock requires complex custom instruction encoding (Secp256k1 signature verification, multiple PDAs). This is beyond the scope of a standard plugin integration. Solana is supported for: status queries, token info, and get-pools. The EVM ↔ EVM flow and EVM lock with status tracking is the primary focus.

**Important:** Allbridge Classic is deprecated and will be stopped in mid-2026. The plugin covers the existing flow while the protocol is still operational.

---

## §2 Interface Mapping

### Operations

| # | Operation | Type | Chains |
|---|-----------|------|--------|
| 1 | get-tokens | off-chain read | All |
| 2 | get-tx-status | off-chain read | All |
| 3 | bridge (EVM lock) | on-chain write | EVM → Any |
| 4 | check-address | off-chain read | All |

---

### Off-chain Queries

#### get-tokens
```
GET https://allbridgeapi.net/token-info
```
No parameters required.

Response (per chain):
```json
{
  "AVA": [{ "tokenAddress": "0x...", "name": "...", "symbol": "USDT", "decimals": 6, "bridgingFee": "0.3", "minFee": "0.5" }],
  "BSC": [...],
  "ETH": [...],
  "POL": [...],
  "SOL": [...]
}
```

#### get-tx-status
```
GET https://allbridgeapi.net/sign/{transactionId}
```
- `transactionId`: The lock transaction ID (obtained from lock tx receipt, from the `Sent` event lockId field)

Response:
```json
{
  "lockId": "199936...",
  "block": "28598359",
  "source": "POL",
  "amount": "5000000000",
  "destination": "SOL",
  "recipient": "0x7972...",
  "tokenSource": "SOL",
  "tokenSourceAddress": "0x069b...",
  "signature": "012000000c0"
}
```
If lock not confirmed yet: returns 404 or error.

#### check-address
```
GET https://allbridgeapi.net/check/{blockchainId}/address/{address}
```
- `blockchainId`: e.g. "SOL", "ETH", "BSC", "POL"
- `address`: recipient address string

---

### On-chain Write Operations (EVM)

**Bridge contract: `0xBBbD1BbB4f9b936C3604906D7592A644071dE884`** (same address on all EVM chains: ETH, BSC, POL, AVA, FTM, CELO)

| Operation | Contract | Function Signature | Selector (cast verified ✅) | ABI Parameter Order |
|-----------|----------|--------------------|----------------------------|---------------------|
| lock (ERC-20 token) | `0xBBbD1BbB4f9b936C3604906D7592A644071dE884` | `lock(uint128,address,bytes32,bytes4,uint256)` | `0x7bacc91e` ✅ | lockId, tokenAddress, recipient(bytes32), destination(bytes4), amount |
| approve (token) | token contract address | `approve(address,uint256)` | `0x095ea7b3` ✅ | spender(bridge addr), amount |

**Notes on `lock` parameters:**
- `lockId`: random 16-byte value; first byte MUST be `0x01` (bridge version). Generate as: `0x01` + 15 random bytes → encode as uint128
- `tokenAddress`: ERC-20 token contract address
- `recipient`: destination chain recipient as 32 bytes. For EVM addresses (20 bytes), right-pad with zeros to 32 bytes
- `destination`: blockchain ID as 4-byte UTF8, right-padded with zeros: e.g. `SOL\0` = `0x534f4c00`, `BSC\0` = `0x42534300`, `POL\0` = `0x504f4c00`, `ETH\0` = `0x45544800`
- `amount`: token amount in token native decimals (e.g. USDT with 6 decimals: 1 USDT = 1_000_000)

**Flow before bridge:**
1. ERC-20 `approve(bridge_contract, amount)` via `wallet contract-call`
2. `lock(lockId, tokenAddress, recipient32, destination4, amount)` via `wallet contract-call`
3. Poll `GET /sign/{lockId_decimal}` until signature returned (bridge confirmed)

---

### Blockchain IDs

| Chain | ID (4-byte UTF8) | Hex |
|-------|-----------------|-----|
| Ethereum | `ETH\0` | `0x45544800` |
| BSC | `BSC\0` | `0x42534300` |
| Polygon | `POL\0` | `0x504f4c00` |
| Avalanche | `AVA\0` | `0x41564100` |
| Solana | `SOL\0` | `0x534f4c00` |
| Fantom | `FTM\0` | `0x46544d00` |

---

## §3 User Scenarios

### Scenario 1: List supported tokens and chains
**User says:** "Show me what tokens I can bridge with Allbridge Classic"

**Agent actions:**
1. [off-chain] `GET https://allbridgeapi.net/token-info`
2. Parse response, group by chain
3. Display: chain → [token symbol, fee%, min fee]

---

### Scenario 2: Bridge USDT from Ethereum to Solana
**User says:** "Bridge 10 USDT from Ethereum to my Solana wallet DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE"

**Agent actions:**
1. [off-chain] `GET /token-info` → find USDT on ETH (address: `0xdac17f958d2ee523a2206206994597c13d831ec7`)
2. [off-chain] `GET /check/SOL/address/DTEqFXyFM9...` → validate recipient
3. [on-chain] `approve(bridge=0xBBbD..., amount=10_000_000)` on USDT contract, chain 1
   - calldata: `0x095ea7b3` + padded bridge addr + padded amount
   - command: `onchainos wallet contract-call --chain 1 --to 0xdac17f958d2ee523a2206206994597c13d831ec7 --input-data <calldata>`
4. Generate lockId: `0x01` + 15 random bytes
5. Encode recipient: Solana base58 → 32 bytes
6. Encode destination: `SOL\0` = `0x534f4c00`
7. [on-chain] `lock(lockId, USDT_addr, recipient32, 0x534f4c00, 10_000_000)` on bridge, chain 1
   - command: `onchainos wallet contract-call --chain 1 --to 0xBBbD1BbB4f9b936C3604906D7592A644071dE884 --input-data <calldata>`
8. Return lockId (as decimal) to user, explain they can check status with `get-tx-status`

---

### Scenario 3: Check bridge transaction status
**User says:** "Check status of my Allbridge transaction 199936896233..."

**Agent actions:**
1. [off-chain] `GET https://allbridgeapi.net/sign/199936896233...`
2. If 200: show bridge confirmed (amount, source, destination, recipient)
3. If 404/error: transaction not yet confirmed — advise to wait

---

### Scenario 4: Check if a recipient address is valid
**User says:** "Is DTEqFXyFM9aMSGu9sw3PpRsZce6xqqmaUbGkFjmeieGE a valid Solana address for Allbridge?"

**Agent actions:**
1. [off-chain] `GET https://allbridgeapi.net/check/SOL/address/DTEqFXyFM9...`
2. Return validity result

---

## §4 External API Dependencies

| API | URL | Auth | Used for |
|-----|-----|------|---------|
| Allbridge API | `https://allbridgeapi.net` | None | token-info, sign, check |

---

## §5 Configuration Parameters

| Param | Default | Description |
|-------|---------|-------------|
| dry_run | false | If true, skip on-chain tx, return mock txHash |
| chain | 1 (Ethereum) | Source chain ID for bridge operations |

---

## §6 Contract Addresses Summary

| Chain | Chain ID | Bridge Contract |
|-------|----------|-----------------|
| Ethereum | 1 | `0xBBbD1BbB4f9b936C3604906D7592A644071dE884` |
| BSC | 56 | `0xBBbD1BbB4f9b936C3604906D7592A644071dE884` |
| Polygon | 137 | `0xBBbD1BbB4f9b936C3604906D7592A644071dE884` |
| Avalanche | 43114 | `0xBBbD1BbB4f9b936C3604906D7592A644071dE884` |

Note: Same contract address on all EVM chains.
