use curve::Curve;
use soroban_sdk::testutils::arbitrary::std::dbg;
use soroban_sdk::{log, panic_with_error, Address, Env, Vec};

use crate::{
    error::ContractError,
    storage::{
        get_config, get_vesting, remove_vesting, save_balance, save_vesting, VestingBalance,
        VestingInfo,
    },
    token_contract,
};

pub fn verify_vesting_and_transfer(
    env: &Env,
    sender: &Address,
    to: &Address,
    amount: i128,
) -> Result<(), ContractError> {
    let vesting_amount = get_vesting(env, sender)
        .curve
        .value(env.ledger().timestamp()) as i128;

    if vesting_amount <= 0 {
        remove_vesting(env, sender);
    }

    let token_client = token_contract::Client::new(env, &get_config(env).token_info.address);
    let balance = token_client.balance(sender);
    let remainder = balance
        .checked_sub(amount)
        .ok_or(ContractError::NotEnoughBalance)?;

    if vesting_amount > remainder {
        log!(
            &env,
            "Vesting: Mixture: Remaining amount must be at least equal to vested amount"
        );
        panic_with_error!(env, ContractError::CantMoveVestingTokens);
    }

    token_client.transfer(sender, to, &amount);

    Ok(())
}

pub fn create_vesting_accounts(
    env: &Env,
    vesting_complexity: u32,
    vesting_accounts: Vec<VestingBalance>,
) -> Result<i128, ContractError> {
    validate_accounts(env, vesting_accounts.clone())?;

    let mut total_supply = 0;

    vesting_accounts.into_iter().for_each(|vb| {
        assert_schedule_vests_amount(&vb.curve, vb.balance as u128)
            .expect("Invalid curve and amount");

        if vesting_complexity <= vb.curve.size() {
            log!(
                &env,
                "Vesting: Create vesting account: Invalid curve complexity for {}",
                vb.address
            );
            panic_with_error!(env, ContractError::VestingComplexityTooHigh);
        }

        save_vesting(
            env,
            &vb.address,
            VestingInfo {
                amount: vb.balance,
                curve: vb.curve,
            },
        );

        save_balance(env, &vb.address, vb.balance);
        total_supply += vb.balance;
    });

    Ok(total_supply)
}

/// Asserts the vesting schedule decreases to 0 eventually, and is never more than the
/// amount being sent. If it doesn't match these conditions, returns an error.
pub fn assert_schedule_vests_amount(schedule: &Curve, amount: u128) -> Result<(), ContractError> {
    dbg!(schedule, amount);
    schedule.validate_monotonic_decreasing()?;
    let (low, high) = schedule.range();
    dbg!(low, high, amount);
    if low != 0 {
        Err(ContractError::NeverFullyVested)
    } else if high > amount {
        Err(ContractError::VestsMoreThanSent)
    } else {
        Ok(())
    }
}

fn validate_accounts(env: &Env, accounts: Vec<VestingBalance>) -> Result<(), ContractError> {
    let mut addresses: Vec<Address> = Vec::new(env);

    for item in accounts.iter() {
        if !addresses.contains(&item.address) {
            addresses.push_back(item.address.clone());
        }
    }

    if addresses.len() != accounts.len() {
        log!(&env, "Vesting: Initialize: Duplicate addresses found");
        panic_with_error!(env, ContractError::DuplicateInitialBalanceAddresses);
    } else {
        Ok(())
    }
}
