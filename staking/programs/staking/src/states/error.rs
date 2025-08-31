use anchor_lang::prelude::*;

#[error_code]
pub enum StakeError {
    #[msg("Amount must be greater than 0")]
    InvalidAmount,
    #[msg("Insufficient staked amount")]
    InsufficientStake,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Arithmetic underflow")]
    Underflow,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    #[msg("Insufficient points to claim tokens")]
    InsufficientTokenPoints,
    #[msg("Invalid mint authority")]
    InvalidMintAuthority,
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Invalid owner")]
    InvalidOwner
}
