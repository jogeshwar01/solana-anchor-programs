use crate::{
    constants::{LAMPORTS_PER_SOL, POINTS_PER_SOL_PER_DAY, SECONDS_PER_DAY},
    states::{error::StakeError, stake_account::StakeAccount},
};
use anchor_lang::prelude::*;

pub fn update_points(pda_account: &mut StakeAccount, current_time: i64) -> Result<()> {
    let time_elapsed = current_time
        .checked_sub(pda_account.last_update_time)
        .ok_or(StakeError::InvalidTimestamp)? as u64;

    if time_elapsed > 0 && pda_account.staked_amount > 0 {
        let new_points = calculate_points_earned(pda_account.staked_amount, time_elapsed)?;
        pda_account.total_points = pda_account
            .total_points
            .checked_add(new_points)
            .ok_or(StakeError::Overflow)?;
    }

    pda_account.last_update_time = current_time;
    Ok(())
}

pub fn calculate_points_earned(staked_amount: u64, time_elapsed_seconds: u64) -> Result<u64> {
    // Points = (staked_amount_in_sol * time_in_days * points_per_sol_per_day)
    // Using micro-points for precision to avoid floating point
    let points = (staked_amount as u128)
        .checked_mul(time_elapsed_seconds as u128)
        .ok_or(StakeError::Overflow)?
        .checked_mul(POINTS_PER_SOL_PER_DAY as u128)
        .ok_or(StakeError::Overflow)?
        .checked_div(LAMPORTS_PER_SOL as u128)
        .ok_or(StakeError::Overflow)?
        .checked_div(SECONDS_PER_DAY as u128)
        .ok_or(StakeError::Overflow)?;

    Ok(points as u64)
}
