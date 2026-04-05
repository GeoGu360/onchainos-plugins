mod calldata;
mod commands;
mod config;
mod onchainos;
mod rpc;

use clap::{Parser, Subcommand};
use serde_json::Value;

#[derive(Parser)]
#[command(
    name = "aave-v2",
    about = "Aave V2 classic lending pool — deposit, withdraw, borrow, repay on Ethereum",
    version = "0.1.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Chain ID (Aave V2 is on Ethereum mainnet: 1)
    #[arg(long, global = true, default_value = "1")]
    chain: u64,
    /// Wallet address (defaults to active onchainos wallet)
    #[arg(long, global = true)]
    from: Option<String>,
    /// Simulate without broadcasting (required for borrow and repay)
    #[arg(long, global = true, default_value = "false")]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List all Aave V2 reserves with supply/borrow APYs
    Reserves {
        /// Filter by asset address (optional)
        #[arg(long)]
        asset: Option<String>,
    },
    /// View your aToken deposits and debt positions
    Positions {},
    /// Deposit an asset to earn interest (aTokens)
    Deposit {
        /// Asset symbol (e.g. USDT, USDC, WETH) or ERC-20 address
        #[arg(long)]
        asset: String,
        /// Human-readable amount (e.g. 0.01 for 0.01 USDT)
        #[arg(long)]
        amount: f64,
    },
    /// Withdraw a previously deposited asset
    Withdraw {
        /// Asset symbol or ERC-20 address
        #[arg(long)]
        asset: String,
        /// Human-readable amount to withdraw
        #[arg(long)]
        amount: Option<f64>,
        /// Withdraw the full aToken balance
        #[arg(long, default_value = "false")]
        all: bool,
    },
    /// Borrow an asset against posted collateral (dry-run only)
    Borrow {
        /// Asset symbol or ERC-20 address
        #[arg(long)]
        asset: String,
        /// Human-readable amount to borrow
        #[arg(long)]
        amount: f64,
        /// Interest rate mode: 1=stable, 2=variable (default: 2)
        #[arg(long, default_value = "2")]
        rate_mode: u128,
    },
    /// Repay borrowed debt (dry-run only)
    Repay {
        /// Asset symbol or ERC-20 address
        #[arg(long)]
        asset: String,
        /// Human-readable amount to repay
        #[arg(long)]
        amount: Option<f64>,
        /// Repay the full outstanding balance
        #[arg(long, default_value = "false")]
        all: bool,
        /// Interest rate mode: 1=stable, 2=variable (default: 2)
        #[arg(long, default_value = "2")]
        rate_mode: u128,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result: anyhow::Result<Value> = match cli.command {
        Commands::Reserves { asset } => {
            commands::reserves::run(cli.chain, asset.as_deref()).await
        }
        Commands::Positions {} => {
            commands::positions::run(cli.chain, cli.from.as_deref()).await
        }
        Commands::Deposit { asset, amount } => {
            commands::deposit::run(cli.chain, &asset, amount, cli.from.as_deref(), cli.dry_run)
                .await
        }
        Commands::Withdraw { asset, amount, all } => {
            commands::withdraw::run(
                cli.chain,
                &asset,
                amount,
                all,
                cli.from.as_deref(),
                cli.dry_run,
            )
            .await
        }
        Commands::Borrow {
            asset,
            amount,
            rate_mode,
        } => {
            commands::borrow::run(
                cli.chain,
                &asset,
                amount,
                rate_mode,
                cli.from.as_deref(),
                cli.dry_run,
            )
            .await
        }
        Commands::Repay {
            asset,
            amount,
            all,
            rate_mode,
        } => {
            commands::repay::run(
                cli.chain,
                &asset,
                amount,
                all,
                rate_mode,
                cli.from.as_deref(),
                cli.dry_run,
            )
            .await
        }
    };

    match result {
        Ok(val) => {
            println!("{}", serde_json::to_string_pretty(&val).unwrap_or_default());
        }
        Err(err) => {
            let error_json = serde_json::json!({
                "ok": false,
                "error": err.to_string()
            });
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&error_json).unwrap_or_default()
            );
            std::process::exit(1);
        }
    }
}
