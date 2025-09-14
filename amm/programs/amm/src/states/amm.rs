use anchor_lang::prelude::*;

#[account(zero_copy)]
#[repr(C)]
pub struct AMM {
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub lp_bump: u8,
    pub _padding: [u8; 7], // Padding for alignment
    pub reserve_a: Pubkey,
    pub reserve_b: Pubkey,
    pub pool_authority: Pubkey,
    pub lp_supply: u64,
}
