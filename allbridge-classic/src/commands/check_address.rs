use anyhow::Result;
use clap::Args;
use crate::api;

#[derive(Args)]
pub struct CheckAddressArgs {
    /// Destination blockchain ID (e.g. SOL, ETH, BSC, POL)
    #[arg(long)]
    pub chain: String,

    /// Recipient address to validate
    #[arg(long)]
    pub address: String,
}

pub async fn run(args: CheckAddressArgs) -> Result<()> {
    let chain_upper = args.chain.to_uppercase();
    let resp = api::check_address(&chain_upper, &args.address).await?;

    let output = serde_json::json!({
        "ok": true,
        "data": {
            "chain": chain_upper,
            "address": args.address,
            "result": resp
        }
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
