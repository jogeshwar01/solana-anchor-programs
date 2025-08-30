use anchor_lang::prelude::*;
mod contexts;
use contexts::{ClaimReward, CreateStakeAccount, Stake, Unstake};
mod states;
use states::*;

declare_id!("FhqzRxobtLDbbdaJXFfx6Tei2iosd9T2UeXvo96FTZbN");

#[program]
pub mod stake {
    use super::*;

    pub fn initialize(ctx: Context<CreateStakeAccount>) -> Result<()> {
        ctx.accounts
            .initialize_stake_account(&ctx.accounts.signer.key(), ctx.bumps.stake_account)
        // You don’t get the bump from ctx.accounts.stake_account.bump. that account isnt initialised yet
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        ctx.accounts.stake(amount)
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        ctx.accounts.unstake(amount)
    }

    pub fn claim_points(ctx: Context<ClaimReward>) -> Result<()> {
        ctx.accounts.claim_reward(ctx.bumps.mint_authority)
    }
}
