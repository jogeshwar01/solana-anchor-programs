use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::{
    contexts::utils::update_points, stake_account::StakeAccount, states::error::StakeError,
};

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(
        mut,
        seeds = [b"stake_account", signer.key().as_ref()],
        bump = user_stake_account.bump,
        constraint = user_stake_account.owner == signer.key() @ StakeError::Unauthorized
    )]
    user_stake_account: Account<'info, StakeAccount>,
    #[account(mut)]
    signer: Signer<'info>,
    system_program: Program<'info, System>,
}

impl<'info> Unstake<'info> {
    pub fn unstake(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, StakeError::InvalidAmount);

        let pda_account = &mut self.user_stake_account;
        let clock = Clock::get()?;

        require!(
            pda_account.staked_amount >= amount,
            StakeError::InsufficientStake
        );

        update_points(pda_account, clock.epoch)?;

        // Transfer SOL from PDA back to user
        let seeds: &[&[u8]; 3] = &[
            b"stake_account",
            self.signer.key.as_ref(),
            &[pda_account.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            system_program::Transfer {
                from: pda_account.to_account_info(),
                to: self.signer.to_account_info(),
            },
            signer_seeds,
        );
        system_program::transfer(cpi_context, amount)?;

        // Update staked amount
        pda_account.staked_amount = pda_account
            .staked_amount
            .checked_sub(amount)
            .ok_or(StakeError::Underflow)?;

        msg!(
            "Unstaked {} lamports. Remaining staked: {}, Total points: {}",
            amount,
            pda_account.staked_amount,
            pda_account.total_points
        );

        Ok(())
    }
}
