use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub staked_amount: u64,
    pub total_points: u64,
    pub last_update_time: i64,
    pub bump: u8,
}

// impl Space for StakeAccount {
//     const INIT_SPACE: usize = 8 + 32 + 8 + 8 + 8 + 1;
// }

// 2 options
// 1) use #[derive(InitSpace)] on struct StakeAccount and then use 8 + StakeAccount::INIT_SPACE directly
// 2) impl Space for StakeAccount
