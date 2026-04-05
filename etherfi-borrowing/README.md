# EtherFi Borrowing (Cash) Plugin

Plugin for EtherFi Cash borrowing protocol on Scroll. Supply USDC liquidity to earn yield, query rates and positions, and repay debt.

## Commands

- `markets` — List supported borrow/collateral tokens with LTV parameters
- `rates` — Show current borrowing APY and liquidity stats
- `position --user-safe <ADDR>` — Show a UserSafe position
- `supply-liquidity --amount <AMOUNT>` — Supply USDC to earn yield
- `withdraw-liquidity --amount <AMOUNT>` — Withdraw supplied USDC
- `repay --user-safe <ADDR> --amount <AMOUNT>` — Repay USDC debt

## Network

All operations run on **Scroll** (chain ID 534352).

## Contracts

| Contract | Address |
|---|---|
| DebtManagerProxy | `0x8f9d2Cd33551CE06dD0564Ba147513F715c2F4a0` |
| USDC (Scroll) | `0x06eFdBFf2a14a7c8E15944D1F4A48F9F95F663A4` |
| weETH (Scroll) | `0x01f0a31698C4d065659b9bdC21B3610292a1c506` |
