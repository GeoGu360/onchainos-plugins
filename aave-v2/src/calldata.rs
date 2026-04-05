use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolCall};
use anyhow::Context;

// Aave V2 LendingPool function signatures.
// NOTE: V2 uses `deposit` not `supply` — selectors are different from V3.
//
// Selector verification (keccak256 first 4 bytes):
//   deposit(address,uint256,address,uint16)         => 0xe8eda9df
//   withdraw(address,uint256,address)               => 0x69328dec
//   borrow(address,uint256,uint256,uint16,address)  => 0xa415bcad
//   repay(address,uint256,uint256,address)          => 0x573ade81
//   approve(address,uint256)                        => 0x095ea7b3

sol! {
    /// Aave V2: deposit (replaces V3's supply)
    function deposit(
        address asset,
        uint256 amount,
        address onBehalfOf,
        uint16 referralCode
    ) external;

    /// Same signature as V3
    function withdraw(
        address asset,
        uint256 amount,
        address to
    ) external returns (uint256);

    /// V2 borrow — same signature as V3
    function borrow(
        address asset,
        uint256 amount,
        uint256 interestRateMode,
        uint16 referralCode,
        address onBehalfOf
    ) external;

    /// V2 repay — same signature as V3
    function repay(
        address asset,
        uint256 amount,
        uint256 rateMode,
        address onBehalfOf
    ) external returns (uint256);

    /// ERC-20 approve
    function approve(
        address spender,
        uint256 amount
    ) external returns (bool);
}

fn parse_address(addr: &str) -> anyhow::Result<Address> {
    addr.parse::<Address>()
        .with_context(|| format!("Invalid address: {}", addr))
}

/// Encode LendingPool.deposit() calldata (Aave V2).
/// Selector: 0xe8eda9df — DIFFERENT from V3's supply (0x617ba037).
pub fn encode_deposit(asset: &str, amount: u128, on_behalf_of: &str) -> anyhow::Result<String> {
    let call = depositCall {
        asset: parse_address(asset)?,
        amount: U256::from(amount),
        onBehalfOf: parse_address(on_behalf_of)?,
        referralCode: crate::config::REFERRAL_CODE,
    };
    let encoded = call.abi_encode();
    Ok(format!("0x{}", hex::encode(encoded)))
}

/// Encode LendingPool.withdraw() calldata.
/// Pass u128::MAX for full withdrawal (maps to type(uint256).max).
pub fn encode_withdraw(asset: &str, amount: u128, to: &str) -> anyhow::Result<String> {
    let amount_u256 = if amount == u128::MAX {
        U256::MAX
    } else {
        U256::from(amount)
    };
    let call = withdrawCall {
        asset: parse_address(asset)?,
        amount: amount_u256,
        to: parse_address(to)?,
    };
    let encoded = call.abi_encode();
    Ok(format!("0x{}", hex::encode(encoded)))
}

/// Encode LendingPool.borrow() calldata.
/// V2 supports stable (1) and variable (2) rate modes.
pub fn encode_borrow(
    asset: &str,
    amount: u128,
    interest_rate_mode: u128,
    on_behalf_of: &str,
) -> anyhow::Result<String> {
    let call = borrowCall {
        asset: parse_address(asset)?,
        amount: U256::from(amount),
        interestRateMode: U256::from(interest_rate_mode),
        referralCode: crate::config::REFERRAL_CODE,
        onBehalfOf: parse_address(on_behalf_of)?,
    };
    let encoded = call.abi_encode();
    Ok(format!("0x{}", hex::encode(encoded)))
}

/// Encode LendingPool.repay() calldata.
/// Pass u128::MAX for full repay (maps to type(uint256).max in Solidity).
pub fn encode_repay(
    asset: &str,
    amount: u128,
    rate_mode: u128,
    on_behalf_of: &str,
) -> anyhow::Result<String> {
    let amount_u256 = if amount == u128::MAX {
        U256::MAX
    } else {
        U256::from(amount)
    };
    let call = repayCall {
        asset: parse_address(asset)?,
        amount: amount_u256,
        rateMode: U256::from(rate_mode),
        onBehalfOf: parse_address(on_behalf_of)?,
    };
    let encoded = call.abi_encode();
    Ok(format!("0x{}", hex::encode(encoded)))
}

/// Encode ERC-20 approve() calldata.
/// Pass u128::MAX for unlimited approval (type(uint256).max).
pub fn encode_erc20_approve(spender: &str, amount: u128) -> anyhow::Result<String> {
    let amount_u256 = if amount == u128::MAX {
        U256::MAX
    } else {
        U256::from(amount)
    };
    let call = approveCall {
        spender: parse_address(spender)?,
        amount: amount_u256,
    };
    let encoded = call.abi_encode();
    Ok(format!("0x{}", hex::encode(encoded)))
}
