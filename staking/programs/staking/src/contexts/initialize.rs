use anchor_lang::prelude::*;

use crate::stake_account::StakeAccount;

#[derive(Accounts)]
pub struct CreateStakeAccount<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + StakeAccount::INIT_SPACE,
        seeds = [b"stake_account", signer.key().as_ref()],
        bump
    )]
    pub stake_account: Account<'info, StakeAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateStakeAccount<'info> {
    pub fn initialize_stake_account(&mut self, signer_public_key: &Pubkey, bump: u8) -> Result<()> {
        self.stake_account.set_inner(StakeAccount {
            owner: *signer_public_key,
            staked_amount: 0,
            total_points: 0,
            last_update_time: 0,
            bump: bump,
        });

        msg!("User stake account created successfully");

        Ok(())
    }
}
