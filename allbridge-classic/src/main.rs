mod api;
mod commands;
mod onchainos;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "allbridge-classic", about = "Allbridge Classic cross-chain bridge plugin for onchainos")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List supported tokens and chains
    GetTokens,
    /// Check bridge transaction status by lock ID
    GetTxStatus(commands::get_tx_status::GetTxStatusArgs),
    /// Bridge ERC-20 tokens cross-chain (EVM source)
    Bridge(commands::bridge::BridgeArgs),
    /// Validate a recipient address for a target chain
    CheckAddress(commands::check_address::CheckAddressArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::GetTokens => commands::get_tokens::run().await,
        Commands::GetTxStatus(args) => commands::get_tx_status::run(args).await,
        Commands::Bridge(args) => commands::bridge::run(args).await,
        Commands::CheckAddress(args) => commands::check_address::run(args).await,
    }
}
