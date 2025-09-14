use anchor_lang::prelude::*;

declare_id!("5GwvY98CgoPByjWEz2ZL6yK2J7oZvKNySrMSSR3E4RU4");

mod instructions;
mod states;

use crate::instructions::*;

// MATH - https://medium.com/@tomarpari90/constant-product-automated-market-maker-everything-you-need-to-know-5bfeb0251ef2

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize_amm_pool(&ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, quantity_a: u64, quantity_b: u64) -> Result<()> {
        ctx.accounts.deposit(quantity_a, quantity_b, &ctx.bumps)
    }

    pub fn swap(ctx: Context<Swap>, quantity: u64, is_a: bool) -> Result<()> {
        ctx.accounts.swap(quantity, is_a, &ctx.bumps)
    }

    pub fn withdraw(ctx: Context<Withdraw>, lp_token_quantity: u64) -> Result<()> {
        ctx.accounts.withdraw(lp_token_quantity, &ctx.bumps)
    }
}
