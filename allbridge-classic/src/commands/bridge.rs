use anyhow::{anyhow, Result};
use clap::Args;
use rand::Rng;

use crate::api;
use crate::onchainos;

/// Bridge contract address (same on all EVM chains)
const BRIDGE_CONTRACT: &str = "0xBBbD1BbB4f9b936C3604906D7592A644071dE884";

/// lock(uint128,address,bytes32,bytes4,uint256) selector = 0x7bacc91e
const LOCK_SELECTOR: &str = "7bacc91e";

/// Blockchain ID 4-byte constants (UTF8, zero-padded to 4 bytes)
fn chain_id_to_blockchain_id(chain_id: u64) -> Option<&'static str> {
    match chain_id {
        1 => Some("ETH"),
        56 => Some("BSC"),
        137 => Some("POL"),
        43114 => Some("AVA"),
        250 => Some("FTM"),
        42220 => Some("CELO"),
        _ => None,
    }
}

/// Convert chain name string to 4-byte blockchain ID hex (e.g. "SOL" -> "534f4c00")
fn blockchain_id_to_hex(chain_name: &str) -> Result<String> {
    let upper = chain_name.to_uppercase();
    let name_map = [
        ("ETH", "45544800"),
        ("BSC", "42534300"),
        ("POL", "504f4c00"),
        ("AVA", "41564100"),
        ("SOL", "534f4c00"),
        ("FTM", "46544d00"),
        ("CELO", "43454c4f"),
    ];
    for (name, hex) in &name_map {
        if upper == *name {
            return Ok(hex.to_string());
        }
    }
    Err(anyhow!("Unsupported destination chain: {}. Supported: ETH, BSC, POL, AVA, SOL, FTM, CELO", chain_name))
}

/// Generate a random 16-byte lock ID with first byte = 0x01
fn generate_lock_id() -> u128 {
    let mut rng = rand::thread_rng();
    // First byte must be 0x01 (bridge version)
    // Generate 15 random bytes
    let random_bytes: Vec<u8> = (0..15).map(|_| rng.gen::<u8>()).collect();
    // Build u128: first byte 0x01, then 15 random bytes
    let mut bytes = [0u8; 16];
    bytes[0] = 0x01;
    bytes[1..16].copy_from_slice(&random_bytes);
    u128::from_be_bytes(bytes)
}

/// Convert an EVM address (0x...) to 32-byte hex padded on the right
fn evm_address_to_bytes32(addr: &str) -> Result<String> {
    let clean = addr.trim_start_matches("0x").to_lowercase();
    if clean.len() != 40 {
        return Err(anyhow!("Invalid EVM address: {}", addr));
    }
    // EVM addresses are 20 bytes; pad to 32 bytes with trailing zeros
    Ok(format!("{:0<64}", clean))
}

/// Convert a Solana base58 address to 32-byte hex
fn solana_address_to_bytes32(addr: &str) -> Result<String> {
    let bytes = bs58::decode(addr).into_vec()
        .map_err(|e| anyhow!("Invalid Solana address {}: {}", addr, e))?;
    if bytes.len() != 32 {
        return Err(anyhow!("Solana address must decode to 32 bytes, got {}", bytes.len()));
    }
    Ok(hex::encode(bytes))
}

/// Encode recipient address to 32-byte hex for bridge
/// dest_chain: destination blockchain ID (e.g. "SOL", "ETH", "BSC")
fn encode_recipient(recipient: &str, dest_chain: &str) -> Result<String> {
    let upper = dest_chain.to_uppercase();
    match upper.as_str() {
        "SOL" => solana_address_to_bytes32(recipient),
        "ETH" | "BSC" | "POL" | "AVA" | "FTM" | "CELO" => evm_address_to_bytes32(recipient),
        _ => Err(anyhow!("Cannot encode recipient for unknown chain: {}", dest_chain)),
    }
}

/// Encode lock() calldata
/// lock(uint128 lockId, address tokenAddress, bytes32 recipient, bytes4 destination, uint256 amount)
/// selector: 0x7bacc91e
fn encode_lock_calldata(
    lock_id: u128,
    token_address: &str,
    recipient_bytes32: &str,
    destination_hex: &str, // 8 hex chars (4 bytes), no 0x prefix
    amount: u128,
) -> Result<String> {
    // ABI encode: each slot is 32 bytes
    // uint128 lockId -> padded to 32 bytes (left pad)
    let lock_id_hex = format!("{:064x}", lock_id);
    // address tokenAddress -> padded to 32 bytes (left pad)
    let token_clean = token_address.trim_start_matches("0x").to_lowercase();
    let token_padded = format!("{:0>64}", token_clean);
    // bytes32 recipient -> exactly 32 bytes (already 64 hex chars)
    let recipient_clean = recipient_bytes32.trim_start_matches("0x");
    if recipient_clean.len() != 64 {
        return Err(anyhow!("recipient_bytes32 must be 32 bytes (64 hex chars), got {}", recipient_clean.len()));
    }
    // bytes4 destination -> padded to 32 bytes (left pad, since bytes4 is right-aligned in ABI)
    // bytes4 in ABI encoding: stored in the high bytes of the slot (left-aligned)
    let dest_padded = format!("{:0<64}", destination_hex);
    // uint256 amount -> padded to 32 bytes
    let amount_hex = format!("{:064x}", amount);

    Ok(format!("0x{}{}{}{}{}{}", LOCK_SELECTOR, lock_id_hex, token_padded, recipient_clean, dest_padded, amount_hex))
}

#[derive(Args)]
pub struct BridgeArgs {
    /// Source chain ID (1=Ethereum, 56=BSC, 137=Polygon, 43114=Avalanche)
    #[arg(long)]
    pub chain: u64,

    /// Token symbol to bridge (e.g. USDT, USDC, BUSD)
    #[arg(long)]
    pub token: String,

    /// Amount to bridge (in token units, e.g. 10.0 for 10 USDT)
    #[arg(long)]
    pub amount: f64,

    /// Destination chain (ETH, BSC, POL, AVA, SOL, FTM)
    #[arg(long)]
    pub dest_chain: String,

    /// Recipient address on destination chain
    #[arg(long)]
    pub recipient: String,

    /// Skip on-chain broadcast (preview calldata only)
    #[arg(long, default_value = "false")]
    pub dry_run: bool,
}

pub async fn run(args: BridgeArgs) -> Result<()> {
    // 1. Validate source chain
    let src_blockchain_id = chain_id_to_blockchain_id(args.chain)
        .ok_or_else(|| anyhow!("Unsupported source chain ID: {}. Supported: 1, 56, 137, 43114, 250, 42220", args.chain))?;

    // 2. Validate destination chain
    let dest_blockchain_id = args.dest_chain.to_uppercase();
    let destination_hex = blockchain_id_to_hex(&dest_blockchain_id)?;

    // 3. Fetch token info to find token address and decimals
    let all_tokens = api::get_token_info().await?;
    let chain_tokens = all_tokens
        .get(src_blockchain_id)
        .ok_or_else(|| anyhow!("No tokens found for source chain {}", src_blockchain_id))?;

    let token_upper = args.token.to_uppercase();
    let token_info = chain_tokens
        .iter()
        .find(|t| t.symbol.to_uppercase() == token_upper)
        .ok_or_else(|| anyhow!("Token {} not found on chain {}. Available: {}",
            token_upper,
            src_blockchain_id,
            chain_tokens.iter().map(|t| t.symbol.as_str()).collect::<Vec<_>>().join(", ")))?;

    let decimals = token_info.precision.unwrap_or(6) as u32;
    let amount_raw = (args.amount * 10f64.powi(decimals as i32)) as u128;

    // 4. Encode recipient as 32 bytes
    let recipient_bytes32 = encode_recipient(&args.recipient, &dest_blockchain_id)?;

    // 5. Generate lock ID
    let lock_id = generate_lock_id();
    let lock_id_decimal = lock_id.to_string();

    // 6. Show bridge summary before proceeding
    let fee_str = "0.3%"; // Default fee; actual fee determined on-chain
    let min_fee_str = token_info.min_fee.as_deref().unwrap_or("unknown");

    eprintln!("Bridge Summary:");
    eprintln!("  Source: {} (chain {})", src_blockchain_id, args.chain);
    eprintln!("  Destination: {}", dest_blockchain_id);
    eprintln!("  Token: {} ({})", token_upper, token_info.address);
    eprintln!("  Amount: {} {}", args.amount, token_upper);
    eprintln!("  Bridge Fee: {} (min: {})", fee_str, min_fee_str);
    eprintln!("  Recipient: {}", args.recipient);
    eprintln!("  Lock ID (decimal): {}", lock_id_decimal);
    eprintln!("  Contract: {}", BRIDGE_CONTRACT);
    if args.dry_run {
        eprintln!("  [DRY RUN MODE - no transactions will be broadcast]");
    } else {
        eprintln!("  NOTE: This will submit 2 transactions: approve + lock. Ask user to confirm before proceeding.");
    }

    // 7. Resolve wallet (only if not dry_run)
    let wallet_addr = if args.dry_run {
        "0x0000000000000000000000000000000000000000".to_string()
    } else {
        onchainos::resolve_wallet(args.chain)?
    };

    // 8. Step 1: ERC-20 approve
    eprintln!("\nStep 1: Approving {} {} for bridge contract...", args.amount, token_upper);
    let approve_result = onchainos::erc20_approve(
        args.chain,
        &token_info.address,
        BRIDGE_CONTRACT,
        amount_raw,
        Some(&wallet_addr),
        args.dry_run,
    ).await?;

    let approve_tx = onchainos::extract_tx_hash(&approve_result);
    if !args.dry_run && approve_tx == "pending" {
        return Err(anyhow!("Approve transaction failed to broadcast: {}", serde_json::to_string(&approve_result)?));
    }

    // 9. Step 2: Lock tokens
    eprintln!("\nStep 2: Locking {} {} on bridge...", args.amount, token_upper);

    // Encode lock() calldata
    // lock(uint128,address,bytes32,bytes4,uint256) selector 0x7bacc91e
    let lock_calldata = encode_lock_calldata(
        lock_id,
        &token_info.address,
        &recipient_bytes32,
        &destination_hex,
        amount_raw,
    )?;

    let lock_result = onchainos::wallet_contract_call(
        args.chain,
        BRIDGE_CONTRACT,
        &lock_calldata,
        Some(&wallet_addr),
        None,
        args.dry_run,
    ).await?;

    let lock_tx = onchainos::extract_tx_hash(&lock_result);
    if !args.dry_run && lock_tx == "pending" {
        return Err(anyhow!("Lock transaction failed to broadcast: {}", serde_json::to_string(&lock_result)?));
    }

    // 10. Return result
    let output = serde_json::json!({
        "ok": true,
        "dry_run": args.dry_run,
        "data": {
            "lockId": lock_id_decimal,
            "sourceChain": src_blockchain_id,
            "destinationChain": dest_blockchain_id,
            "token": token_upper,
            "amount": args.amount,
            "recipient": args.recipient,
            "approveTxHash": approve_tx,
            "lockTxHash": lock_tx,
            "lockCalldata": lock_calldata,
            "nextStep": format!(
                "Use 'allbridge-classic get-tx-status --lock-id {}' to check when bridge confirms the transfer",
                lock_id_decimal
            )
        }
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
