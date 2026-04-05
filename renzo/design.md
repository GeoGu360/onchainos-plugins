# Plugin Design: Renzo EigenLayer Restaking

**Plugin name:** `renzo`  
**DApp:** Renzo Protocol  
**Category:** defi-protocol  
**Tags:** restaking, eigenlayer, liquid-restaking, ezeth, eth  
**Author:** Plugin Dev Pipeline — Phase 1 Researcher Agent  
**Date:** 2026-04-05  
**Status:** Draft

---

## §0 Plugin Meta

| Field | Value |
|---|---|
| `plugin_name` | `renzo` |
| `dapp_name` | Renzo |
| `target_chains` | Ethereum mainnet (chain ID 1) |
| `target_protocols` | EigenLayer restaking via Renzo |
| `category` | defi-protocol |

---

## §1 Feasibility Table

| Dimension | Assessment |
|---|---|
| Official Rust SDK | None — no official Rust/WASM SDK published |
| SDK supported stacks | TypeScript SDK via `@renzo-protocol/sdk` (not Rust) |
| REST API | Yes — `https://api.renzoprotocol.com/apr` for APR; `https://api.renzoprotocol.com/tvl` for TVL |
| Official Plugin / Skill | None published |
| Community Skill | None found |
| Supported chains | Ethereum mainnet (chain 1) for staking/restaking; bridged ezETH on L2s (read-only) |
| onchainos broadcast needed | Yes — depositETH, deposit (stETH), approve (ERC-20 stETH) all require onchainos wallet contract-call |
| Integration path | Direct EVM contract calls via onchainos CLI + Renzo REST API for APR/TVL |

**Access path:** API (Renzo REST API for queries) + direct EVM contract calls (no Rust SDK available)

---

## §2 Interface Mapping

### Verified Contract Addresses (Ethereum Mainnet, chain 1)

| Contract | Address | Source |
|---|---|---|
| RestakeManager (proxy) | `0x74a09653A083691711cF8215a6ab074BB4e99ef5` | Etherscan verified proxy |
| RestakeManager (impl) | `0xD5B3Be349ed0b7c82dbd9271ce3739a381Fc7aa0` | From proxy.implementation() |
| ezETH token (proxy) | `0xbf5495Efe5DB9ce00f80364C8B423567e58d2110` | Etherscan ERC-20, totalSupply ~185k |
| RenzoOracle | `0x5a12796f7e7ebbbc8a402667d266d2e65a814042` | From RestakeManager.renzoOracle() |
| stETH (Lido, accepted collateral) | `0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84` | From RestakeManager.collateralTokens(0) |

**Note:** RestakeManager accepts only 1 collateral token (stETH) at current state. Native ETH deposit is also supported via `depositETH()`.

### Verified Function Selectors

All selectors computed via `cast sig` (Foundry keccak256) — **not** Python hashlib.sha3_256.

#### RestakeManager (`0x74a09653A083691711cF8215a6ab074BB4e99ef5`)

| Function signature | Selector | cast sig verified |
|---|---|---|
| `depositETH()` | `0xf6326fb3` | ✅ |
| `depositETH(uint256)` | `0x5358fbda` | ✅ |
| `deposit(address,uint256)` | `0x47e7ef24` | ✅ |
| `deposit(address,uint256,uint256)` | `0x0efe6a8b` | ✅ |
| `calculateTVLs()` | `0xff9969cd` | ✅ |
| `getCollateralTokensLength()` | `0x75c745a6` | ✅ |
| `collateralTokens(uint256)` | `0x172c48c7` | ✅ |
| `renzoOracle()` | `0x892866a4` | ✅ |
| `ezETH()` | `0x13a73c78` | ✅ |
| `paused()` | `0x5c975abb` | ✅ |

#### ezETH Token (`0xbf5495Efe5DB9ce00f80364C8B423567e58d2110`)

| Function signature | Selector | cast sig verified |
|---|---|---|
| `balanceOf(address)` | `0x70a08231` | ✅ |
| `totalSupply()` | `0x18160ddd` | ✅ |

#### stETH Token (`0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84`)

| Function signature | Selector | cast sig verified |
|---|---|---|
| `balanceOf(address)` | `0x70a08231` | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | ✅ |
| `allowance(address,address)` | `0xdd62ed3e` | ✅ |

---

### Operation: `deposit-eth` — Deposit native ETH to receive ezETH

**Description:** Deposit ETH into Renzo RestakeManager to receive ezETH (liquid restaking token). This calls `depositETH()` payable function — ETH amount is sent as msg.value.

**Solidity:**
```solidity
// RestakeManager: 0x74a09653A083691711cF8215a6ab074BB4e99ef5
function depositETH() external payable;
// OR with referralId
function depositETH(uint256 _referralId) public payable nonReentrant notPaused;
```

**ABI encoding:**
- `depositETH()`: selector `0xf6326fb3`, no parameters
- `depositETH(uint256 _referralId)`: selector `0x5358fbda` + 32-byte referralId (use 0 if no referral)

**onchainos command (deposit 0.00005 ETH):**
```bash
onchainos wallet contract-call \
  --chain 1 \
  --to 0x74a09653A083691711cF8215a6ab074BB4e99ef5 \
  --input-data 0xf6326fb3 \
  --amt 50000000000000
```

**Pre-flight checks:**
- Call `paused()` → if true, abort with "Renzo is paused"
- Verify `amount_wei > 0`

---

### Operation: `deposit-steth` — Deposit stETH to receive ezETH

**Description:** Deposit stETH LST into RestakeManager to receive ezETH. Requires ERC-20 approve first.

**Solidity:**
```solidity
function deposit(IERC20 _collateralToken, uint256 _amount) external;
```

**ABI encoding for `deposit(address,uint256)`:**
- Selector: `0x47e7ef24`
- Param 1: stETH address (32-byte padded)
- Param 2: amount in wei (32-byte)

**Two-transaction flow:**

**TX 1 — Approve stETH:**
```bash
# approve(address,uint256) → 0x095ea7b3
# spender = RestakeManager, amount = deposit_amount
onchainos wallet contract-call \
  --chain 1 \
  --to 0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84 \
  --input-data 0x095ea7b3000000000000000000000000<RESTAKE_MANAGER_PADDED><AMOUNT_PADDED>
```

**TX 2 — Deposit stETH:**
```bash
onchainos wallet contract-call \
  --chain 1 \
  --to 0x74a09653A083691711cF8215a6ab074BB4e99ef5 \
  --input-data 0x47e7ef24<STETH_PADDED><AMOUNT_PADDED>
```

**Pre-flight checks:**
- Call `paused()` → if true, abort
- Check stETH `allowance(wallet, RestakeManager)` — skip approve if sufficient

---

### Operation: `get-apr` — Get current Renzo APR

**Description:** Fetch current restaking APR from Renzo REST API. No on-chain call needed.

**REST API:**
```
GET https://api.renzoprotocol.com/apr
Response: {"apr": 2.52}
```

**Verified response structure (actual curl test result):**
```json
{"apr":2.520837480169691}
```

**onchainos command:** Not applicable — pure HTTP GET.

---

### Operation: `balance` — Check ezETH balance

**Description:** Query ezETH balance for a given address.

**Read call (direct eth_call via JSON-RPC):**
```bash
# balanceOf(address) = 0x70a08231
# to: ezETH contract, param: address padded to 32 bytes
# RPC: https://ethereum.publicnode.com
```

**Returns:** uint256 ezETH balance in wei. Divide by 1e18 for display.

---

### Operation: `get-tvl` — Get protocol TVL

**Description:** Get the total value locked in Renzo via calculateTVLs() on RestakeManager, or from the Renzo API.

**on-chain read (calculateTVLs):**
```bash
# calculateTVLs() = 0xff9969cd
# Returns: (uint256[][] operatorDelegatorTvls, uint256[] tvls, uint256 totalTVL)
# totalTVL is the last uint256 in the return (currently ~55k ETH on mainnet)
```

---

## §3 User Scenarios

### Scenario 1: Alice deposits 0.00005 ETH to receive ezETH

1. Alice: "Deposit 0.00005 ETH into Renzo"
2. Plugin calls `paused()` on RestakeManager → false, proceed
3. Plugin calls `get-apr` → shows "Current Renzo APR: 2.52%"
4. Plugin shows: deposit amount = 0.00005 ETH, expected ezETH output, contract address
5. **Agent asks Alice to confirm** the deposit transaction
6. Plugin executes:
   ```bash
   onchainos wallet contract-call \
     --chain 1 \
     --to 0x74a09653A083691711cF8215a6ab074BB4e99ef5 \
     --input-data 0xf6326fb3 \
     --amt 50000000000000
   ```
7. Alice receives ezETH. Display txHash and new ezETH balance.

---

### Scenario 2: Bob deposits stETH to earn restaking yield

1. Bob: "Deposit my stETH into Renzo"
2. Plugin calls `paused()` → false
3. Plugin calls `balance` for stETH → shows Bob's stETH balance
4. Plugin checks stETH `allowance(bob_addr, RestakeManager)` → 0, need approve
5. Plugin shows approve details. **Agent asks Bob to confirm** approve tx
6. Plugin executes approve:
   ```bash
   onchainos wallet contract-call --chain 1 \
     --to 0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84 \
     --input-data 0x095ea7b3<RESTAKE_MGR_PADDED><AMOUNT_PADDED>
   ```
7. Plugin shows deposit details. **Agent asks Bob to confirm** deposit tx
8. Plugin executes deposit:
   ```bash
   onchainos wallet contract-call --chain 1 \
     --to 0x74a09653A083691711cF8215a6ab074BB4e99ef5 \
     --input-data 0x47e7ef24<STETH_PADDED><AMOUNT_PADDED>
   ```
9. Bob receives ezETH.

---

### Scenario 3: Carol checks her ezETH balance and current APR

1. Carol: "What's my Renzo balance and current APR?"
2. Plugin calls `balanceOf(carol_addr)` on ezETH contract → returns balance in wei
3. Plugin calls `GET https://api.renzoprotocol.com/apr` → returns 2.52%
4. Plugin displays: "Your ezETH balance: X.XXX ezETH | Current APR: 2.52%"
5. No transactions required (read-only operations).

---

### Scenario 4: Dave checks Renzo TVL

1. Dave: "What's the total TVL in Renzo?"
2. Plugin calls `calculateTVLs()` on RestakeManager → returns total TVL in ETH
3. Plugin displays: "Renzo total TVL: ~55,378 ETH"
4. No transactions required.

---

## §4 External API Dependencies

| API | URL | Purpose | Auth required |
|---|---|---|---|
| Renzo APR | `https://api.renzoprotocol.com/apr` | Current restaking APR | None |
| Ethereum RPC (read) | `https://ethereum.publicnode.com` | eth_call for contract queries | None |
| Ethereum RPC (write) | Via onchainos wallet CLI | depositETH, deposit, approve | Wallet signature |

**API response verification (actual curl test 2026-04-05):**
- `https://api.renzoprotocol.com/apr` → `{"apr":2.520837480169691}` (single field `apr` as float)

---

## §5 Configuration Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `chain_id` | integer | `1` | Ethereum mainnet |
| `restake_manager` | string | `0x74a09653A083691711cF8215a6ab074BB4e99ef5` | RestakeManager proxy |
| `ezeth_address` | string | `0xbf5495Efe5DB9ce00f80364C8B423567e58d2110` | ezETH token proxy |
| `steth_address` | string | `0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84` | stETH (Lido) — accepted collateral |
| `renzo_oracle` | string | `0x5a12796f7e7ebbbc8a402667d266d2e65a814042` | RenzoOracle for mint calculations |
| `dry_run` | bool | `false` | If true, return calldata without broadcasting |
| `rpc_url` | string | `https://ethereum.publicnode.com` | Ethereum JSON-RPC endpoint |
| `api_base_url` | string | `https://api.renzoprotocol.com` | Renzo REST API base URL |

---

## §6 Caveats & Integration Notes

### depositETH requires msg.value
`depositETH()` is payable. The ETH amount must be sent via `--amt <wei>` in onchainos. Without `--amt`, the call reverts.

### stETH deposit requires 2 transactions
`deposit(address,uint256)` requires prior `approve(RestakeManager, amount)` on stETH. Always check allowance first; skip approve if sufficient.

### paused() check
Renzo admins can pause the protocol. Always call `paused()` before building deposit transactions.

### ezETH is non-rebasing
Unlike stETH, ezETH is a non-rebasing ERC-20. The exchange rate (ETH per ezETH) appreciates over time as restaking rewards accrue.

### No withdrawal on mainnet (current state)
As of 2026-04, Renzo has not enabled native ETH withdrawal from ezETH on Ethereum mainnet. Users must use secondary DEX markets (swap ezETH → ETH on Uniswap/Curve) to exit.

### Only ETH and stETH supported
The RestakeManager currently only accepts native ETH and stETH as deposits. wBETH and other LSTs shown in older docs are not currently active collateral.
