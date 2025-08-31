use crate::{
    contexts::utils::update_points,
    states::{constants::TOKEN_DECIMALS, error::StakeError, stake_account::StakeAccount},
};
use anchor_spl::token::{self as token, Mint, Token, TokenAccount};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(
        mut,
        seeds = [b"stake_account", signer.key().as_ref()],
        bump = user_stake_account.bump
    )]
    pub user_stake_account: Account<'info, StakeAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        constraint = reward_mint.mint_authority.unwrap() == mint_authority.key() 
            @ StakeError::InvalidMintAuthority
    )]
    pub reward_mint: Account<'info, Mint>,
    /// CHECK: PDA validated by seeds - [b"mint_authority", &[bump]]
    #[account(seeds = [b"mint_authority"], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        constraint = user_token_account.mint == reward_mint.key() 
            @ StakeError::InvalidTokenAccount,
        constraint = user_token_account.owner == signer.key() 
            @ StakeError::InvalidOwner
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> ClaimReward<'info> {
    pub fn claim_reward(&mut self, bumps: &ClaimRewardBumps) -> Result<()> {
        let user_stake_account = &mut self.user_stake_account;
        let clock = Clock::get()?;

        update_points(user_stake_account, clock.epoch)?;

        // Rewards = accumulated points
        let tokens_to_mint = user_stake_account.total_points;
        require!(
            tokens_to_mint > 0,
            StakeError::InsufficientTokenPoints
        );

        // Convert points into token amount
        let reward_amount = tokens_to_mint
            .checked_mul(10u64.pow(TOKEN_DECIMALS as u32))
            .ok_or(StakeError::Overflow)?;

        let authority_seeds: &[&[u8]] = &[
            b"mint_authority",
            &[bumps.mint_authority],
        ];

        // Mint tokens to user
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

        // Reset points after claiming
        user_stake_account.total_points = 0;
        user_stake_account.last_update_epoch = clock.epoch;

        msg!(
            "Minted {} reward tokens to {}",
            reward_amount,
            self.signer.key()
        );

        Ok(())
    }
}
