use curve::Curve;
use soroban_sdk::{log, panic_with_error, Address, Env, Vec};

use crate::{
    error::ContractError,
    storage::{
        get_max_vesting_complexity, get_vesting, remove_vesting, save_balance, save_vesting,
        VestingBalance, VestingInfo,
    },
    token_contract,
};

pub fn verify_vesting(
    env: &Env,
    sender: &Address,
    amount: i128,
    token_client: &token_contract::Client,
) -> Result<(), ContractError> {
    let vesting_amount = get_vesting(env, sender)?
        .curve
        .value(env.ledger().timestamp()) as i128;

    if vesting_amount <= 0 {
        remove_vesting(env, sender);
    }

    let sender_balance = token_client.balance(sender);
    let sender_remainder = sender_balance
        .checked_sub(amount)
        .ok_or(ContractError::NotEnoughBalance)?;

    if vesting_amount > sender_remainder {
        log!(
            &env,
            "Vesting: Verity Vesting: Remaining amount must be at least equal to vested amount"
        );
        panic_with_error!(env, ContractError::CantMoveVestingTokens);
    }

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
        assert_schedule_vests_amount(env, &vb.curve, vb.balance).expect("Invalid curve and amount");

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
            &VestingInfo {
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
pub fn assert_schedule_vests_amount(
    env: &Env,
    schedule: &Curve,
    amount: i128,
) -> Result<(), ContractError> {
    schedule.validate_monotonic_decreasing()?;
    let (low, high) = schedule.range();
    if low != 0 {
        log!(
            &env,
            "Vesting: Transfer Vesting: Cannot transfer when non-fully vested"
        );
        panic_with_error!(&env, ContractError::NeverFullyVested)
    } else if high as i128 > amount {
        log!(
            &env,
            "Vesting: Assert Schedule Vest Amount: Vesting amount more than sent"
        );
        panic_with_error!(&env, ContractError::VestsMoreThanSent)
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

pub fn update_vesting(
    env: &Env,
    to_address: &Address,
    amount: i128,
    new_curve: Curve,
) -> Result<(), ContractError> {
    let max_complexity = get_max_vesting_complexity(env);
    env.storage()
        .persistent()
        .update(&to_address, |current_value: Option<Curve>| {
            let new_curve_schedule = current_value
                .map(|current_value| current_value.combine(env, &new_curve))
                .unwrap_or(new_curve);

            new_curve_schedule
                .validate_complexity(max_complexity)
                .unwrap_or_else(|_| {
                    log!(
                        &env,
                        "Vesting: Update Vesting: new vesting complexity invalid"
                    );
                    panic_with_error!(env, ContractError::VestingComplexityTooHigh);
                });

            save_vesting(
                env,
                to_address,
                &VestingInfo {
                    amount,
                    curve: new_curve_schedule.clone(),
                },
            );

            new_curve_schedule
        });

    Ok(())
}
#[cfg(test)]
mod test {
    use curve::SaturatingLinear;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::vec;

    use super::*;

    #[test]
    fn validate_accounts_works() {
        let env = Env::default();
        let address1 = Address::generate(&env);
        let address2 = Address::generate(&env);
        let address3 = Address::generate(&env);

        let accounts = vec![
            &env,
            VestingBalance {
                address: address1.clone(),
                balance: 100,
                curve: Curve::SaturatingLinear(SaturatingLinear {
                    min_x: 15,
                    min_y: 120,
                    max_x: 60,
                    max_y: 0,
                }),
            },
            VestingBalance {
                address: address2.clone(),
                balance: 200,
                curve: Curve::SaturatingLinear(SaturatingLinear {
                    min_x: 15,
                    min_y: 120,
                    max_x: 60,
                    max_y: 0,
                }),
            },
            VestingBalance {
                address: address3.clone(),
                balance: 300,
                curve: Curve::SaturatingLinear(SaturatingLinear {
                    min_x: 15,
                    min_y: 120,
                    max_x: 60,
                    max_y: 0,
                }),
            },
        ];

        assert_eq!(validate_accounts(&env, accounts), Ok(()));
    }

    #[test]
    #[should_panic(expected = "Vesting: Initialize: Duplicate addresses found")]
    fn validate_accounts_should_panic() {
        let env = Env::default();
        let address1 = Address::generate(&env);
        let address2 = Address::generate(&env);

        let accounts = vec![
            &env,
            VestingBalance {
                address: address1.clone(),
                balance: 100,
                curve: Curve::SaturatingLinear(SaturatingLinear {
                    min_x: 15,
                    min_y: 120,
                    max_x: 60,
                    max_y: 0,
                }),
            },
            VestingBalance {
                address: address2.clone(),
                balance: 200,
                curve: Curve::SaturatingLinear(SaturatingLinear {
                    min_x: 15,
                    min_y: 120,
                    max_x: 60,
                    max_y: 0,
                }),
            },
            VestingBalance {
                address: address1.clone(),
                balance: 300,
                curve: Curve::SaturatingLinear(SaturatingLinear {
                    min_x: 15,
                    min_y: 120,
                    max_x: 60,
                    max_y: 0,
                }),
            },
        ];

        validate_accounts(&env, accounts).unwrap_err();
    }

    #[test]
    fn assert_schedule_vests_amount_works() {
        let env = Env::default();
        let curve = Curve::SaturatingLinear(SaturatingLinear {
            min_x: 15,
            min_y: 120,
            max_x: 60,
            max_y: 0,
        });

        assert_eq!(assert_schedule_vests_amount(&env, &curve, 121), Ok(()));
    }

    #[test]
    #[should_panic(expected = "Vesting: Transfer Vesting: Cannot transfer when non-fully vested")]
    fn assert_schedule_vests_amount_fails_when_low_not_zero() {
        const MIN_NOT_ZERO: u128 = 1;
        let env = Env::default();
        let curve = Curve::SaturatingLinear(SaturatingLinear {
            min_x: 15,
            min_y: 120,
            max_x: 60,
            max_y: MIN_NOT_ZERO,
        });

        assert_schedule_vests_amount(&env, &curve, 1_000).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Vesting: Assert Schedule Vest Amount: Vesting amount more than sent"
    )]
    fn assert_schedule_vests_amount_fails_when_high_bigger_than_amount() {
        const HIGH: u128 = 2;
        const AMOUNT: i128 = 1;
        let env = Env::default();
        let curve = Curve::SaturatingLinear(SaturatingLinear {
            min_x: 15,
            min_y: HIGH,
            max_x: 60,
            max_y: 0,
        });

        assert_schedule_vests_amount(&env, &curve, AMOUNT).unwrap();
    }
}
