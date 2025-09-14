use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::states::{AMMError, AMM};

#[derive(Accounts)]
pub struct Swap<'info> {
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

    pub token_program: Program<'info, Token>,

    #[account(mut)]
    pub signer: Signer<'info>,
}

impl<'info> Swap<'info> {
    pub fn swap(&mut self, quantity: u64, is_a: bool, bumps: &SwapBumps) -> Result<()> {
        require!(quantity > 0, AMMError::InvalidQuantity);

        let (from_reserve, to_reserve, from_token_account, to_token_account) = if is_a {
            (
                &self.reserve_a,
                &self.reserve_b,
                &self.token_a_account,
                &self.token_b_account,
            )
        } else {
            (
                &self.reserve_b,
                &self.reserve_a,
                &self.token_b_account,
                &self.token_a_account,
            )
        };

        // CONSTANT PRODUCT AMM
        // xy = k
        // (x + dx)(y - dy) = k
        // y - dy = k / (x + dx)
        // y - k/(x + dx) = dy
        // y - xy(x + dx) = dy
        // (yx + ydx - xy)/(x + dx) = dy
        // ydx /(x + dx) = dy

        // here,
        // dx = quantity
        // dy = other token quantity
        // x = to_reserve_amount
        // y = from_reserve_amount

        // amountOut = (reserveOut * amountIn) / (reserveIn + amountIn)

        let other_token_quantity = from_reserve
            .amount
            .checked_mul(quantity)
            .and_then(|p| p.checked_div(to_reserve.amount.checked_add(quantity)?))
            .ok_or(AMMError::ArithmeticOverflow)?;

        let transfer_to_reserve = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: from_token_account.to_account_info(),
                to: to_reserve.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        );

        transfer(transfer_to_reserve, quantity)?;

        let token_a_mint_key = self.token_a_mint.key();
        let token_b_mint_key = self.token_b_mint.key();

        let seeds: &[&[u8]; 4] = &[
            b"authority",
            token_a_mint_key.as_ref(),
            token_b_mint_key.as_ref(),
            &[bumps.pool_authority],
        ];

        let signer_seeds = &[&seeds[..]];

        let transfer_to_user = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            Transfer {
                from: from_reserve.to_account_info(),
                to: to_token_account.to_account_info(),
                authority: self.pool_authority.to_account_info(),
            },
            signer_seeds,
        );

        transfer(transfer_to_user, other_token_quantity)?;

        Ok(())
    }
}
