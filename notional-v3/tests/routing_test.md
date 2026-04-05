# Skill Routing Test — notional-v3

## Positive Trigger Tests

| # | User Input | Expected Command | Expected Params | SKILL.md Coverage? | Result |
|---|-----------|-----------------|-----------------|-------------------|--------|
| 1 | "show me notional vaults" | get-vaults | — | ✅ "notional vaults" in description triggers | PASS |
| 2 | "list notional USDC vaults" | get-vaults | --asset USDC | ✅ "notional vaults" + "USDC" | PASS |
| 3 | "what are my notional positions?" | get-positions | — | ✅ "notional positions" in description | PASS |
| 4 | "enter notional leveraged yield vault" | enter-position | --vault, --amount | ✅ "enter notional vault" trigger | PASS |
| 5 | "deposit 0.01 USDC into notional exponent" | enter-position | --amount 0.01 --asset USDC | ✅ "notional leveraged yield" trigger | PASS |
| 6 | "exit my notional position" | exit-position | --vault, --shares | ✅ "exit notional vault" trigger | PASS |
| 7 | "withdraw all shares from notional vault" | exit-position | --shares all | ✅ "exit notional vault" + "shares" | PASS |
| 8 | "claim notional rewards" | claim-rewards | --vault | ✅ "claim notional rewards" trigger | PASS |
| 9 | "initiate notional withdrawal" | initiate-withdraw | --vault, --shares | ✅ "initiate notional withdraw" trigger | PASS |

## Negative Trigger Tests

| # | User Input | Should NOT trigger | Do NOT use rule? | Result |
|---|-----------|-------------------|-----------------|--------|
| 1 | "deposit to Morpho vault" | any command | ✅ Description scoped to Notional/Exponent | PASS |
| 2 | "stake ETH on Lido" | any command | ✅ Different protocol, no trigger match | PASS |
| 3 | "Aave V3 supply USDC" | any command | ✅ No Aave trigger words | PASS |

## Issues Fixed

None — all routing tests passed on first pass.
