mod api;
mod commands;
mod config;
mod onchainos;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sanctum-infinity", about = "Sanctum Infinity LST pool — swap, deposit, withdraw, and query")]
struct Cli {
    /// Chain ID (default: 501 Solana mainnet)
    #[arg(long, default_value = "501")]
    chain: u64,

    /// Simulate without broadcasting
    #[arg(long)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Query Sanctum Infinity pool stats and LST allocations
    Pools,

    /// Get a swap quote between LSTs
    Quote {
        /// Input LST symbol (e.g. jitoSOL, mSOL, INF) or mint address
        #[arg(long)]
        from: String,
        /// Output LST symbol or mint address
        #[arg(long)]
        to: String,
        /// Amount of input LST (UI units, e.g. 0.005)
        #[arg(long)]
        amount: f64,
        /// Slippage tolerance in percent (default: 0.5%)
        #[arg(long, default_value = "0.5")]
        slippage: f64,
    },

    /// Swap one LST for another via Sanctum Infinity
    Swap {
        /// Input LST symbol or mint address
        #[arg(long)]
        from: String,
        /// Output LST symbol or mint address
        #[arg(long)]
        to: String,
        /// Amount of input LST (UI units)
        #[arg(long)]
        amount: f64,
        /// Slippage tolerance in percent (default: 0.5%)
        #[arg(long, default_value = "0.5")]
        slippage: f64,
    },

    /// Deposit LST into Sanctum Infinity pool to earn fees
    Deposit {
        /// LST symbol or mint address to deposit
        #[arg(long)]
        lst: String,
        /// Amount to deposit (UI units)
        #[arg(long)]
        amount: f64,
        /// Slippage tolerance in percent (default: 0.5%)
        #[arg(long, default_value = "0.5")]
        slippage: f64,
    },

    /// Withdraw LST from Sanctum Infinity pool by burning INF
    Withdraw {
        /// LST symbol or mint address to receive
        #[arg(long)]
        lst: String,
        /// Amount of INF to burn (UI units)
        #[arg(long)]
        amount: f64,
        /// Slippage tolerance in percent (default: 0.5%)
        #[arg(long, default_value = "0.5")]
        slippage: f64,
    },

    /// View your INF token holdings
    Positions,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = reqwest::Client::new();

    let result = match cli.command {
        Commands::Pools => commands::pools::execute(&client).await,
        Commands::Quote { from, to, amount, slippage } => {
            commands::quote::execute(&client, &from, &to, amount, slippage).await
        }
        Commands::Swap { from, to, amount, slippage } => {
            commands::swap::execute(&client, &from, &to, amount, slippage, cli.dry_run).await
        }
        Commands::Deposit { lst, amount, slippage } => {
            commands::deposit::execute(&client, &lst, amount, slippage, cli.dry_run).await
        }
        Commands::Withdraw { lst, amount, slippage } => {
            commands::withdraw::execute(&client, &lst, amount, slippage, cli.dry_run).await
        }
        Commands::Positions => commands::positions::execute(&client).await,
    };

    if let Err(e) = result {
        let error_output = serde_json::json!({
            "ok": false,
            "error": e.to_string()
        });
        eprintln!("{}", serde_json::to_string_pretty(&error_output).unwrap());
        std::process::exit(1);
    }
}
