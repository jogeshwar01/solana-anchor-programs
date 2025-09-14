use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::states::{AMMError, AMM};

#[derive(Accounts)]
pub struct Deposit<'info> {
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
    // pub associated_token_program: Program<'info, AssociatedToken>,  needed if init
    #[account(mut)]
    pub signer: Signer<'info>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(
        &mut self,
        quantity_a: u64,
        quantity_b: u64,
        bumps: &DepositBumps,
    ) -> Result<()> {
        require!(quantity_a > 0 && quantity_b > 0, AMMError::InvalidQuantity);

        let amm = self.amm.load()?;
        let tokens_to_issue: u64;
        if amm.lp_supply == 0 {
            // sqrt mean of token deposits
            // first LP sets constant product
            // LP[minted] = Sqrt(qA X qB)

            // Example
            // Deposit - 100 A and 400 B
            // LP[minted] = Sqrt(100 X 400) = 200
            // First LP gets 200 tokens

            let value = (quantity_a as u128) * (quantity_b as u128);
            tokens_to_issue = binary_search_sqrt(value);
        } else {
            require!(
                (quantity_a * self.reserve_b.amount == quantity_b * self.reserve_a.amount),
                AMMError::InvalidLiquidity
            );

            // LP[minted] = min (qA/vA X LP[total], qB/vB X LP[total] )
            // qA, qB - deposit amounts
            // vA, vB - current pool reserves
            // LP[total] - total lp issued - lp_supply

            // we'll check them to be equal - to ensure all quantity is converted correctly

            let lp_tokens_a = amm
                .lp_supply
                .checked_mul(quantity_a)
                .and_then(|v| v.checked_div(self.reserve_a.amount))
                .ok_or(AMMError::ArithmeticOverflow)?;

            let lp_tokens_b = amm
                .lp_supply
                .checked_mul(quantity_b)
                .and_then(|v| v.checked_div(self.reserve_b.amount))
                .ok_or(AMMError::ArithmeticOverflow)?;

            require!(lp_tokens_a == lp_tokens_b, AMMError::InvalidLiquidity);

            tokens_to_issue = lp_tokens_a;
        }
        drop(amm);

        let transfer_to_reserve_a = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.token_a_account.to_account_info(),
                to: self.reserve_a.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        );

        let transfer_to_reserve_b = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.token_b_account.to_account_info(),
                to: self.reserve_b.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        );

        transfer(transfer_to_reserve_a, quantity_a)?;
        transfer(transfer_to_reserve_b, quantity_b)?;

        let token_a_mint_key = self.token_a_mint.key();
        let token_b_mint_key = self.token_b_mint.key();

        let seeds: &[&[u8]; 4] = &[
            b"authority",
            token_a_mint_key.as_ref(),
            token_b_mint_key.as_ref(),
            &[bumps.pool_authority],
        ];
        let signer_seeds = &[&seeds[..]];

        let mint_lp_token_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.lp_mint.to_account_info(),
                to: self.token_lp_account.to_account_info(),
                authority: self.pool_authority.to_account_info(),
            },
            signer_seeds,
        );

        mint_to(mint_lp_token_ctx, tokens_to_issue)?;

        let mut amm = self.amm.load_mut()?;
        amm.lp_supply += tokens_to_issue;

        Ok(())
    }
}

pub fn binary_search_sqrt(value: u128) -> u64 {
    if value < 2 {
        return value as u64;
    }

    let mut left: u128 = 1;
    let mut right: u128 = value;
    let mut ans: u128 = 0;

    while left <= right {
        let mid = left + (right - left) / 2;
        if mid * mid <= value {
            ans = mid;
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }
    ans as u64
}
