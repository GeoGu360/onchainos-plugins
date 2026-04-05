mod commands;
mod config;
mod onchainos;
mod rpc;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "layer-bank", about = "LayerBank lending protocol plugin for Scroll (chain 534352)")]
struct Cli {
    /// Chain ID (534352 = Scroll mainnet)
    #[arg(long, default_value = "534352")]
    chain: u64,

    /// Simulate without broadcasting on-chain transactions
    #[arg(long)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List lToken markets with TVL, utilization, and asset prices
    Markets,

    /// View your supplied and borrowed positions, health factor
    Positions {
        /// Wallet address (defaults to logged-in onchainos wallet)
        #[arg(long)]
        wallet: Option<String>,
    },

    /// Supply an asset to earn interest (mints lTokens)
    Supply {
        /// Asset symbol: ETH, USDC, USDT, wstETH, WBTC
        #[arg(long)]
        asset: String,

        /// Human-readable amount (e.g. 0.01 for 0.01 USDC)
        #[arg(long)]
        amount: f64,

        /// Sender wallet address (defaults to logged-in wallet)
        #[arg(long)]
        from: Option<String>,
    },

    /// Withdraw a supplied asset (redeemUnderlying)
    Withdraw {
        /// Asset symbol: ETH, USDC, USDT, wstETH, WBTC
        #[arg(long)]
        asset: String,

        /// Human-readable amount to withdraw
        #[arg(long)]
        amount: f64,

        /// Sender wallet address (defaults to logged-in wallet)
        #[arg(long)]
        from: Option<String>,
    },

    /// Borrow an asset against supplied collateral (DRY-RUN recommended)
    Borrow {
        /// Asset symbol: ETH, USDC, USDT, wstETH, WBTC
        #[arg(long)]
        asset: String,

        /// Human-readable borrow amount
        #[arg(long)]
        amount: f64,

        /// Sender wallet address (defaults to logged-in wallet)
        #[arg(long)]
        from: Option<String>,
    },

    /// Repay a borrow position (DRY-RUN recommended)
    Repay {
        /// Asset symbol: ETH, USDC, USDT, wstETH, WBTC
        #[arg(long)]
        asset: String,

        /// Human-readable repay amount
        #[arg(long)]
        amount: f64,

        /// Sender wallet address (defaults to logged-in wallet)
        #[arg(long)]
        from: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Markets => {
            commands::markets::run(cli.chain).await
        }
        Commands::Positions { wallet } => {
            commands::positions::run(cli.chain, wallet).await
        }
        Commands::Supply { asset, amount, from } => {
            commands::supply::run(cli.chain, asset, amount, from, cli.dry_run).await
        }
        Commands::Withdraw { asset, amount, from } => {
            commands::withdraw::run(cli.chain, asset, amount, from, cli.dry_run).await
        }
        Commands::Borrow { asset, amount, from } => {
            commands::borrow::run(cli.chain, asset, amount, from, cli.dry_run).await
        }
        Commands::Repay { asset, amount, from } => {
            commands::repay::run(cli.chain, asset, amount, from, cli.dry_run).await
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
