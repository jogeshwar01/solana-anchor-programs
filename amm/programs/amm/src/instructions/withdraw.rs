use anchor_lang::prelude::*;
use anchor_spl::token::{
    burn, close_account, transfer, Burn, CloseAccount, Mint, Token, TokenAccount, Transfer,
};

use crate::states::{AMMError, AMM};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds=[b"amm", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
    )]
    pub amm: AccountLoader<'info, AMM>,

    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = signer
    )]
    pub token_a_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = signer
    )]
    pub token_b_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = signer
    )]
    pub token_lp_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"reserve_a", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
        token::mint = token_a_mint,
        token::authority = pool_authority
    )]
    pub reserve_a: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
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
        mut,
        seeds = [b"lp_mint", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,

    #[account(mut)]
    pub signer: Signer<'info>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, lp_token_quantity: u64, bumps: &WithdrawBumps) -> Result<()> {
        // amount_0 = (shares * bal0) / totalSupply
        // amount_1 = (shares * bal1) / totalSupply

        let amm = self.amm.load()?;
        let token_a_to_release = lp_token_quantity
            .checked_mul(self.reserve_a.amount)
            .and_then(|v| v.checked_div(amm.lp_supply))
            .ok_or(AMMError::ArithmeticOverflow)?;

        let token_b_to_release = lp_token_quantity
            .checked_mul(self.reserve_b.amount)
            .and_then(|v| v.checked_div(amm.lp_supply))
            .ok_or(AMMError::ArithmeticOverflow)?;
        drop(amm);

        let lp_token_amount = self.token_lp_account.amount;

        let burn_lp_tokens_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.lp_mint.to_account_info(),
                from: self.token_lp_account.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        );

        burn(burn_lp_tokens_ctx, lp_token_quantity)?;

        let mut amm = self.amm.load_mut()?;
        amm.lp_supply -= lp_token_quantity;
        drop(amm);

        let token_a_mint_key = self.token_a_mint.key();
        let token_b_mint_key = self.token_b_mint.key();

        let seeds: &[&[u8]; 4] = &[
            b"authority",
            token_a_mint_key.as_ref(),
            token_b_mint_key.as_ref(),
            &[bumps.pool_authority],
        ];

        let signer_seeds = &[&seeds[..]];

        let release_token_a_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            Transfer {
                from: self.reserve_a.to_account_info(),
                to: self.token_a_account.to_account_info(),
                authority: self.pool_authority.to_account_info(),
            },
            signer_seeds,
        );

        let release_token_b_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            Transfer {
                from: self.reserve_b.to_account_info(),
                to: self.token_b_account.to_account_info(),
                authority: self.pool_authority.to_account_info(),
            },
            signer_seeds,
        );

        transfer(release_token_a_ctx, token_a_to_release)?;
        transfer(release_token_b_ctx, token_b_to_release)?;

        if lp_token_quantity == lp_token_amount {
            let close_lp_account_ctx = CpiContext::new(
                self.token_program.to_account_info(),
                CloseAccount {
                    account: self.token_lp_account.to_account_info(),
                    destination: self.signer.to_account_info(),
                    authority: self.signer.to_account_info(),
                },
            );

            close_account(close_lp_account_ctx)?;
        }

        Ok(())
    }
}
