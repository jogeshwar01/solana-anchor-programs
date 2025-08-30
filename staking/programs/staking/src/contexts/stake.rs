use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::{contexts::utils::update_points, stake_account::StakeAccount, states::error::StakeError};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        mut, 
        seeds = [b"stake_account", signer.key().as_ref()], 
        bump = user_stake_account.bump, 
        constraint = user_stake_account.owner == signer.key() @ StakeError::Unauthorized)]
    user_stake_account: Account<'info, StakeAccount>,
    #[account(mut)]
    signer: Signer<'info>,
    system_program: Program<'info, System>,
}

impl<'info> Stake<'info> {
    pub fn stake(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, StakeError::InvalidAmount);
        
        let pda_account = &mut self.user_stake_account;
        let clock = Clock::get()?;
        
        // Update points before changing staked amount
        update_points(pda_account, clock.unix_timestamp)?;
        
        // Transfer SOL from user to PDA
        let cpi_context = CpiContext::new(
     self.system_program.to_account_info(),
    system_program::Transfer {
                from: self.signer.to_account_info(),
                to: pda_account.to_account_info(),
            },
        );
        system_program::transfer(cpi_context, amount)?;
        
        // Update staked amount
        pda_account.staked_amount = pda_account.staked_amount.checked_add(amount)
            .ok_or(StakeError::Overflow)?;
        
        msg!("Staked {} lamports. Total staked: {}, Total points: {}", 
             amount, pda_account.staked_amount, pda_account.total_points / 1_000_000);
        Ok(())
    }
}
