mod commands;
mod config;
mod onchainos;
mod rpc;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "etherfi-borrowing",
    about = "EtherFi Cash (Borrowing) on Scroll - supply USDC liquidity, view rates, check positions, repay debt"
)]
struct Cli {
    /// Chain ID (default: 534352 Scroll mainnet)
    #[arg(long, default_value = "534352")]
    chain: u64,

    /// Simulate without broadcasting (no on-chain transactions)
    #[arg(long)]
    dry_run: bool,

    /// Scroll JSON-RPC URL
    #[arg(long, default_value = "https://rpc.scroll.io")]
    rpc_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show supported borrow and collateral tokens with LTV parameters
    Markets,

    /// Show current borrowing rates and protocol liquidity stats
    Rates,

    /// Show a UserSafe position (collateral, debt, remaining capacity)
    Position {
        /// UserSafe address to query
        #[arg(long)]
        user_safe: String,
    },

    /// Supply USDC liquidity to earn yield from borrowers
    #[command(name = "supply-liquidity")]
    SupplyLiquidity {
        /// Amount of USDC to supply (e.g. 0.01)
        #[arg(long)]
        amount: f64,
    },

    /// Withdraw previously supplied USDC liquidity
    #[command(name = "withdraw-liquidity")]
    WithdrawLiquidity {
        /// Amount of USDC to withdraw (e.g. 0.01)
        #[arg(long)]
        amount: f64,
    },

    /// Repay USDC debt on behalf of a UserSafe
    Repay {
        /// UserSafe address to repay debt for
        #[arg(long)]
        user_safe: String,

        /// Amount of USDC to repay (e.g. 0.01)
        #[arg(long)]
        amount: f64,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Markets => {
            commands::markets::execute(&cli.rpc_url).await
        }

        Commands::Rates => {
            commands::rates::execute(&cli.rpc_url).await
        }

        Commands::Position { user_safe } => {
            commands::position::execute(&user_safe, &cli.rpc_url).await
        }

        Commands::SupplyLiquidity { amount } => {
            commands::supply::execute(amount, cli.chain, &cli.rpc_url, cli.dry_run).await
        }

        Commands::WithdrawLiquidity { amount } => {
            commands::withdraw::execute(amount, cli.chain, &cli.rpc_url, cli.dry_run).await
        }

        Commands::Repay { user_safe, amount } => {
            commands::repay::execute(&user_safe, amount, cli.chain, &cli.rpc_url, cli.dry_run).await
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}
