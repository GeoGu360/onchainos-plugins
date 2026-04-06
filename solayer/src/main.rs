mod commands;
mod onchainos;
mod config;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "solayer", about = "Solayer liquid staking on Solana")]
struct Cli {
    /// Chain ID (501 = Solana mainnet)
    #[arg(long, default_value = "501")]
    chain: u64,

    /// Simulate without broadcasting on-chain
    #[arg(long)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get current sSOL/SOL exchange rate, APY, and TVL
    Rates,
    /// Check your sSOL positions and balance
    Positions,
    /// Stake SOL to receive sSOL
    Stake {
        /// Amount of SOL to stake (UI units, e.g. 0.001)
        #[arg(long)]
        amount: f64,
        /// Simulate without broadcasting on-chain
        #[arg(long)]
        dry_run: bool,
    },
    /// Unstake sSOL to receive SOL (returns UI guidance)
    Unstake {
        /// Amount of sSOL to unstake (UI units)
        #[arg(long)]
        amount: f64,
        /// Simulate without broadcasting on-chain
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Rates => commands::rates::execute().await,
        Commands::Positions => commands::positions::execute().await,
        Commands::Stake { amount, dry_run } => {
            commands::stake::execute(amount, dry_run || cli.dry_run).await
        }
        Commands::Unstake { amount, dry_run } => {
            commands::unstake::execute(amount, dry_run || cli.dry_run).await
        }
    };

    match result {
        Ok(val) => println!("{}", val),
        Err(e) => {
            let err = serde_json::json!({
                "ok": false,
                "error": e.to_string()
            });
            println!("{}", err);
            std::process::exit(1);
        }
    }
}
