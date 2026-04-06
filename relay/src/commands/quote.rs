use crate::{config, onchainos, rpc};
use clap::Args;

#[derive(Args)]
pub struct QuoteArgs {
    /// Source chain ID (e.g. 8453 for Base)
    #[arg(long)]
    pub from_chain: u64,

    /// Destination chain ID (e.g. 1 for Ethereum)
    #[arg(long)]
    pub to_chain: u64,

    /// Token symbol or address on source chain (e.g. ETH or 0x0000...)
    #[arg(long, default_value = "ETH")]
    pub token: String,

    /// Amount to bridge (in ETH/token units, e.g. 0.001)
    #[arg(long)]
    pub amount: f64,

    /// Recipient address (defaults to wallet address)
    #[arg(long)]
    pub recipient: Option<String>,

    /// Wallet address (resolved from onchainos if omitted)
    #[arg(long)]
    pub from: Option<String>,
}

pub fn run(args: QuoteArgs) -> anyhow::Result<()> {
    // Resolve user address (dry_run=false for quote, it's read-only)
    let user = if let Some(f) = args.from.clone() {
        f
    } else {
        onchainos::resolve_wallet(args.from_chain, false)
            .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string())
    };

    // Resolve currency addresses and decimals
    let (origin_currency, decimals) = if args.token.starts_with("0x") {
        // Raw address provided — assume 18 decimals (user must pass correct amount)
        (args.token.clone(), 18u32)
    } else {
        match rpc::resolve_token(args.from_chain, &args.token)? {
            Some((addr, dec)) => (addr, dec),
            None => {
                eprintln!("Warning: token '{}' not found on chain {} — treating as ETH", args.token, args.from_chain);
                (config::ETH_ADDRESS.to_string(), 18u32)
            }
        }
    };
    let destination_currency = if args.token.starts_with("0x") {
        args.token.clone()
    } else {
        match rpc::resolve_token(args.to_chain, &args.token)? {
            Some((addr, _)) => addr,
            None => origin_currency.clone(),
        }
    };

    // Convert amount using actual token decimals
    let scale = 10u128.pow(decimals);
    let amount_wei = (args.amount * scale as f64) as u128;
    let amount_str = amount_wei.to_string();

    let recipient = args.recipient.as_deref();

    println!("=== Relay Bridge Quote ===");
    println!("From:     Chain {} ({} {})", args.from_chain, args.amount, args.token);
    println!("To:       Chain {}", args.to_chain);
    println!("Amount:   {} ({}d scaled)", amount_str, decimals);
    println!("User:     {}", user);
    println!();

    let quote = rpc::get_quote(
        &user,
        args.from_chain,
        args.to_chain,
        &origin_currency,
        &destination_currency,
        &amount_str,
        recipient,
    )?;

    if let Some(err) = quote.get("message") {
        anyhow::bail!("API error: {}", err);
    }

    // Display fees
    if let Some(fees) = quote.get("fees") {
        println!("--- Fees ---");
        if let Some(gas) = fees.get("gas") {
            println!("  Gas:             {} ({})",
                gas["amountFormatted"].as_str().unwrap_or("?"),
                gas["currency"]["symbol"].as_str().unwrap_or("ETH"));
        }
        if let Some(relayer) = fees.get("relayer") {
            println!("  Relayer:         {} ETH (~${} USD)",
                relayer["amountFormatted"].as_str().unwrap_or("?"),
                relayer["amountUsd"].as_str().unwrap_or("?"));
        }
        if let Some(relayer_gas) = fees.get("relayerGas") {
            println!("  Relayer Gas:     {} ETH",
                relayer_gas["amountFormatted"].as_str().unwrap_or("?"));
        }
        if let Some(relayer_svc) = fees.get("relayerService") {
            println!("  Relayer Service: {} ETH",
                relayer_svc["amountFormatted"].as_str().unwrap_or("?"));
        }
    }

    // Display details
    if let Some(details) = quote.get("details") {
        println!();
        println!("--- Details ---");
        let time_est = details["timeEstimate"].as_u64().unwrap_or(0);
        println!("  Time Estimate:  ~{} seconds", time_est);

        if let Some(currency_out) = details.get("currencyOut") {
            println!("  You Receive:    {} {}",
                currency_out["amountFormatted"].as_str().unwrap_or("?"),
                currency_out["currency"]["symbol"].as_str().unwrap_or("?"));
            println!("  Value Out:      ~${} USD",
                currency_out["amountUsd"].as_str().unwrap_or("?"));
        }
        if let Some(currency_in) = details.get("currencyIn") {
            println!("  Value In:       ~${} USD",
                currency_in["amountUsd"].as_str().unwrap_or("?"));
        }
        if let Some(impact) = details.get("totalImpact") {
            println!("  Total Cost:     ~${} USD ({}%)",
                impact["usd"].as_str().unwrap_or("?"),
                impact["percent"].as_str().unwrap_or("?"));
        }
    }

    // Show requestId if available
    if let Some(steps) = quote["steps"].as_array() {
        if let Some(step) = steps.first() {
            if let Some(request_id) = step["requestId"].as_str() {
                println!();
                println!("Request ID: {}", request_id);
                println!("  (Use `relay status --request-id {}` to check status)", request_id);
            }
        }
    }

    Ok(())
}

