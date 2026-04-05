use anyhow::Result;
use clap::Args;
use crate::api;

#[derive(Args)]
pub struct GetTxStatusArgs {
    /// Lock transaction ID (decimal number from bridge Sent event)
    #[arg(long)]
    pub lock_id: String,
}

pub async fn run(args: GetTxStatusArgs) -> Result<()> {
    let sign_resp = api::get_sign(&args.lock_id).await?;

    let output = serde_json::json!({
        "ok": true,
        "data": {
            "lockId": sign_resp.lock_id,
            "block": sign_resp.block,
            "source": sign_resp.source,
            "destination": sign_resp.destination,
            "amount": sign_resp.amount,
            "recipient": sign_resp.recipient,
            "tokenSource": sign_resp.token_source,
            "tokenSourceAddress": sign_resp.token_source_address,
            "status": "confirmed",
            "note": "Bridge has confirmed the lock. Funds will be available on the destination chain shortly."
        }
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
