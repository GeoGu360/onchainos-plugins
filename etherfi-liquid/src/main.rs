mod commands;
mod config;
mod onchainos;
mod rpc;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "etherfi-liquid",
    about = "ether.fi Liquid multi-strategy yield vaults — deposit, withdraw, and view positions"
)]
struct Cli {
    /// Chain ID (default: 1 Ethereum mainnet)
    #[arg(long, default_value = "1")]
    chain: u64,

    /// Simulate without broadcasting (no on-chain transactions)
    #[arg(long)]
    dry_run: bool,

    /// Ethereum JSON-RPC URL
    #[arg(long, default_value = "https://ethereum.publicnode.com")]
    rpc_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available ether.fi Liquid vaults with APY and TVL
    Vaults,

    /// Show your positions across all ether.fi Liquid vaults
    Positions {
        /// Wallet address (default: resolved from onchainos)
        #[arg(long)]
        wallet: Option<String>,
    },

    /// Show current exchange rates for each vault
    Rates,

    /// Deposit tokens into an ether.fi Liquid vault
    Deposit {
        /// Vault symbol (LIQUIDETH, LIQUIDUSD, LIQUIDBTC). Default: LIQUIDETH
        #[arg(long, default_value = "LIQUIDETH")]
        vault: String,

        /// Deposit token symbol (e.g. weETH, WETH for ETH vault). Default: primary token for vault
        #[arg(long, default_value = "")]
        token: String,

        /// Amount to deposit in human-readable units (e.g. 0.00005 for 0.00005 weETH)
        #[arg(long)]
        amount: f64,
    },

    /// Withdraw tokens from an ether.fi Liquid vault
    Withdraw {
        /// Vault symbol (LIQUIDETH, LIQUIDUSD, LIQUIDBTC). Default: LIQUIDETH
        #[arg(long, default_value = "LIQUIDETH")]
        vault: String,

        /// Number of shares to withdraw in human-readable units (e.g. 0.00005)
        #[arg(long)]
        shares: Option<f64>,

        /// Withdraw all shares from this vault
        #[arg(long)]
        all: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Vaults => {
            commands::vaults::execute(&cli.rpc_url).await
        }

        Commands::Positions { wallet } => {
            // Resolve wallet: use provided --wallet, or resolve from onchainos
            let resolved_wallet = if let Some(w) = wallet {
                Ok(w)
            } else {
                onchainos::resolve_wallet(cli.chain)
            };
            match resolved_wallet {
                Ok(w) => commands::positions::execute(&w, &cli.rpc_url).await,
                Err(e) => Err(e),
            }
        }

        Commands::Rates => {
            commands::rates::execute(&cli.rpc_url).await
        }

        Commands::Deposit { vault, token, amount } => {
            commands::deposit::execute(
                &vault,
                &token,
                amount,
                cli.chain,
                &cli.rpc_url,
                cli.dry_run,
            )
            .await
        }

        Commands::Withdraw { vault, shares, all } => {
            commands::withdraw::execute(
                &vault,
                shares,
                all,
                cli.chain,
                &cli.rpc_url,
                cli.dry_run,
            )
            .await
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}
