mod api;
mod commands;
mod config;
mod onchainos;
mod rpc;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "convex", about = "Convex Finance plugin for onchainos - stake cvxCRV, lock CVX, claim rewards")]
struct Cli {
    /// Chain ID (default: 1 Ethereum mainnet)
    #[arg(long, global = true, default_value = "1")]
    chain: u64,

    /// Simulate without broadcasting
    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List Convex-supported Curve pools and their APYs
    GetPools(commands::get_pools::GetPoolsArgs),
    /// Query your Convex positions (staked cvxCRV, locked CVX, pending rewards)
    GetPositions(commands::get_positions::GetPositionsArgs),
    /// Stake cvxCRV to earn boosted CRV rewards
    StakeCvxcrv(commands::stake_cvxcrv::StakeCvxCrvArgs),
    /// Unstake cvxCRV from the staking contract
    UnstakeCvxcrv(commands::unstake_cvxcrv::UnstakeCvxCrvArgs),
    /// Lock CVX as vlCVX (16-week lock) for voting and rewards
    LockCvx(commands::lock_cvx::LockCvxArgs),
    /// Process expired vlCVX locks to withdraw or relock
    UnlockCvx(commands::unlock_cvx::UnlockCvxArgs),
    /// Claim pending rewards from cvxCRV staking and/or vlCVX
    ClaimRewards(commands::claim_rewards::ClaimRewardsArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let chain_id = cli.chain;
    let dry_run = cli.dry_run;

    match cli.command {
        Commands::GetPools(args) => commands::get_pools::run(args).await,
        Commands::GetPositions(args) => commands::get_positions::run(args, chain_id).await,
        Commands::StakeCvxcrv(args) => commands::stake_cvxcrv::run(args, chain_id, dry_run).await,
        Commands::UnstakeCvxcrv(args) => commands::unstake_cvxcrv::run(args, chain_id, dry_run).await,
        Commands::LockCvx(args) => commands::lock_cvx::run(args, chain_id, dry_run).await,
        Commands::UnlockCvx(args) => commands::unlock_cvx::run(args, chain_id, dry_run).await,
        Commands::ClaimRewards(args) => commands::claim_rewards::run(args, chain_id, dry_run).await,
    }
}
