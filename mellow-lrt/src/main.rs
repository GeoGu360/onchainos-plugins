mod api;
mod commands;
mod config;
mod onchainos;
mod rpc;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "mellow-lrt",
    about = "Mellow LRT — Liquid Restaking Token vaults on Ethereum. Deposit ETH/wstETH to earn restaking yield."
)]
struct Cli {
    /// Chain ID (default: 1 Ethereum mainnet)
    #[arg(long, default_value = "1")]
    chain: u64,

    /// Simulate without broadcasting (dry run)
    #[arg(long)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List Mellow LRT vaults: symbol, APR, TVL, supported deposit tokens
    Vaults {
        /// Maximum number of vaults to display (sorted by TVL)
        #[arg(long, default_value = "20")]
        limit: usize,
    },

    /// Show your current LRT positions (shares, estimated assets, claimable, pending)
    Positions {
        /// Wallet address (optional, resolved from onchainos if not provided)
        #[arg(long)]
        wallet: Option<String>,
    },

    /// Deposit ETH/wstETH/WETH/stETH into a Mellow LRT vault
    Deposit {
        /// Vault symbol (e.g. steakLRT, Re7LRT) or vault address
        #[arg(long)]
        vault: String,

        /// Token to deposit: ETH, WETH, stETH, or wstETH
        #[arg(long, default_value = "ETH")]
        token: String,

        /// Amount to deposit (e.g. 0.00005)
        #[arg(long)]
        amount: String,

        /// Sender address (optional, resolved from onchainos if not provided)
        #[arg(long)]
        from: Option<String>,
    },

    /// Withdraw (redeem) LRT shares — initiates 2-step async withdrawal process
    Withdraw {
        /// Vault symbol or address
        #[arg(long)]
        vault: String,

        /// Amount of shares to redeem (human-readable, e.g. 0.001)
        #[arg(long)]
        amount: Option<String>,

        /// Redeem all shares
        #[arg(long)]
        all: bool,

        /// Sender address (optional)
        #[arg(long)]
        from: Option<String>,
    },

    /// Claim unlocked withdrawal assets after the queue period (~14 days)
    Claim {
        /// Vault symbol or address
        #[arg(long)]
        vault: String,

        /// Recipient address (optional)
        #[arg(long)]
        from: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let result = run(cli).await;
    if let Err(e) = result {
        let error_output = serde_json::json!({
            "ok": false,
            "error": e.to_string(),
        });
        eprintln!("{}", serde_json::to_string_pretty(&error_output).unwrap());
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Vaults { limit } => {
            commands::vaults::run(cli.chain, limit).await
        }
        Commands::Positions { wallet } => {
            // For positions, always need a wallet address
            let resolved_wallet = if let Some(w) = wallet {
                w
            } else {
                onchainos::resolve_wallet(cli.chain, false)?
            };
            commands::positions::run(cli.chain, &resolved_wallet).await
        }
        Commands::Deposit { vault, token, amount, from } => {
            commands::deposit::run(
                &vault,
                &token,
                &amount,
                cli.chain,
                from.as_deref(),
                cli.dry_run,
            ).await
        }
        Commands::Withdraw { vault, amount, all, from } => {
            commands::withdraw::run(
                &vault,
                amount.as_deref(),
                all,
                cli.chain,
                from.as_deref(),
                cli.dry_run,
            ).await
        }
        Commands::Claim { vault, from } => {
            commands::claim::run(
                &vault,
                cli.chain,
                from.as_deref(),
                cli.dry_run,
            ).await
        }
    }
}
