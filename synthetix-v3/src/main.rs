mod commands;
mod config;
mod onchainos;
mod rpc;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "synthetix-v3", about = "Synthetix V3 on Base — collateral management and perps market queries")]
struct Cli {
    /// Chain ID (default: 8453 Base mainnet)
    #[arg(long, default_value = "8453")]
    chain: u64,

    /// Simulate without broadcasting on-chain transactions
    #[arg(long)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List Synthetix V3 Perps markets with funding rates and sizes
    Markets {
        /// Optional specific market ID to query
        #[arg(long)]
        market_id: Option<u128>,
    },
    /// Query open Perps positions for a given account
    Positions {
        /// Synthetix V3 account ID (uint128)
        #[arg(long)]
        account_id: u128,
    },
    /// Query collateral balances for a given account
    Collateral {
        /// Synthetix V3 account ID (uint128)
        #[arg(long)]
        account_id: u128,
    },
    /// Deposit sUSDC collateral into Synthetix V3 Core
    DepositCollateral {
        /// Synthetix V3 account ID
        #[arg(long)]
        account_id: u128,
        /// Amount to deposit (human-readable, e.g. 10.0 for 10 sUSDC)
        #[arg(long)]
        amount: f64,
        /// Sender address (optional, resolves from onchainos if omitted)
        #[arg(long)]
        from: Option<String>,
    },
    /// Withdraw sUSDC collateral from Synthetix V3 Core
    WithdrawCollateral {
        /// Synthetix V3 account ID
        #[arg(long)]
        account_id: u128,
        /// Amount to withdraw (human-readable, e.g. 5.0 for 5 sUSDC)
        #[arg(long)]
        amount: f64,
        /// Sender address (optional, resolves from onchainos if omitted)
        #[arg(long)]
        from: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Markets { market_id } => {
            commands::markets::execute(market_id).await?;
        }
        Commands::Positions { account_id } => {
            commands::positions::execute(account_id).await?;
        }
        Commands::Collateral { account_id } => {
            commands::collateral::execute(account_id).await?;
        }
        Commands::DepositCollateral {
            account_id,
            amount,
            from,
        } => {
            commands::deposit_collateral::execute(account_id, amount, from, cli.dry_run, cli.chain)
                .await?;
        }
        Commands::WithdrawCollateral {
            account_id,
            amount,
            from,
        } => {
            commands::withdraw_collateral::execute(account_id, amount, from, cli.dry_run, cli.chain)
                .await?;
        }
    }

    Ok(())
}
