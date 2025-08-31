use crate::{
    constants::{LAMPORTS_PER_SOL, POINTS_PER_SOL_PER_EPOCH},
    states::{error::StakeError, stake_account::StakeAccount},
};
use anchor_lang::prelude::*;

pub fn update_points(pda_account: &mut StakeAccount, current_epoch: u64) -> Result<()> {
    let epochs_elapsed = current_epoch
        .checked_sub(pda_account.last_update_epoch)
        .ok_or(StakeError::InvalidEpoch)?;

    if epochs_elapsed > 0 && pda_account.staked_amount > 0 {
        let new_points = calculate_points_earned(pda_account.staked_amount, epochs_elapsed)?;
        pda_account.total_points = pda_account
            .total_points
            .checked_add(new_points)
            .ok_or(StakeError::Overflow)?;
    }

    pda_account.last_update_epoch = current_epoch;
    Ok(())
}

pub fn calculate_points_earned(staked_amount: u64, epochs_elapsed: u64) -> Result<u64> {
    // Example: 1 point per SOL per epoch
    // Use u128 internally to prevent overflow
    let points = (staked_amount as u128)
        .checked_mul(epochs_elapsed as u128)
        .ok_or(StakeError::Overflow)?
        .checked_div(LAMPORTS_PER_SOL as u128) // convert lamports → SOL
        .ok_or(StakeError::Overflow)?
        .checked_mul(POINTS_PER_SOL_PER_EPOCH as u128) // configurable constant
        .ok_or(StakeError::Overflow)?;

    Ok(points as u64)
}
