mod commands;
mod config;
mod onchainos;
mod rpc;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "curve-lending",
    about = "Curve Lending (LlamaLend) — borrow crvUSD against ETH/wstETH/tBTC collateral"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all Curve Lending markets with TVL and activity
    Markets(commands::markets::MarketsArgs),

    /// Show active lending positions for a wallet
    Positions(commands::positions::PositionsArgs),

    /// Show borrow and lend APY rates for markets
    Rates(commands::rates::RatesArgs),

    /// Deposit collateral into a Curve Lending market
    DepositCollateral(commands::deposit_collateral::DepositCollateralArgs),

    /// Borrow crvUSD against deposited collateral
    Borrow(commands::borrow::BorrowArgs),

    /// Repay crvUSD debt
    Repay(commands::repay::RepayArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Markets(args) => commands::markets::run(args).await,
        Commands::Positions(args) => commands::positions::run(args).await,
        Commands::Rates(args) => commands::rates::run(args).await,
        Commands::DepositCollateral(args) => commands::deposit_collateral::run(args).await,
        Commands::Borrow(args) => commands::borrow::run(args).await,
        Commands::Repay(args) => commands::repay::run(args).await,
    }
}
