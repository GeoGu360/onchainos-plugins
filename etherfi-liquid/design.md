# Plugin Design: ether.fi Liquid

**Plugin name:** `etherfi-liquid`
**DApp:** ether.fi Liquid
**Category:** defi-protocol
**Tags:** yield, vault, liquid-staking, restaking, eth, weeth, etherfi, erc-4626
**Author:** Plugin Dev Pipeline — Phase 1 Researcher Agent
**Date:** 2026-04-05
**Status:** Draft

---

## §0 Plugin Meta

| Field | Value |
|---|---|
| plugin_name | `etherfi-liquid` |
| dapp_name | ether.fi Liquid |
| target_chains | EVM — Ethereum mainnet (chain ID 1) |
| target_protocols | ether.fi Liquid multi-strategy yield vaults (Veda BoringVault architecture) |
| primary_vault | ETH Yield Vault (LIQUIDETH) — accepts weETH/WETH/eETH |
| vault_token | LIQUIDETH (`0xf0bb20865277abd641a307ece5ee04e79073416c`) |

---

## §1 Feasibility Table

| Dimension | Assessment |
|---|---|
| Official Rust/WASM SDK | None — ether.fi Liquid has no official client SDK |
| REST API (off-chain) | No official ether.fi REST API for vault rates. DefiLlama public yields API available for APY data (no auth required) |
| Official Plugin / Skill | None published |
| Community Skill or MCP server | None found |
| Integration path | Direct EVM contract calls via `onchainos wallet contract-call`; reads via direct `eth_call` |
| onchainos broadcast needed | Yes — for `deposit`, `withdraw` |
| Primary chain | Ethereum mainnet (chain ID 1) |
| Other chains | Hyperliquid L1 has LIQUIDHYPE vault — out of scope (not supported by onchainos EVM) |

**Chosen integration path:** Direct EVM calls to Teller contract (deposit/withdraw), Accountant (rates), BoringVault ERC-20 (positions). APY from DefiLlama yields API.

**Architecture note:** ether.fi Liquid vaults use Veda's BoringVault architecture, NOT standard ERC-4626. The entry point is the **Teller** contract, which handles multi-asset deposits and withdrawals. The BoringVault is the share token. The Accountant holds exchange rates.

---

## §2 Interface Mapping

### Contract Addresses — Ethereum Mainnet (chain ID 1)

#### ETH Yield Vault (LIQUIDETH) — Primary vault for this plugin

| Contract | Address |
|---|---|
| BoringVault (share token LIQUIDETH) | `0xf0bb20865277abd641a307ece5ee04e79073416c` |
| Teller (deposit/withdraw entry) | `0x9AA79C84b79816ab920bBcE20f8f74557B514734` |
| Accountant (rates) | `0x0d05D94a5F1E76C18fbeB7A13d17C8a314088198` |

#### USD Yield Vault (LIQUIDUSD) — Stablecoin vault

| Contract | Address |
|---|---|
| BoringVault (share token LIQUIDUSD) | `0x08c6F91e2B681FaF5e17227F2a44C307b3C1364C` |
| Teller | `0x4DE413a26fC24c3FC27Cc983be70aA9c5C299387` |
| Accountant | `0xc315D6e14DDCDC7407784e2Caf815d131Bc1D3E7` |

#### BTC Yield Vault (LIQUIDBTC) — Bitcoin vault

| Contract | Address |
|---|---|
| BoringVault (share token LIQUIDBTC) | `0x5f46d540b6eD704C3c8789105F30E075AA900726` |
| Teller | `0x8Ea0B382D054dbEBeB1d0aE47ee4AC433C730353` |
| Accountant | `0xEa23aC6D7D11f6b181d6B98174D334478ADAe6b0` |

#### Token Addresses (Ethereum Mainnet)

| Token | Address | Decimals |
|---|---|---|
| weETH | `0xCd5fE23C85820F7B72D0926FC9b05b43E359b7ee` | 18 |
| WETH | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` | 18 |
| eETH | `0x35fA164735182de50811E8e2E824cFb9B6118ac2` | 18 |
| USDC | `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48` | 6 |
| WBTC | `0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599` | 8 |

---

### Verified Function Selectors

All selectors verified with `cast sig` (Foundry).

#### Teller Contract

| Function signature | Selector | Notes |
|---|---|---|
| `deposit(address,uint256,uint256,address)` | `0x8b6099db` | depositAsset, assets, minSharesOut, receiver |
| `bulkWithdraw(address,uint256[],uint256[],address[])` | `0x8432f02b` | withdrawAsset, shares[], minAssetsOut[], receivers[] |
| `assetData(address)` | `0x41fee44a` | Returns (allowDeposits, allowWithdrawals, sharePremium) |
| `vault()` | `0xfbfa77cf` | Returns BoringVault address |

#### Accountant Contract

| Function signature | Selector | Notes |
|---|---|---|
| `getRateInQuote(address)` | `0x1dcbb110` | Returns share price in given quote token (18 dec) |
| `getRateInQuoteSafe(address)` | `0x820973da` | Safe version (reverts if stale), same ABI |

#### BoringVault (ERC-20 share token)

| Function signature | Selector | Notes |
|---|---|---|
| `totalSupply()` | `0x18160ddd` | Total shares in circulation |
| `balanceOf(address)` | `0x70a08231` | Share balance of address |

#### ERC-20 (weETH/WETH/USDC for approve)

| Function signature | Selector | Notes |
|---|---|---|
| `approve(address,uint256)` | `0x095ea7b3` | Approve Teller as spender |
| `allowance(address,address)` | `0xdd62ed3e` | Check current allowance |

---

### Operation: `vaults`

**Description:** List available Liquid vaults with current APY, TVL, accepted tokens, and vault addresses.

**Data source:** DefiLlama yields API (pool IDs are stable).

**DefiLlama pool IDs:**
- LIQUIDETH: `b86d4934-2e75-415a-bdd2-e28143d72491`
- LIQUIDUSD: `7c12f175-37bc-41db-967a-1d7f1f4a23c4`
- LIQUIDBTC: `2f063aed-0a5a-4ba5-8b63-8404b2a99fca`

**API endpoint:**
```
GET https://yields.llama.fi/chart/{pool_id}
```
Returns: `{ data: [{ timestamp, tvlUsd, apy, apyBase, apyBase7d }] }` — use last entry.

**On-chain supplement:** Read `getRateInQuote(weETH_addr)` on ETH vault Accountant to get current share price.

**onchainos command:** Not applicable — pure HTTP GET + eth_call.

---

### Operation: `positions`

**Description:** Show user's current positions across all Liquid vaults.

**Steps:**
1. Resolve wallet address via `onchainos wallet balance --chain 1 --output json`
2. Call `balanceOf(wallet)` on each BoringVault (LIQUIDETH, LIQUIDUSD, LIQUIDBTC)
3. Call `getRateInQuote(depositToken)` on each Accountant to get USD-equivalent value

**eth_call for LIQUIDETH balance:**
```bash
# balanceOf(address) = 0x70a08231
curl -s -X POST https://ethereum.publicnode.com \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_call","params":[{"to":"0xf0bb20865277abd641a307ece5ee04e79073416c","data":"0x70a08231000000000000000000000000<WALLET_PADDED>"},"latest"],"id":1}'
```

**onchainos command:** Not applicable — read-only eth_call.

---

### Operation: `deposit`

**Description:** Deposit weETH (or WETH) into the ETH Yield Vault (LIQUIDETH). User specifies amount and optionally a minimum shares out for slippage protection. Requires prior ERC-20 approve of the Teller as spender.

**Deposit flow:**
1. Check current allowance: `allowance(wallet, teller_addr)` on weETH contract
2. If insufficient: ERC-20 `approve(teller_addr, amount)` on weETH — selector `0x095ea7b3`
3. Call `Teller.deposit(weETH_addr, amount_wei, minSharesOut=0, receiver=wallet)` — selector `0x8b6099db`

**ABI encoding for `deposit(address,uint256,uint256,address)` — selector `0x8b6099db`:**
- param 1: depositAsset (weETH): 32-byte padded address
- param 2: assets (wei amount): 32-byte uint256
- param 3: minSharesOut (0 for no slippage protection): 32-byte uint256
- param 4: receiver (wallet): 32-byte padded address

**onchainos command (example: deposit 0.00005 weETH):**
```bash
# Step 1: approve weETH to Teller (if needed)
onchainos wallet contract-call \
  --chain 1 \
  --to 0xCd5fE23C85820F7B72D0926FC9b05b43E359b7ee \
  --input-data 0x095ea7b3000000000000000000000000<TELLER_PADDED><AMOUNT_PADDED> \
  --force

# Step 2: deposit
onchainos wallet contract-call \
  --chain 1 \
  --to 0x9AA79C84b79816ab920bBcE20f8f74557B514734 \
  --input-data 0x8b6099db<DEPOSIT_ASSET_PADDED><AMOUNT_PADDED>0000000000000000000000000000000000000000000000000000000000000000<RECEIVER_PADDED> \
  --force
```

**Notes:**
- weETH supports both deposits and withdrawals on the ETH Yield Vault Teller.
- WETH supports deposits but not direct withdrawals (must use weETH to withdraw).
- No ETH value needed (ERC-20 deposit, not payable).
- For test: deposit 0.00005 weETH = 50000000000000 wei. Expected shares ≈ 0.00005012 LIQUIDETH.

---

### Operation: `withdraw`

**Description:** Withdraw weETH from the ETH Yield Vault by burning LIQUIDETH shares via `bulkWithdraw`.

**Withdraw flow:**
1. Check LIQUIDETH balance of wallet
2. Approve LIQUIDETH to Teller if needed (but shares might auto-approve or be handled internally)
3. Call `Teller.bulkWithdraw(weETH_addr, [shares_to_burn], [minWeETHOut=0], [receiver])` — selector `0x8432f02b`

**ABI encoding for `bulkWithdraw(address,uint256[],uint256[],address[])` — selector `0x8432f02b`:**
- param 1: withdrawAsset (weETH): 32-byte padded address
- param 2: shares[] (dynamic array)
- param 3: minAssetsOut[] (dynamic array, set to 0 for no slippage)
- param 4: receivers[] (dynamic array)

**Note on bulkWithdraw:** The function takes arrays to support bulk operations. For a single withdrawal, arrays have length 1.

**ABI encoding for single withdrawal (1 entry):**
```
0x8432f02b
// param1: withdrawAsset
000000000000000000000000Cd5fE23C85820F7B72D0926FC9b05b43E359b7ee
// param2: offset to shares[] = 0x80 (128 bytes from this offset)
0000000000000000000000000000000000000000000000000000000000000080
// param3: offset to minAssetsOut[] = 0xC0 (192 bytes)
00000000000000000000000000000000000000000000000000000000000000c0
// param4: offset to receivers[] = 0x100 (256 bytes)
0000000000000000000000000000000000000000000000000000000000000100
// shares[] length = 1
0000000000000000000000000000000000000000000000000000000000000001
// shares[0] = <SHARES_AMOUNT>
<SHARES_AMOUNT_32BYTES>
// minAssetsOut[] length = 1
0000000000000000000000000000000000000000000000000000000000000001
// minAssetsOut[0] = 0
0000000000000000000000000000000000000000000000000000000000000000
// receivers[] length = 1
0000000000000000000000000000000000000000000000000000000000000001
// receivers[0] = <WALLET>
000000000000000000000000<WALLET_20BYTES>
```

**onchainos command:**
```bash
onchainos wallet contract-call \
  --chain 1 \
  --to 0x9AA79C84b79816ab920bBcE20f8f74557B514734 \
  --input-data <ABI_ENCODED_BULK_WITHDRAW_ABOVE> \
  --force
```

---

### Operation: `rates`

**Description:** Show current exchange rates for each vault (shares per token).

**eth_call for ETH Yield Vault rate (weETH quote):**
```bash
# getRateInQuote(address) = 0x1dcbb110
curl -s -X POST https://ethereum.publicnode.com \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_call","params":[{"to":"0x0d05D94a5F1E76C18fbeB7A13d17C8a314088198","data":"0x1dcbb110000000000000000000000000Cd5fE23C85820F7B72D0926FC9b05b43E359b7ee"},"latest"],"id":1}'
```
Returns: uint256 rate (18 decimals). Divide by 1e18 = weETH per LIQUIDETH share.

**APY source:** DefiLlama yields chart API (same as `vaults` command).

---

## §3 User Scenarios

### Scenario 1: Alice checks available Liquid vaults and rates

1. Alice says: "Show me ether.fi Liquid vaults"
2. Plugin calls `vaults --chain 1`
3. Plugin fetches DefiLlama chart for each pool ID (LIQUIDETH, LIQUIDUSD, LIQUIDBTC)
4. Plugin calls `getRateInQuote(weETH)` on each Accountant for current share price
5. Plugin displays: vault name, accepted tokens, current APY, TVL, share price

---

### Scenario 2: Bob deposits weETH into the ETH Yield Vault

1. Bob says: "Deposit 0.1 weETH into ether.fi Liquid"
2. Plugin calls `positions` to show current LIQUIDETH balance
3. Plugin shows dry-run: calldata preview for approve + deposit
4. **Ask user to confirm** before executing on-chain
5. After confirmation:
   - Step A: approve weETH to Teller: `onchainos wallet contract-call --chain 1 --to <weETH> --input-data 0x095ea7b3<teller_padded><amount_padded> --force`
   - Step B: deposit: `onchainos wallet contract-call --chain 1 --to <teller> --input-data 0x8b6099db<weETH_padded><amount_padded>0000...0000<receiver_padded> --force`
6. Plugin displays: "Deposited 0.1 weETH, received X LIQUIDETH shares. txHash: 0x..."

---

### Scenario 3: Carol withdraws from the ETH Yield Vault

1. Carol says: "Withdraw all my ether.fi Liquid ETH position"
2. Plugin calls `positions` to read LIQUIDETH balance
3. Plugin calculates expected weETH out using current rate
4. Plugin shows dry-run calldata for bulkWithdraw
5. **Ask user to confirm** before executing on-chain
6. After confirmation:
   - Plugin calls `bulkWithdraw(weETH, [shares], [0], [wallet])` via Teller
7. Plugin displays: "Withdrew X LIQUIDETH shares, received Y weETH. txHash: 0x..."

---

### Scenario 4: Dave checks his positions across all vaults

1. Dave says: "Show my ether.fi Liquid positions"
2. Plugin calls `positions --chain 1`
3. Plugin resolves wallet address, reads `balanceOf(wallet)` from each BoringVault
4. Plugin reads rates from each Accountant to calculate USD-equivalent values
5. Plugin displays: table of vault name, shares held, current value in base token

---

## §4 API Dependencies

| API | URL | Purpose | Auth required |
|---|---|---|---|
| DefiLlama Yields Chart | `https://yields.llama.fi/chart/{pool_id}` | Vault APY and TVL history | None |
| Ethereum JSON-RPC | `https://ethereum.publicnode.com` | eth_call for balances, rates, assetData | None |
| onchainos wallet | `onchainos wallet contract-call --chain 1` | Deposit and withdraw on-chain ops | Wallet via onchainos |

**DefiLlama pool IDs:**
- LIQUIDETH (ETH Yield): `b86d4934-2e75-415a-bdd2-e28143d72491`
- LIQUIDUSD (USD Yield): `7c12f175-37bc-41db-967a-1d7f1f4a23c4`
- LIQUIDBTC (BTC Yield): `2f063aed-0a5a-4ba5-8b63-8404b2a99fca`

---

## §5 Configuration Parameters

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `chain_id` | integer | No | `1` | EVM chain ID (1 = Ethereum mainnet) |
| `rpc_url` | string | No | `https://ethereum.publicnode.com` | Ethereum JSON-RPC endpoint |
| `dry_run` | boolean | No | `false` | If true, build calldata but do not broadcast |

**Hardcoded vault registry (in config.rs):**
- ETH Yield Vault: teller=`0x9AA79C84b79816ab920bBcE20f8f74557B514734`, vault=`0xf0bb20865277abd641a307ece5ee04e79073416c`, accountant=`0x0d05D94a5F1E76C18fbeB7A13d17C8a314088198`
- USD Yield Vault: teller=`0x4DE413a26fC24c3FC27Cc983be70aA9c5C299387`, vault=`0x08c6F91e2B681FaF5e17227F2a44C307b3C1364C`, accountant=`0xc315D6e14DDCDC7407784e2Caf815d131Bc1D3E7`
- BTC Yield Vault: teller=`0x8Ea0B382D054dbEBeB1d0aE47ee4AC433C730353`, vault=`0x5f46d540b6eD704C3c8789105F30E075AA900726`, accountant=`0xEa23aC6D7D11f6b181d6B98174D334478ADAe6b0`

---

## §6 Integration Notes & Caveats

### BoringVault is NOT ERC-4626
Despite the mission brief mentioning ERC-4626 selectors (`0x6e553f65` / `0xba087652`), ether.fi Liquid uses Veda's **BoringVault architecture** — not a standard ERC-4626 vault. The deposit entry point is the **Teller** contract with a custom `deposit(address,uint256,uint256,address)` function. Do NOT use ERC-4626 `deposit(uint256,address)` selector `0x6e553f65`.

### Withdrawal flow is synchronous via bulkWithdraw
Unlike some vault architectures that require a withdrawal queue with delays, the Teller's `bulkWithdraw` appears to be instant (the BoringOnChainQueue is a separate optional flow for large withdrawals). For small amounts, `bulkWithdraw` on the Teller directly works.

### Deposit requires ERC-20 approve first
The Teller cannot pull tokens without prior ERC-20 approval. Always check `allowance(wallet, teller)` and approve if insufficient before calling `deposit`.

### weETH is the preferred deposit/withdraw token for ETH vault
On the ETH Yield Vault:
- `assetData(weETH)` returns `allowDeposits=true, allowWithdrawals=true`
- `assetData(WETH)` returns `allowDeposits=true, allowWithdrawals=false`
- Use weETH for both deposit and withdraw paths (we already have weETH from etherfi-stake testing)

### dry_run mode
When `dry_run = true`, log all constructed calldata and parameters to stdout, do NOT execute `onchainos wallet contract-call`.

---

## §7 Selector Verification Summary

| Function | Signature | Selector | Verified via |
|---|---|---|---|
| Teller.deposit | `deposit(address,uint256,uint256,address)` | `0x8b6099db` | `cast sig` |
| Teller.bulkWithdraw | `bulkWithdraw(address,uint256[],uint256[],address[])` | `0x8432f02b` | `cast sig` |
| Teller.assetData | `assetData(address)` | `0x41fee44a` | `cast sig` |
| Teller.vault | `vault()` | `0xfbfa77cf` | `cast sig` |
| Accountant.getRateInQuote | `getRateInQuote(address)` | `0x1dcbb110` | `cast sig` |
| Accountant.getRateInQuoteSafe | `getRateInQuoteSafe(address)` | `0x820973da` | `cast sig` |
| ERC-20.totalSupply | `totalSupply()` | `0x18160ddd` | `cast sig` |
| ERC-20.balanceOf | `balanceOf(address)` | `0x70a08231` | `cast sig` |
| ERC-20.approve | `approve(address,uint256)` | `0x095ea7b3` | `cast sig` |
| ERC-20.allowance | `allowance(address,address)` | `0xdd62ed3e` | `cast sig` |

---

## §8 References

- ether.fi Liquid docs: https://etherfi.gitbook.io/etherfi/liquid
- ether.fi Liquid Technical Docs: https://etherfi.gitbook.io/etherfi/liquid/technical-documentation
- ETH Yield Vault: https://etherfi.gitbook.io/etherfi/liquid/eth-yield-vault
- USD Yield Vault: https://etherfi.gitbook.io/etherfi/liquid/liquid-usd-vault
- BTC Yield Vault: https://etherfi.gitbook.io/etherfi/liquid/liquid-btc-yield-vault
- Veda Architecture: https://docs.veda.tech/architecture-and-flow-of-funds
- Etherscan ETH Vault Teller: https://etherscan.io/address/0x9AA79C84b79816ab920bBcE20f8f74557B514734
- Etherscan LIQUIDETH BoringVault: https://etherscan.io/address/0xf0bb20865277abd641a307ece5ee04e79073416c
- DefiLlama ether.fi-liquid pools: https://yields.llama.fi/pools (filter project=ether.fi-liquid)
