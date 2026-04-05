mod commands;
mod config;
mod onchainos;
mod rpc;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "benqi", about = "Benqi Lending plugin for Avalanche C-Chain")]
struct Cli {
    /// Chain ID (43114 = Avalanche C-Chain)
    #[arg(long, default_value = "43114")]
    chain: u64,

    /// Simulate without broadcasting on-chain transactions
    #[arg(long)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List qiToken markets with supply/borrow APR
    Markets,

    /// View your supplied and borrowed positions
    Positions {
        /// Wallet address (defaults to logged-in onchainos wallet)
        #[arg(long)]
        wallet: Option<String>,
    },

    /// Supply an asset to earn interest (mints qiTokens)
    Supply {
        /// Asset symbol: AVAX, USDC, USDT, ETH, BTC, LINK, DAI, QI
        #[arg(long)]
        asset: String,

        /// Human-readable amount (e.g. 0.01 for 0.01 USDC)
        #[arg(long)]
        amount: f64,

        /// Sender wallet (defaults to logged-in wallet)
        #[arg(long)]
        from: Option<String>,
    },

    /// Redeem qiTokens to get back underlying asset
    Redeem {
        /// Asset symbol: AVAX, USDC, USDT, ETH, BTC, LINK, DAI, QI
        #[arg(long)]
        asset: String,

        /// Underlying amount to redeem (e.g. 0.01 for 0.01 USDC)
        #[arg(long)]
        amount: f64,

        /// Sender wallet (defaults to logged-in wallet)
        #[arg(long)]
        from: Option<String>,
    },

    /// Preview borrowing an asset (DRY-RUN ONLY)
    Borrow {
        /// Asset symbol: AVAX, USDC, USDT, ETH, BTC, LINK, DAI, QI
        #[arg(long)]
        asset: String,

        /// Human-readable borrow amount
        #[arg(long)]
        amount: f64,

        /// Sender wallet
        #[arg(long)]
        from: Option<String>,
    },

    /// Preview repaying a borrow (DRY-RUN ONLY)
    Repay {
        /// Asset symbol: AVAX, USDC, USDT, ETH, BTC, LINK, DAI, QI
        #[arg(long)]
        asset: String,

        /// Human-readable repay amount
        #[arg(long)]
        amount: f64,

        /// Sender wallet
        #[arg(long)]
        from: Option<String>,
    },

    /// Claim QI or AVAX rewards from the Comptroller
    ClaimRewards {
        /// Reward type: 0 = QI token, 1 = AVAX
        #[arg(long, default_value = "0")]
        reward_type: u8,

        /// Sender wallet (defaults to logged-in wallet)
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
        Commands::Redeem { asset, amount, from } => {
            commands::redeem::run(cli.chain, asset, amount, from, cli.dry_run).await
        }
        Commands::Borrow { asset, amount, from } => {
            commands::borrow::run(cli.chain, asset, amount, from).await
        }
        Commands::Repay { asset, amount, from } => {
            commands::repay::run(cli.chain, asset, amount, from).await
        }
        Commands::ClaimRewards { reward_type, from } => {
            commands::claim_rewards::run(cli.chain, reward_type, from, cli.dry_run).await
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
