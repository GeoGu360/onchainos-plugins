use anyhow::Result;
use clap::Args;
use crate::{config, onchainos, rpc};

#[derive(Args, Debug)]
pub struct GetPositionsArgs {
    /// Wallet address to query (defaults to onchainos logged-in wallet)
    #[arg(long)]
    pub address: Option<String>,
}

pub async fn run(args: GetPositionsArgs, chain_id: u64) -> Result<()> {
    let wallet = match args.address {
        Some(addr) => addr,
        None => onchainos::resolve_wallet(chain_id)?,
    };

    // cvxCRV staked balance
    let cvxcrv_staked = rpc::erc20_balance_of(config::CVXCRV_STAKING, &wallet).await.unwrap_or(0);
    let cvxcrv_earned = rpc::cvxcrv_earned(&wallet).await.unwrap_or(0);

    // vlCVX locked balance
    let vlcvx_balance = rpc::erc20_balance_of(config::VLCVX, &wallet).await.unwrap_or(0);

    // Liquid token balances
    let cvx_liquid = rpc::erc20_balance_of(config::CVX_TOKEN, &wallet).await.unwrap_or(0);
    let cvxcrv_liquid = rpc::erc20_balance_of(config::CVXCRV_TOKEN, &wallet).await.unwrap_or(0);
    let crv_liquid = rpc::erc20_balance_of(config::CRV_TOKEN, &wallet).await.unwrap_or(0);

    let output = serde_json::json!({
        "ok": true,
        "data": {
            "wallet": wallet,
            "chain": "ethereum",
            "chain_id": chain_id,
            "positions": {
                "cvxCRV_staked": {
                    "contract": config::CVXCRV_STAKING,
                    "balance": rpc::format_amount(cvxcrv_staked, 18),
                    "pending_crv_rewards": rpc::format_amount(cvxcrv_earned, 18)
                },
                "vlCVX_locked": {
                    "contract": config::VLCVX,
                    "balance": rpc::format_amount(vlcvx_balance, 18),
                    "note": "16-week lock period"
                }
            },
            "liquid_balances": {
                "CVX": rpc::format_amount(cvx_liquid, 18),
                "cvxCRV": rpc::format_amount(cvxcrv_liquid, 18),
                "CRV": rpc::format_amount(crv_liquid, 18)
            }
        }
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
