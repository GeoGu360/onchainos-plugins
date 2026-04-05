mod abi;
mod commands;
mod config;
mod onchainos;
mod rpc;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "init-capital",
    about = "INIT Capital lending protocol plugin for Blast (chain 81457)"
)]
struct Cli {
    /// Chain ID (81457 = Blast mainnet)
    #[arg(long, default_value = "81457")]
    chain: u64,

    /// Simulate without broadcasting on-chain transactions
    #[arg(long)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List lending pools with supply/borrow rates and total assets
    Pools,

    /// View your INIT Capital positions with health factor
    Positions {
        /// Wallet address (defaults to logged-in onchainos wallet)
        #[arg(long)]
        wallet: Option<String>,
    },

    /// Get health factor for a specific position
    HealthFactor {
        /// Position ID to check
        #[arg(long)]
        pos_id: u64,
    },

    /// Supply an asset to earn interest (creates new position if pos-id=0)
    Supply {
        /// Asset symbol: WETH, USDB
        #[arg(long)]
        asset: String,

        /// Human-readable amount (e.g. 0.01 for 0.01 WETH)
        #[arg(long)]
        amount: f64,

        /// Position ID (0 = create new position)
        #[arg(long, default_value = "0")]
        pos_id: u64,

        /// Sender wallet address (defaults to logged-in wallet)
        #[arg(long)]
        from: Option<String>,
    },

    /// Withdraw collateral from a position
    Withdraw {
        /// Asset symbol: WETH, USDB
        #[arg(long)]
        asset: String,

        /// Human-readable amount to withdraw
        #[arg(long)]
        amount: f64,

        /// Position ID to withdraw from
        #[arg(long)]
        pos_id: u64,

        /// Recipient address (defaults to sender)
        #[arg(long)]
        to: Option<String>,

        /// Sender wallet address (defaults to logged-in wallet)
        #[arg(long)]
        from: Option<String>,
    },

    /// Borrow an asset against supplied collateral (DRY-RUN recommended)
    Borrow {
        /// Asset symbol: WETH, USDB
        #[arg(long)]
        asset: String,

        /// Human-readable borrow amount
        #[arg(long)]
        amount: f64,

        /// Position ID to borrow from
        #[arg(long)]
        pos_id: u64,

        /// Recipient address for borrowed tokens (defaults to sender)
        #[arg(long)]
        to: Option<String>,

        /// Sender wallet address (defaults to logged-in wallet)
        #[arg(long)]
        from: Option<String>,
    },

    /// Repay a borrow position (DRY-RUN recommended)
    Repay {
        /// Asset symbol: WETH, USDB
        #[arg(long)]
        asset: String,

        /// Human-readable repay amount
        #[arg(long)]
        amount: f64,

        /// Position ID to repay
        #[arg(long)]
        pos_id: u64,

        /// Sender wallet address (defaults to logged-in wallet)
        #[arg(long)]
        from: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Pools => {
            commands::pools::run(cli.chain).await
        }
        Commands::Positions { wallet } => {
            commands::positions::run(cli.chain, wallet).await
        }
        Commands::HealthFactor { pos_id } => {
            commands::health_factor::run(cli.chain, pos_id).await
        }
        Commands::Supply { asset, amount, pos_id, from } => {
            commands::supply::run(cli.chain, asset, amount, pos_id, from, cli.dry_run).await
        }
        Commands::Withdraw { asset, amount, pos_id, to, from } => {
            commands::withdraw::run(cli.chain, asset, amount, pos_id, to, from, cli.dry_run).await
        }
        Commands::Borrow { asset, amount, pos_id, to, from } => {
            commands::borrow::run(cli.chain, asset, amount, pos_id, to, from, cli.dry_run).await
        }
        Commands::Repay { asset, amount, pos_id, from } => {
            commands::repay::run(cli.chain, asset, amount, pos_id, from, cli.dry_run).await
        }
    };

    match result {
        Ok(val) => println!("{}", serde_json::to_string_pretty(&val).unwrap()),
        Err(e) => {
            let err = serde_json::json!({"ok": false, "error": e.to_string()});
            eprintln!("{}", serde_json::to_string_pretty(&err).unwrap());
            std::process::exit(1);
        }
    }
}
