// src/abi.rs — ABI encoding for INIT Capital MoneyMarketHook.execute()
//
// MoneyMarketHook.execute(OperationParams) selector: 0x247d4981
//
// OperationParams struct:
//   (uint256 posId, address viewer, uint16 mode,
//    DepositParams[] depositParams,
//    WithdrawParams[] withdrawParams,
//    BorrowParams[] borrowParams,
//    RepayParams[] repayParams,
//    uint256 minHealth_e18, bool returnNative)
//
// DepositParams: (address pool, uint256 amt, (address helper, address tokenIn))
// WithdrawParams: (address pool, uint256 shares, (address helper, address tokenIn), address to)
// BorrowParams: (address pool, uint256 amt, address to)
// RepayParams: (address pool, uint256 shares)
//
// ABI encoding uses dynamic tuple encoding. Since this struct contains arrays,
// the top-level tuple is encoded with offsets for dynamic fields.

#[derive(Debug, Clone)]
pub struct DepositParams {
    pub pool: String,
    pub amt: u128,
}

#[derive(Debug, Clone)]
pub struct WithdrawParams {
    pub pool: String,
    pub shares: u128,
    pub to: String,
}

#[derive(Debug, Clone)]
pub struct BorrowParams {
    pub pool: String,
    pub amt: u128,
    pub to: String,
}

#[derive(Debug, Clone)]
pub struct RepayParams {
    pub pool: String,
    pub shares: u128,
}

/// Zero address constant
const ZERO_ADDR: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn pad_addr(addr: &str) -> String {
    format!("{:0>64}", addr.trim_start_matches("0x").to_lowercase())
}

fn pad_u128(val: u128) -> String {
    format!("{:064x}", val)
}

fn pad_u64(val: u64) -> String {
    format!("{:064x}", val)
}

fn pad_bool(val: bool) -> String {
    format!("{:064x}", if val { 1u32 } else { 0u32 })
}

/// Encode a single DepositParams element as a fixed tuple (3 static slots + nested static tuple)
/// Layout: pool(addr=32) | amt(uint256=32) | helper(addr=32) | tokenIn(addr=32) = 128 bytes
fn encode_deposit_params(p: &DepositParams) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(hex::decode(pad_addr(&p.pool)).unwrap());
    bytes.extend(hex::decode(pad_u128(p.amt)).unwrap());
    // RebaseHelperParams: helper=address(0), tokenIn=address(0)
    bytes.extend(hex::decode(ZERO_ADDR).unwrap()); // helper
    bytes.extend(hex::decode(ZERO_ADDR).unwrap()); // tokenIn
    bytes
}

/// Encode a single WithdrawParams element as a fixed tuple (5 slots = 160 bytes)
/// Layout: pool(32) | shares(32) | helper(32) | tokenIn(32) | to(32)
fn encode_withdraw_params(p: &WithdrawParams) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(hex::decode(pad_addr(&p.pool)).unwrap());
    bytes.extend(hex::decode(pad_u128(p.shares)).unwrap());
    // RebaseHelperParams: helper=address(0), tokenIn=address(0)
    bytes.extend(hex::decode(ZERO_ADDR).unwrap()); // helper
    bytes.extend(hex::decode(ZERO_ADDR).unwrap()); // tokenIn
    bytes.extend(hex::decode(pad_addr(&p.to)).unwrap());
    bytes
}

/// Encode a single BorrowParams element (3 slots = 96 bytes)
/// Layout: pool(32) | amt(32) | to(32)
fn encode_borrow_params(p: &BorrowParams) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(hex::decode(pad_addr(&p.pool)).unwrap());
    bytes.extend(hex::decode(pad_u128(p.amt)).unwrap());
    bytes.extend(hex::decode(pad_addr(&p.to)).unwrap());
    bytes
}

/// Encode a single RepayParams element (2 slots = 64 bytes)
/// Layout: pool(32) | shares(32)
fn encode_repay_params(p: &RepayParams) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(hex::decode(pad_addr(&p.pool)).unwrap());
    bytes.extend(hex::decode(pad_u128(p.shares)).unwrap());
    bytes
}

/// Encode a dynamic array of fixed-size elements
/// Returns: length(32) | element0 | element1 | ...
fn encode_array_of_fixed(elements: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = Vec::new();
    // Array length
    bytes.extend(hex::decode(pad_u64(elements.len() as u64)).unwrap());
    for elem in elements {
        bytes.extend(elem);
    }
    bytes
}

/// Build the calldata for MoneyMarketHook.execute(OperationParams)
///
/// Selector: 0x247d4981
///
/// The OperationParams is a tuple with 9 fields. Fields 4-7 are dynamic arrays.
/// ABI encoding for a top-level tuple with dynamic fields uses offsets.
///
/// Static fields: posId(32) viewer(32) mode(32) [offset_deposits(32)] [offset_withdraws(32)]
///                [offset_borrows(32)] [offset_repays(32)] minHealth(32) returnNative(32)
/// Then the dynamic array data follows.
pub fn encode_execute(
    pos_id: u64,
    viewer: &str,
    mode: u16,
    deposit_params: &[DepositParams],
    withdraw_params: &[WithdrawParams],
    borrow_params: &[BorrowParams],
    repay_params: &[RepayParams],
    min_health_e18: u128,
    return_native: bool,
) -> String {
    let selector = "247d4981";

    // Encode each array's data section
    let deposit_data = encode_array_of_fixed(
        &deposit_params.iter().map(encode_deposit_params).collect::<Vec<_>>()
    );
    let withdraw_data = encode_array_of_fixed(
        &withdraw_params.iter().map(encode_withdraw_params).collect::<Vec<_>>()
    );
    let borrow_data = encode_array_of_fixed(
        &borrow_params.iter().map(encode_borrow_params).collect::<Vec<_>>()
    );
    let repay_data = encode_array_of_fixed(
        &repay_params.iter().map(encode_repay_params).collect::<Vec<_>>()
    );

    // The tuple is the direct calldata argument (not wrapped in another dynamic offset)
    // Static head is 9 slots × 32 bytes = 288 bytes (0x120)
    // Slots: posId viewer mode offset_dep offset_wdraw offset_bor offset_rep minHealth returnNative
    let head_size: u64 = 9 * 32;

    // Offsets are relative to the start of the tuple data (after function selector)
    let offset_deposits: u64 = head_size; // starts right after head
    let offset_withdraws: u64 = offset_deposits + deposit_data.len() as u64;
    let offset_borrows: u64 = offset_withdraws + withdraw_data.len() as u64;
    let offset_repays: u64 = offset_borrows + borrow_data.len() as u64;

    let mut calldata = String::from("0x");
    calldata.push_str(selector);

    // Static head
    calldata.push_str(&pad_u64(pos_id));                   // posId
    calldata.push_str(&pad_addr(viewer));                  // viewer
    calldata.push_str(&format!("{:064x}", mode as u64));   // mode (uint16 padded)
    calldata.push_str(&pad_u64(offset_deposits));          // offset to depositParams
    calldata.push_str(&pad_u64(offset_withdraws));         // offset to withdrawParams
    calldata.push_str(&pad_u64(offset_borrows));           // offset to borrowParams
    calldata.push_str(&pad_u64(offset_repays));            // offset to repayParams
    calldata.push_str(&pad_u128(min_health_e18));          // minHealth_e18
    calldata.push_str(&pad_bool(return_native));           // returnNative

    // Dynamic data sections
    calldata.push_str(&hex::encode(&deposit_data));
    calldata.push_str(&hex::encode(&withdraw_data));
    calldata.push_str(&hex::encode(&borrow_data));
    calldata.push_str(&hex::encode(&repay_data));

    calldata
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_execute_supply_only() {
        let calldata = encode_execute(
            0,
            "0x0000000000000000000000000000000000000000",
            1,
            &[DepositParams {
                pool: "0xD20989EB39348994AA99F686bb4554090d0C09F3".to_string(),
                amt: 10_000_000_000_000_000, // 0.01 WETH
            }],
            &[],
            &[],
            &[],
            0,
            false,
        );
        // Must start with selector
        assert!(calldata.starts_with("0x247d4981"), "wrong selector: {}", &calldata[..10]);
        // Must be non-trivial length
        assert!(calldata.len() > 100);
    }

    #[test]
    fn test_encode_execute_empty() {
        let calldata = encode_execute(
            1,
            "0x87fb0647faabea33113eaf1d80d67acb1c491b90",
            1,
            &[],
            &[],
            &[],
            &[],
            0,
            false,
        );
        assert!(calldata.starts_with("0x247d4981"));
    }
}
