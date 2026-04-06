/// ABI encoding utilities for GMX V2 multicall construction

/// Encode a single bytes32 value (already 32 bytes as hex string)
pub fn encode_bytes32(val: &str) -> String {
    let v = val.trim_start_matches("0x");
    format!("{:0>64}", v)
}

/// Encode an address (20 bytes) into 32-byte ABI slot (left-zero-padded)
pub fn encode_address(addr: &str) -> String {
    let a = addr.trim_start_matches("0x");
    format!("{:0>64}", a)
}

/// Encode a uint256 value into 32-byte ABI slot
pub fn encode_u256(val: u128) -> String {
    format!("{:064x}", val)
}

/// Encode a bool into 32-byte ABI slot
pub fn encode_bool(val: bool) -> String {
    if val {
        "0000000000000000000000000000000000000000000000000000000000000001".to_string()
    } else {
        "0000000000000000000000000000000000000000000000000000000000000000".to_string()
    }
}

/// Zero address (32 bytes)
pub fn zero_address() -> String {
    "0000000000000000000000000000000000000000000000000000000000000000".to_string()
}

/// Max uint256
pub fn max_uint256() -> u128 {
    u128::MAX
}

/// Encode `sendWnt(address receiver, uint256 amount)` calldata
/// Selector: 0x7d39aaf1
pub fn encode_send_wnt(receiver: &str, amount: u64) -> String {
    let receiver_padded = encode_address(receiver);
    let amount_padded = encode_u256(amount as u128);
    format!("7d39aaf1{}{}", receiver_padded, amount_padded)
}

/// Encode `sendTokens(address token, address receiver, uint256 amount)` calldata
/// Selector: 0xe6d66ac8
pub fn encode_send_tokens(token: &str, receiver: &str, amount: u128) -> String {
    let token_padded = encode_address(token);
    let receiver_padded = encode_address(receiver);
    let amount_padded = encode_u256(amount);
    format!("e6d66ac8{}{}{}", token_padded, receiver_padded, amount_padded)
}

/// Encode `cancelOrder(bytes32 key)` calldata
/// Selector: 0x7489ec23
pub fn encode_cancel_order(key: &str) -> String {
    let key_clean = key.trim_start_matches("0x");
    let key_padded = format!("{:0>64}", key_clean);
    format!("7489ec23{}", key_padded)
}

/// Encode `claimFundingFees(address[] markets, address[] tokens, address receiver)` calldata
/// Selector: 0xc41b1ab3
pub fn encode_claim_funding_fees(markets: &[&str], tokens: &[&str], receiver: &str) -> String {
    // ABI encoding for dynamic arrays:
    // selector (4 bytes) + offset(markets) + offset(tokens) + offset(receiver_param -> but receiver is address, not dynamic)
    // Actually: claimFundingFees(address[],address[],address)
    // Head: offset to markets array, offset to tokens array, receiver address (padded)
    // Then arrays inline

    let head_size = 3 * 32; // 3 slots in head
    let markets_array_size = (1 + markets.len()) * 32; // length + elements

    let offset_markets = head_size; // 0x60
    let offset_tokens = head_size + markets_array_size;

    let mut out = String::from("c41b1ab3");
    // Head
    out.push_str(&encode_u256(offset_markets as u128));
    out.push_str(&encode_u256(offset_tokens as u128));
    out.push_str(&encode_address(receiver));
    // markets array
    out.push_str(&encode_u256(markets.len() as u128));
    for m in markets {
        out.push_str(&encode_address(m));
    }
    // tokens array
    out.push_str(&encode_u256(tokens.len() as u128));
    for t in tokens {
        out.push_str(&encode_address(t));
    }
    out
}

/// Encode `createOrder(CreateOrderParams)` calldata for GMX V2 (current ABI)
/// Selector: 0xf59c48eb
///
/// Current GMX V2 CreateOrderParams struct:
///   CreateOrderParamsAddresses: (receiver, cancellationReceiver, callbackContract, uiFeeReceiver, market, initialCollateralToken, swapPath[])
///   CreateOrderParamsNumbers:   (sizeDeltaUsd, initialCollateralDeltaAmount, triggerPrice, acceptablePrice, executionFee, callbackGasLimit, minOutputAmount, validFromTime)
///   orderType: uint8  (top-level field)
///   decreasePositionSwapType: uint8  (top-level field)
///   isLong: bool
///   shouldUnwrapNativeToken: bool
///   autoCancel: bool
///   referralCode: bytes32
///   dataList: bytes32[]
///
/// Function signature: createOrder(((address,address,address,address,address,address,address[]),(uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256),uint8,uint8,bool,bool,bool,bytes32,bytes32[]))
/// Note: The whole struct is wrapped as a single tuple argument (hence the extra outer parens).
#[allow(clippy::too_many_arguments)]
pub fn encode_create_order(
    _account: &str,      // unused — account is implicit (msg.sender) in current GMX V2
    receiver: &str,
    market: &str,
    collateral_token: &str,
    order_type: u8,
    size_delta_usd: u128,
    collateral_delta_amount: u128,
    trigger_price: u128,
    acceptable_price: u128,
    execution_fee: u64,
    is_long: bool,
    _src_chain_id: u64,  // unused — srcChainId is set by the contract (block.chainid)
) -> String {
    // ABI layout for createOrder(CreateOrderParams):
    // The function takes ONE argument of struct type (dynamic because of swapPath[] and dataList[]).
    //
    // Struct top-level fields (9 components):
    // [0] addresses_tuple (dynamic)  <- offset
    // [1] numbers_tuple (static)     <- offset (because [0] is dynamic, all subsequent are also offset-relative)
    // [2] orderType uint8            (static, but since tuple is dynamic, placed in tail)
    // ... wait: mixed static/dynamic components in a tuple.
    //
    // Solidity ABI for a tuple with dynamic components:
    // - Head: for each component: if static, its value; if dynamic, an offset to its data in tail
    // - Tail: data for dynamic components
    //
    // In our struct (9 components):
    // [0] addresses (DYNAMIC, has swapPath[])
    // [1] numbers (STATIC, 8 x uint256 = 256 bytes)
    // [2] orderType uint8 (STATIC)
    // [3] decreasePositionSwapType uint8 (STATIC)
    // [4] isLong bool (STATIC)
    // [5] shouldUnwrapNativeToken bool (STATIC)
    // [6] autoCancel bool (STATIC)
    // [7] referralCode bytes32 (STATIC)
    // [8] dataList bytes32[] (DYNAMIC)
    //
    // Head (9 slots x 32 bytes = 288 bytes):
    // [0]: offset to addresses_tuple data
    // [1]: numbers_tuple (inline, 8 x 32 = 256 bytes... NO, wait)
    //
    // Actually in ABI: when a tuple contains BOTH static and dynamic elements,
    // dynamic elements are placed in the tail and referenced by offset.
    // Static elements are placed inline in the head.
    // But numbers_tuple is itself a static tuple (no dynamic elements), so it goes inline.
    //
    // Head layout:
    // slot 0: offset to addresses_tuple (32 bytes, points into tail)
    // slots 1-8: numbers_tuple (8 x uint256 = 256 bytes, inline)
    // slot 9: orderType uint8 (1 slot = 32 bytes)
    // slot 10: decreasePositionSwapType uint8 (1 slot)
    // slot 11: isLong bool (1 slot)
    // slot 12: shouldUnwrapNativeToken bool (1 slot)
    // slot 13: autoCancel bool (1 slot)
    // slot 14: referralCode bytes32 (1 slot)
    // slot 15: offset to dataList (32 bytes, points into tail)
    //
    // Tail layout:
    // [addresses_tuple data]
    // [dataList data]
    //
    // Total head size = 16 * 32 = 512 bytes
    //
    // Addresses tuple structure (dynamic, contains swapPath[]):
    // Head of addresses: 7 slots (6 static addresses + 1 offset to swapPath)
    // Tail of addresses: swapPath length=0
    // Addresses tuple total: 8 * 32 = 256 bytes
    //
    // dataList = [] (empty bytes32 array)
    // dataList encoded: length=0 = 32 bytes

    // Build addresses tuple.
    // Addresses has 6 static address fields + 1 dynamic array (swapPath).
    // In ABI, the offset for swapPath is placed in slot 6 and points to after all 7 head slots.
    // offset = 7 * 32 = 224 bytes (relative to start of this tuple's head).
    let swap_path_offset_in_addr = 7 * 32usize; // 7 head slots * 32 bytes
    let mut addr_tuple = String::new();
    addr_tuple.push_str(&encode_address(receiver));          // receiver
    addr_tuple.push_str(&encode_address(receiver));          // cancellationReceiver = receiver
    addr_tuple.push_str(&zero_address());                    // callbackContract = 0x0
    addr_tuple.push_str(&zero_address());                    // uiFeeReceiver = 0x0
    addr_tuple.push_str(&encode_address(market));            // market
    addr_tuple.push_str(&encode_address(collateral_token));  // initialCollateralToken
    addr_tuple.push_str(&encode_u256(swap_path_offset_in_addr as u128)); // offset to swapPath
    addr_tuple.push_str(&encode_u256(0));                    // swapPath length = 0 (empty array)

    // Numbers tuple (8 static uint256 values)
    let mut num_tuple = String::new();
    num_tuple.push_str(&encode_u256(size_delta_usd));
    num_tuple.push_str(&encode_u256(collateral_delta_amount));
    num_tuple.push_str(&encode_u256(trigger_price));
    num_tuple.push_str(&encode_u256(acceptable_price));
    num_tuple.push_str(&encode_u256(execution_fee as u128));
    num_tuple.push_str(&encode_u256(0)); // callbackGasLimit = 0
    num_tuple.push_str(&encode_u256(0)); // minOutputAmount = 0
    num_tuple.push_str(&encode_u256(0)); // validFromTime = 0

    // Head of the outer struct (16 slots):
    // Head size = 16 * 32 = 512 bytes
    let head_size = 16 * 32usize;
    let addr_tuple_bytes = addr_tuple.len() / 2;

    let offset_addr = head_size; // addresses starts right after head
    let offset_datalist = head_size + addr_tuple_bytes; // dataList starts after addresses

    let mut struct_data = String::new();
    struct_data.push_str(&encode_u256(offset_addr as u128));    // [0] offset to addresses
    struct_data.push_str(&num_tuple);                            // [1-8] numbers inline (256 bytes = 8 slots)
    struct_data.push_str(&encode_u256(order_type as u128));     // [9] orderType
    struct_data.push_str(&encode_u256(0));                       // [10] decreasePositionSwapType = 0
    struct_data.push_str(&encode_bool(is_long));                 // [11] isLong
    struct_data.push_str(&encode_bool(false));                   // [12] shouldUnwrapNativeToken = false
    struct_data.push_str(&encode_bool(false));                   // [13] autoCancel = false
    struct_data.push_str(&encode_u256(0));                       // [14] referralCode = bytes32(0)
    struct_data.push_str(&encode_u256(offset_datalist as u128)); // [15] offset to dataList

    // Tail: addresses tuple data
    struct_data.push_str(&addr_tuple);
    // Tail: dataList (empty bytes32[] = just length=0)
    struct_data.push_str(&encode_u256(0)); // dataList length = 0

    // The function takes one argument: the struct (as a tuple).
    // Since the struct is dynamic, the ABI wraps it with an offset pointer:
    // [selector][offset=0x20][struct_data]
    format!("f59c48eb{}{}", encode_u256(0x20), struct_data)
}

/// Encode `createDeposit(CreateDepositParams)` calldata
/// Selector: 0xadc567e6 (createDeposit((address,address,address,address,address,address[],address[],uint256,uint256,uint256,uint256,uint256)))
/// We use manual ABI encoding for the struct.
#[allow(clippy::too_many_arguments)]
pub fn encode_create_deposit(
    receiver: &str,
    callback_contract: &str,
    ui_fee_receiver: &str,
    market: &str,
    initial_long_token: &str,
    initial_short_token: &str,
    min_market_tokens: u128,
    execution_fee: u64,
    src_chain_id: u64,
) -> String {
    // createDeposit((Addresses, Numbers, Flags))
    // Addresses: (receiver, callbackContract, uiFeeReceiver, market, initialLongToken, initialShortToken, longTokenSwapPath[], shortTokenSwapPath[])
    // Numbers: (minMarketTokens, executionFee, callbackGasLimit, srcChainId)
    // Flags: (shouldUnwrapNativeToken)
    //
    // Selector: let's use the verified one from design
    // The function signature is complex, so we'll build it piece by piece.

    // Addresses tuple (static head + 2 dynamic arrays):
    // 6 static address slots + offset to longSwapPath + offset to shortSwapPath + 2 empty arrays
    let addr_head_slots = 8usize; // 6 addresses + 2 offsets
    let long_swap_offset = addr_head_slots * 32; // offset to longSwapPath within addr tuple
    let short_swap_offset = long_swap_offset + 32; // 32 bytes for length=0 array

    let mut addr_encoded = String::new();
    addr_encoded.push_str(&encode_address(receiver));
    addr_encoded.push_str(&encode_address(callback_contract));
    addr_encoded.push_str(&encode_address(ui_fee_receiver));
    addr_encoded.push_str(&encode_address(market));
    addr_encoded.push_str(&encode_address(initial_long_token));
    addr_encoded.push_str(&encode_address(initial_short_token));
    addr_encoded.push_str(&encode_u256(long_swap_offset as u128));
    addr_encoded.push_str(&encode_u256(short_swap_offset as u128));
    addr_encoded.push_str(&encode_u256(0)); // longSwapPath length=0
    addr_encoded.push_str(&encode_u256(0)); // shortSwapPath length=0

    // Numbers tuple (4 static slots):
    let mut num_encoded = String::new();
    num_encoded.push_str(&encode_u256(min_market_tokens));
    num_encoded.push_str(&encode_u256(execution_fee as u128));
    num_encoded.push_str(&encode_u256(0)); // callbackGasLimit
    num_encoded.push_str(&encode_u256(src_chain_id as u128));

    // Flags tuple (1 bool):
    let mut flags_encoded = String::new();
    flags_encoded.push_str(&encode_bool(false)); // shouldUnwrapNativeToken

    // Build struct encoding
    let addr_bytes = addr_encoded.len() / 2;
    let num_bytes = num_encoded.len() / 2;
    let offset_addr = 3 * 32usize;
    let offset_num = offset_addr + addr_bytes;
    let offset_flags = offset_num + num_bytes;

    let mut struct_encoding = String::new();
    struct_encoding.push_str(&encode_u256(offset_addr as u128));
    struct_encoding.push_str(&encode_u256(offset_num as u128));
    struct_encoding.push_str(&encode_u256(offset_flags as u128));
    struct_encoding.push_str(&addr_encoded);
    struct_encoding.push_str(&num_encoded);
    struct_encoding.push_str(&flags_encoded);

    // Selector for createDeposit
    // createDeposit((address,address,address,address,address,address,address[],address[],uint256,uint256,uint256,uint256,bool))
    // We use: 0xadc567e6
    format!("adc567e6{}{}", encode_u256(0x20), struct_encoding)
}

/// Encode `createWithdrawal(CreateWithdrawalParams)` calldata
/// Selector: 0x9b8eb9e7
#[allow(clippy::too_many_arguments)]
pub fn encode_create_withdrawal(
    receiver: &str,
    callback_contract: &str,
    ui_fee_receiver: &str,
    market: &str,
    min_long_token_amount: u128,
    min_short_token_amount: u128,
    execution_fee: u64,
    src_chain_id: u64,
) -> String {
    // CreateWithdrawalParams: (receiver, callbackContract, uiFeeReceiver, market, longTokenSwapPath[], shortTokenSwapPath[])
    // Numbers: (minLongTokenAmount, minShortTokenAmount, executionFee, callbackGasLimit, srcChainId)
    // Flags: (shouldUnwrapNativeToken)

    // Addresses tuple
    let addr_head_slots = 6usize; // 4 addresses + 2 offsets for swap paths
    let long_swap_offset = addr_head_slots * 32;
    let short_swap_offset = long_swap_offset + 32;

    let mut addr_encoded = String::new();
    addr_encoded.push_str(&encode_address(receiver));
    addr_encoded.push_str(&encode_address(callback_contract));
    addr_encoded.push_str(&encode_address(ui_fee_receiver));
    addr_encoded.push_str(&encode_address(market));
    addr_encoded.push_str(&encode_u256(long_swap_offset as u128));
    addr_encoded.push_str(&encode_u256(short_swap_offset as u128));
    addr_encoded.push_str(&encode_u256(0)); // longSwapPath length=0
    addr_encoded.push_str(&encode_u256(0)); // shortSwapPath length=0

    // Numbers tuple
    let mut num_encoded = String::new();
    num_encoded.push_str(&encode_u256(min_long_token_amount));
    num_encoded.push_str(&encode_u256(min_short_token_amount));
    num_encoded.push_str(&encode_u256(execution_fee as u128));
    num_encoded.push_str(&encode_u256(0)); // callbackGasLimit
    num_encoded.push_str(&encode_u256(src_chain_id as u128));

    // Flags
    let mut flags_encoded = String::new();
    flags_encoded.push_str(&encode_bool(false)); // shouldUnwrapNativeToken

    let addr_bytes = addr_encoded.len() / 2;
    let num_bytes = num_encoded.len() / 2;
    let offset_addr = 3 * 32usize;
    let offset_num = offset_addr + addr_bytes;
    let offset_flags = offset_num + num_bytes;

    let mut struct_encoding = String::new();
    struct_encoding.push_str(&encode_u256(offset_addr as u128));
    struct_encoding.push_str(&encode_u256(offset_num as u128));
    struct_encoding.push_str(&encode_u256(offset_flags as u128));
    struct_encoding.push_str(&addr_encoded);
    struct_encoding.push_str(&num_encoded);
    struct_encoding.push_str(&flags_encoded);

    // Selector for createWithdrawal: 0x9b8eb9e7
    format!("9b8eb9e7{}{}", encode_u256(0x20), struct_encoding)
}

/// Encode the outer `multicall(bytes[])` calldata
/// Selector: 0xac9650d8
pub fn encode_multicall(inner_calls: &[String]) -> String {
    // multicall(bytes[]) — single dynamic array argument
    // Encoding:
    // [selector][offset_to_array=0x20][array_length][offsets_to_each_element][element_data]

    let n = inner_calls.len();

    // Ethereum ABI encoding for bytes[]:
    // [length N][offset[0]][offset[1]]...[offset[N-1]][element0 data][element1 data]...
    //
    // Offsets are relative to the start of the OFFSET AREA (i.e., immediately after the length word).
    // The offset area is n*32 bytes, so the first element starts at n*32 from the offset area start.
    // Each element is encoded as: [32-byte length][data padded to 32-byte boundary].

    let offsets_size = n * 32; // n offset words (no length word in offset base)

    let mut element_offsets: Vec<usize> = Vec::with_capacity(n);
    let mut element_data: Vec<String> = Vec::with_capacity(n);
    let mut current_offset = offsets_size; // first element starts right after all offset words

    for call_hex in inner_calls {
        element_offsets.push(current_offset);
        let data_bytes = call_hex.len() / 2; // hex string → byte length
        // Encode: length (32 bytes) + data (padded to 32-byte boundary)
        let padded_len = (data_bytes + 31) / 32 * 32;
        let padded_hex_len = padded_len * 2;
        let data_padded = format!("{:0<width$}", call_hex, width = padded_hex_len);
        let encoded_element = format!("{}{}", encode_u256(data_bytes as u128), data_padded);
        current_offset += encoded_element.len() / 2;
        element_data.push(encoded_element);
    }

    let mut result = String::from("ac9650d8");
    // Outer offset: points to start of bytes[] encoding = 0x20 (after this offset word)
    result.push_str(&encode_u256(0x20));
    // Array length
    result.push_str(&encode_u256(n as u128));
    // Offsets to each element (relative to start of the offset area = right after length word)
    for &off in &element_offsets {
        result.push_str(&encode_u256(off as u128));
    }
    // Element data
    for ed in &element_data {
        result.push_str(ed);
    }

    result
}

/// Convert a U256 price in 30-decimal GMX precision to a human-readable USD string
pub fn price_from_gmx(price_str: &str) -> f64 {
    let price_u128 = if let Ok(v) = price_str.parse::<u128>() {
        v
    } else {
        return 0.0;
    };
    // Price is in 10^30 units; divide by 10^30
    price_u128 as f64 / 1e30
}

/// Compute acceptable price with slippage
/// long: minPrice * (1 - slippage_bps/10000)
/// short: maxPrice * (1 + slippage_bps/10000)
pub fn compute_acceptable_price(price: u128, is_long: bool, slippage_bps: u32) -> u128 {
    let bps = slippage_bps as u128;
    if is_long {
        price.saturating_sub(price * bps / 10_000)
    } else {
        price + price * bps / 10_000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_address() {
        let addr = "0x1C3fa76e6E1088bCE750f23a5BFcffa1efEF6A41";
        let encoded = encode_address(addr);
        assert_eq!(encoded.len(), 64);
        assert!(encoded.ends_with("1c3fa76e6e1088bce750f23a5bfcffa1efef6a41") || encoded.to_lowercase().ends_with("1c3fa76e6e1088bce750f23a5bfcffa1efef6a41"));
    }

    #[test]
    fn test_encode_u256() {
        let encoded = encode_u256(1000);
        assert_eq!(encoded.len(), 64);
    }

    #[test]
    fn test_price_from_gmx() {
        let price = "1800000000000000000000000000000000"; // 1800 * 10^30
        let usd = price_from_gmx(price);
        assert!((usd - 1800.0).abs() < 1.0);
    }
}
