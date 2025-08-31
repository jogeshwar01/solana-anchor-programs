use crate::{
    contexts::utils::update_points,
    states::{constants::TOKEN_DECIMALS, error::StakeError, stake_account::StakeAccount},
};
use anchor_spl::token::{self as token, Mint, Token, TokenAccount};

use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut, seeds = [b"stake_account", signer.key().as_ref()], bump = user_stake_account.bump)]
    user_stake_account: Account<'info, StakeAccount>,
    #[account(mut)]
    signer: Signer<'info>,
    #[account(
        mut,
        constraint = reward_mint.mint_authority.unwrap() == mint_authority.key() @ StakeError::InvalidMintAuthority
    )]
    reward_mint: Account<'info, Mint>,
    /// CHECK: This account is used as a mint authority and is validated by the seeds constraint
    #[account(seeds = [b"mint_authority"], bump)]
    mint_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        constraint = user_token_account.mint == reward_mint.key() @ StakeError::InvalidTokenAccount,
        constraint = user_token_account.owner == signer.key() @ StakeError::InvalidOwner
    )]
    user_token_account: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
}

impl<'info> ClaimReward<'info> {
    pub fn claim_reward(&mut self, authority_bump: u8) -> Result<()> {
        let user_stake_account = &mut self.user_stake_account;
        let clock = Clock::get()?;

        update_points(user_stake_account, clock.unix_timestamp)?;

        let tokens_to_mint = user_stake_account.total_points;
        require!(tokens_to_mint > 0, StakeError::InsufficientTokenPoints);

        let reward_amount = tokens_to_mint * 10u64.pow(TOKEN_DECIMALS as u32);

        let authority_seeds = &[b"mint_authority".as_ref(), &[authority_bump]];

        token::mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::MintTo {
                    mint: self.reward_mint.to_account_info(),
                    to: self.user_token_account.to_account_info(),
                    authority: self.mint_authority.to_account_info(),
                },
                &[authority_seeds],
            ),
            reward_amount,
        )?;

        user_stake_account.total_points = user_stake_account.total_points;
        user_stake_account.last_update_time = clock.unix_timestamp;

        msg!("Minted {} reward tokens to user", tokens_to_mint);
        Ok(())
    }
}
