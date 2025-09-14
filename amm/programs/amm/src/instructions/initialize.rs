use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::states::{AMMError, AMM};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer=signer,
        space=8 + std::mem::size_of::<AMM>(),
        seeds=[b"amm", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
        constraint = token_a_mint.key() != token_b_mint.key() 
            @ AMMError::SameTokenMint,
    )]
    pub amm: AccountLoader<'info, AMM>,

    #[account(
        init,
        payer = signer,
        seeds = [b"reserve_a", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
        token::mint = token_a_mint,
        token::authority = pool_authority
    )]
    pub reserve_a: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = signer,
        seeds = [b"reserve_b", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
        token::mint = token_b_mint,
        token::authority = pool_authority
    )]
    pub reserve_b: Box<Account<'info, TokenAccount>>,

    /// CHECK: pool authority over token reserves and lp mint
    #[account(
        seeds=[b"authority", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()], 
        bump
    )]
    pub pool_authority: UncheckedAccount<'info>,

    pub token_a_mint: Box<Account<'info, Mint>>,
    pub token_b_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = signer,
        seeds = [b"lp_mint", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = pool_authority,
        mint::freeze_authority = pool_authority
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,

    #[account(mut)]
    pub signer: Signer<'info>,
}

impl<'info> Initialize<'info> {
    pub fn initialize_amm_pool(&mut self, bumps: &InitializeBumps) -> Result<()> {
        let mut amm = self.amm.load_init()?;
        amm.token_a_mint = self.token_a_mint.key();
        amm.token_b_mint = self.token_b_mint.key();
        amm.lp_mint = self.lp_mint.key();
        amm.lp_bump = bumps.lp_mint;
        amm.reserve_a = self.reserve_a.key();
        amm.reserve_b = self.reserve_b.key();
        amm.pool_authority = self.pool_authority.key();
        amm.lp_supply = 0;

        Ok(())
    }
}
