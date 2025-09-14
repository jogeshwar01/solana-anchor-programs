use anchor_lang::prelude::*;

#[error_code]
pub enum AMMError {
    #[msg("AMM Pool must be for 2 different mints")]
    SameTokenMint,
    #[msg("Cannot perform that Math operation - Arithmetic Overflow")]
    ArithmeticOverflow,
    #[msg("Invalid Quantity specified")]
    InvalidQuantity,
    #[msg("Invalid Liquidity specified")]
    InvalidLiquidity,
}
