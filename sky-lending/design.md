# Sky Lending Plugin — Design Document

## Overview

Sky Lending is the rebrand of MakerDAO CDP (Collateralized Debt Position) system. Users deposit collateral (ETH, WBTC, etc.) into vaults and borrow DAI (or USDS) stablecoins against it. The protocol runs on Ethereum mainnet (chain ID 1).

**Key facts:**
- Sky = MakerDAO rebrand; core smart contracts (Vat, DssCdpManager) remain unchanged
- CDP = Collateralized Debt Position; users "lock" collateral to "draw" DAI debt
- Liquidation risk: if collateral value drops below the liquidation ratio, the vault is liquidated
- All write operations in this plugin are dry-run only (safety measure)

## Architecture

### Core Contracts (Ethereum Mainnet)

| Contract | Address | Purpose |
|---|---|---|
| DssCdpManager | `0x5ef30b9986345249bc32d8928B7ee64DE9435E39` | High-level CDP management; tracks cdpId → urn mapping |
| Vat | `0x35D1b3F3D7966A1DFe207aa4514C12a259A0492B` | Core accounting engine; stores all urn (vault) state |
| Jug | `0x19c0976f590D67707E62397C87829d896Dc0f1F` | Stability fee accumulator per ilk |
| ETH-A Join | `0x2F0b23f53734252Bda2277357e97e1517d6B042A` | Adapter to lock ETH-A collateral into Vat |
| DAI Join | `0x9759A6Ac90977b93B58547b4A71c78317f391A28` | Adapter to mint/burn DAI from Vat |
| DAI Token | `0x6B175474E89094C44Da98b954EedeAC495271d0F` | ERC-20 DAI stablecoin |
| MCD Spot | `0x65C79fcB50Ca1594B025960e539eD7A9a6D434A7` | Stores spot (oracle) prices per ilk |

### Key Concepts

- **ilk**: collateral type identifier (bytes32), e.g., `ETH-A`, `WBTC-A`
- **urn**: per-user vault address within the Vat for a given ilk
- **cdpId**: user-friendly integer CDP ID (from DssCdpManager), maps to urn
- **ink**: collateral amount locked in a vault (ray = 1e27 units)
- **art**: normalized debt in a vault (multiply by ilk.rate to get actual DAI debt)
- **rate**: stability fee accumulator; actual DAI debt = art * rate
- **spot**: liquidation price per unit of collateral (spot = price / liquidation_ratio)
- **line**: max total DAI debt allowed for an ilk
- **dust**: minimum DAI debt for a vault

### CDP Operation Flow (via DSProxy)

MakerDAO uses a proxy pattern: each user has a DSProxy contract, and operations are delegated through it to DssProxyActions. However, for simplicity this plugin implements direct Vat/Join interaction calldata (educational/informational level) with dry-run mode only.

For real operations in production, users would need a DSProxy deployed.

## Operations

### Read Operations

1. **`ilks`** — list collateral types
   - Call `Vat.ilks(bytes32 ilk)` for known ilks (ETH-A, WBTC-A, USDC-A, WSTETH-A)
   - Returns: Art (total debt), rate (stability fee accumulator), spot (liquidation price), line (debt ceiling), dust (minimum debt)
   - Also calls `Jug.ilks(bytes32 ilk)` for stability fee (duty) per ilk

2. **`vaults`** — list user's CDP vaults
   - Call `DssCdpManager.first(address owner)` → first cdpId
   - Walk linked list via `DssCdpManager.list(uint256 cdpId)` → (prev, next)
   - For each cdpId: call `DssCdpManager.urns(uint256 cdpId)` → urn address
   - Call `DssCdpManager.ilks(uint256 cdpId)` → ilk bytes32
   - Call `Vat.urns(bytes32 ilk, address urn)` → (ink, art)
   - Compute: collateral_amount = ink/1e18, dai_debt = art * rate / 1e27 / 1e18

### Write Operations (All Dry-Run Only)

3. **`open-vault`** — open new CDP vault
   - Target: DssCdpManager.open(bytes32 ilk, address usr)
   - Selector: `0x6090dec5` (open(bytes32,address))
   - Dry-run: show calldata, do not broadcast

4. **`deposit-collateral`** — lock ETH collateral
   - Target: ETH-A Join adapter
   - Call `join(address urn, uint256 wad)` with ETH value
   - Selector: `0x3b4da69f` (join(address,uint256)) — but ETH-A join is `join(address)` with msg.value
   - Actually ETH-A GemJoin: `join(address usr)` payable with ETH value
   - Dry-run: show calldata + amount

5. **`draw-dai`** — mint DAI against collateral
   - Target: Vat.frob(bytes32 ilk, address u, address v, address w, int256 dink, int256 dart)
   - Then: DaiJoin.exit(address usr, uint256 wad)
   - Dry-run: show calldata

6. **`repay-dai`** — burn DAI to reduce debt
   - Target: DAI.approve + DaiJoin.join(address usr, uint256 wad) + Vat.frob(... negative dart)
   - Dry-run: show calldata

7. **`withdraw-collateral`** — free collateral from vault
   - Target: Vat.frob(... negative dink) + ETH-A Join.exit(address usr, uint256 wad)
   - Dry-run: show calldata

## ABI Selectors

```
DssCdpManager:
  open(bytes32,address)         = 0x6090dec5
  urns(uint256)                 = 0x2424be5c (returns address urn)
  ilks(uint256)                 = 0x1d3e6f76 (returns bytes32 ilk)
  owns(uint256)                 = 0x43830a26 (returns address owner)
  first(address)                = 0x3ae451db (returns uint256 cdpId)
  count(address)                = 0x7254fb8b (returns uint256 count)
  list(uint256)                 = 0x74b9a629 (returns (uint256 prev, uint256 next))
  cdpi()                        = 0xf6de1f80 (returns uint256 last cdpId)

Vat:
  urns(bytes32,address)         = 0x35b29077 (returns (uint256 ink, uint256 art))
  ilks(bytes32)                 = 0x129ae9a0 (returns (uint256 Art, uint256 rate, uint256 spot, uint256 line, uint256 dust))
  frob(bytes32,address,address,address,int256,int256) = 0x76088703

Jug:
  ilks(bytes32)                 = 0xd9c82e86 (returns (uint256 duty, uint256 rho))
  base()                        = 0x5001f3b5 (returns uint256)

EthJoin (ETH-A):
  join(address)                 = 0xf04dba69 (payable, locks ETH)
  exit(address,uint256)         = 0xef693bed

DaiJoin:
  join(address,uint256)         = 0xef693bed
  exit(address,uint256)         = 0x7f8661a1

DAI (ERC-20):
  balanceOf(address)            = 0x70a08231
  approve(address,uint256)      = 0x095ea7b3
```

## Known Ilks (bytes32 encoding)

| Ilk Name | bytes32 (hex, right-padded) |
|---|---|
| ETH-A | `0x4554482d410000000000000000000000000000000000000000000000000000` |
| WBTC-A | `0x574254432d41000000000000000000000000000000000000000000000000000` |
| USDC-A | `0x555344432d410000000000000000000000000000000000000000000000000000` |
| WSTETH-A | `0x575354455448412d000000000000000000000000000000000000000000000000` |

## Risk Warnings

- CDP operations carry liquidation risk if collateral value falls below the liquidation ratio
- Stability fees accrue continuously and increase debt over time
- All write operations in this plugin are dry-run only for safety
- Users must have a DSProxy deployed to use write operations in production

## Implementation Approach

- Read operations: direct eth_call via JSON-RPC to publicnode.com
- Write operations: construct calldata and display in dry-run mode (no broadcast)
- No external API dependencies needed (all on-chain data)
- Wallet resolution: `onchainos wallet addresses` → EVM list → chain 1
