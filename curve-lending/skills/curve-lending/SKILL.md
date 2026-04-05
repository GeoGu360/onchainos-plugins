---
name: curve-lending
version: 0.1.0
description: Curve Lending (LlamaLend) — isolated lending markets using crvUSD. Deposit ETH/wstETH/tBTC as collateral and borrow crvUSD, or check lending rates.
binary: curve-lending
---

# Curve Lending Skill

Curve Lending (also known as LlamaLend) is Curve Finance's isolated lending product. Users deposit collateral (WETH, wstETH, tBTC, etc.) to borrow crvUSD, or lend crvUSD to earn yield. Each market is isolated with its own LLAMMA AMM and Controller contract.

**Chain:** Ethereum mainnet (chain ID: 1)
**Borrow token:** crvUSD (`0xf939E0A03FB07F59A73314E73794Be0E57ac1b4e`)
**Factory:** `0xeA6876DDE9e3467564acBeE1Ed5bac88783205E0` (46 markets)

> **Note:** We don't hold crvUSD in the test wallet. Read operations (markets, rates) and collateral deposit dry-runs are fully supported. Borrow and repay are dry-run only.

---

## Pre-flight Checks

1. Verify `curve-lending` binary is installed: `curve-lending --help`
2. Verify `onchainos` is available: `onchainos wallet addresses`
3. Confirm active wallet on Ethereum (chain 1): `onchainos wallet balance --chain 1`

---

## Commands

### markets — List Lending Markets

**Triggers:** "show curve lending markets", "list crvUSD lending pools", "what are the Curve lending markets", "curve llamalend markets"

**Usage:**
```
curve-lending markets [--chain 1] [--limit 20]
```

**Parameters:**
- `--chain` (optional): EVM chain ID, default 1
- `--limit` (optional): max markets to show, default 20

**Example:**
```
curve-lending markets --chain 1 --limit 10
```

**Output:** JSON with market name, controller address, vault, collateral symbol, total supply, active loans, total debt.

---

### rates — View Borrow/Lend APY Rates

**Triggers:** "what's the borrow rate on curve lending", "show WETH lending APY", "curve lending interest rates", "how much does it cost to borrow crvUSD"

**Usage:**
```
curve-lending rates [--chain 1] [--market <name_or_index>]
```

**Parameters:**
- `--chain` (optional): EVM chain ID, default 1
- `--market` (optional): market name substring (e.g. "WETH-long") or index

**Examples:**
```
curve-lending rates --chain 1
curve-lending rates --chain 1 --market WETH-long
curve-lending rates --chain 1 --market 1
```

**Output:** JSON with borrow APY, lend APY, min/max APY range, total supply, total debt, utilization.

---

### positions — View Active Positions

**Triggers:** "show my curve lending positions", "do I have any loans on curve", "what's my debt on curve llamalend", "my crvUSD borrowing positions"

**Usage:**
```
curve-lending positions [--chain 1] [--address <addr>] [--market <name>]
```

**Parameters:**
- `--chain` (optional): EVM chain ID, default 1
- `--address` (optional): wallet address (resolved from onchainos if not provided)
- `--market` (optional): narrow to a specific market

**Example:**
```
curve-lending positions --chain 1
```

**Output:** JSON with active positions: collateral amount, debt (crvUSD), LLAMMA band range, liquidation prices, health factor.

---

### deposit-collateral — Deposit Collateral

**Triggers:** "deposit WETH to curve lending", "add collateral to curve llamalend", "put wstETH as collateral on curve"

**Usage:**
```
curve-lending deposit-collateral --market <name> --amount <amount> [--chain 1] [--dry-run] [--bands 10]
```

**Parameters:**
- `--market` (required): market name (e.g. "WETH-long") or index
- `--amount` (required): collateral amount in token units (e.g. 0.001)
- `--chain` (optional): EVM chain ID, default 1
- `--dry-run` (optional): preview without executing
- `--bands` (optional): number of LLAMMA bands (4-50), default 10

**Examples:**
```
curve-lending deposit-collateral --market WETH-long --amount 0.001 --chain 1 --dry-run
curve-lending deposit-collateral --market WETH-long --amount 0.001 --chain 1
```

**Execution flow:**
1. Check if loan exists for wallet
2. If no loan: calls `create_loan(collateral, min_debt, N)` after ERC-20 approve
3. If loan exists: calls `add_collateral(collateral, wallet)` after ERC-20 approve
4. Both steps go via `onchainos wallet contract-call`

> **Ask user to confirm** before executing the real transaction. Always run `--dry-run` first to preview calldata.
> Write operations are submitted via `onchainos wallet contract-call --chain 1`.

---

### borrow — Borrow crvUSD

**Triggers:** "borrow crvUSD from curve lending", "take a loan on curve llamalend", "borrow against my WETH on curve"

**Usage:**
```
curve-lending borrow --market <name> --amount <crvUSD_amount> [--collateral <amount>] [--chain 1] [--dry-run] [--bands 10]
```

**Parameters:**
- `--market` (required): market name or index
- `--amount` (required): crvUSD amount to borrow
- `--collateral` (optional): additional collateral to deposit simultaneously
- `--chain` (optional): chain ID, default 1
- `--dry-run` (optional): preview calldata without broadcasting

**Examples:**
```
curve-lending borrow --market WETH-long --amount 100 --collateral 0.05 --chain 1 --dry-run
curve-lending borrow --market WETH-long --amount 100 --chain 1 --dry-run
```

> **Ask user to confirm** before executing the real transaction. Borrow operations carry liquidation risk if health factor drops below 0.
> Write operations are submitted via `onchainos wallet contract-call --chain 1`.

---

### repay — Repay crvUSD Debt

**Triggers:** "repay my curve lending debt", "pay back crvUSD to curve", "close my curve loan", "repay llamalend"

**Usage:**
```
curve-lending repay --market <name> --amount <crvUSD_amount> [--chain 1] [--dry-run]
```

**Parameters:**
- `--market` (required): market name or index
- `--amount` (required): crvUSD to repay (use 0 for full repay using wallet crvUSD balance)
- `--chain` (optional): chain ID, default 1
- `--dry-run` (optional): preview calldata without broadcasting

**Examples:**
```
curve-lending repay --market WETH-long --amount 0 --chain 1 --dry-run
curve-lending repay --market WETH-long --amount 100 --chain 1 --dry-run
```

> **Ask user to confirm** before executing the real transaction. Repay requires holding crvUSD.
> Write operations are submitted via `onchainos wallet contract-call --chain 1`.

---

## Routing Rules

- For Curve DEX swaps (not lending), use the `curve` skill
- For Aave/Compound lending, use `aave` or `compound-v3` skills
- Curve Lending is isolated lending using LLAMMA AMM — distinct from Curve DEX pools
- Collateral tokens: WETH, wstETH, tBTC, CRV, sfrxETH (check `markets` for full list)

---

## Error Handling

| Error | Cause | Fix |
|-------|-------|-----|
| "Market not found" | Market name doesn't match | Run `markets` first to see valid names |
| "No active loan and no collateral" | Borrow without collateral on first loan | Add `--collateral <amount>` |
| "Loan doesn't exist" | Repay/positions on wallet with no loan | Check `positions` first |
| "eth_call error" | RPC issue | Retry; publicnode.com is default |
