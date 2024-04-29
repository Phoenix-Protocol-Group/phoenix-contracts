use curve::Curve;
use soroban_sdk::{log, panic_with_error, Address, Env, Vec};

use crate::{
    error::ContractError,
    storage::{get_vesting, save_vesting, VestingBalance, VestingInfo},
};

pub fn verify_vesting_and_update_balances(
    env: &Env,
    sender: &Address,
    amount: u128,
) -> Result<(), ContractError> {
    let vesting_info = get_vesting(env, sender)?;
    let vested = vesting_info
        .distribution_info
        .get_curve()
        .value(env.ledger().timestamp());

    let sender_balance = vesting_info.balance;
    let sender_liquid = sender_balance // this checks if we can withdraw any vesting
        .checked_sub(vested)
        .ok_or(ContractError::NotEnoughBalance)?;

    if sender_liquid < amount {
        log!(
            &env,
            "Vesting: Verify Vesting Update Balances: Remaining amount must be at least equal to vested amount"
        );
        panic_with_error!(env, ContractError::CantMoveVestingTokens);
    }

    save_vesting(
        env,
        sender,
        &VestingInfo {
            balance: sender_balance - amount,
            distribution_info: vesting_info.distribution_info,
        },
    );

    Ok(())
}

pub fn create_vesting_accounts(
    env: &Env,
    vesting_complexity: u32,
    vesting_accounts: Vec<VestingBalance>,
) -> Result<u128, ContractError> {
    validate_accounts(env, vesting_accounts.clone())?;

    let mut total_vested_amount = 0;

    vesting_accounts.into_iter().for_each(|vb| {
        assert_schedule_vests_amount(
            env,
            &vb.distribution_info.get_curve(),
            vb.distribution_info.amount,
        )
        .expect("Invalid curve and amount");

        if vesting_complexity <= vb.distribution_info.get_curve().size() {
            log!(
                &env,
                "Vesting: Create vesting account: Invalid curve complexity for {}",
                vb.rcpt_address
            );
            panic_with_error!(env, ContractError::VestingComplexityTooHigh);
        }

        save_vesting(
            env,
            &vb.rcpt_address,
            &VestingInfo {
                balance: vb.distribution_info.amount,
                distribution_info: vb.distribution_info.clone(),
            },
        );

        total_vested_amount += vb.distribution_info.amount;
    });

    Ok(total_vested_amount)
}

/// Asserts the vesting schedule decreases to 0 eventually, and is never more than the
/// amount being sent. If it doesn't match these conditions, returns an error.
pub fn assert_schedule_vests_amount(
    env: &Env,
    schedule: &Curve,
    amount: u128,
) -> Result<(), ContractError> {
    schedule.validate_monotonic_decreasing()?;
    let (low, high) = schedule.range();
    if low != 0 {
        log!(
            &env,
            "Vesting: Transfer Vesting: Cannot transfer when non-fully vested"
        );
        panic_with_error!(&env, ContractError::NeverFullyVested)
    } else if high > amount {
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
        if !addresses.contains(&item.rcpt_address) {
            addresses.push_back(item.rcpt_address.clone());
        }
    }

    if addresses.len() != accounts.len() {
        log!(&env, "Vesting: Initialize: Duplicate addresses found");
        panic_with_error!(env, ContractError::DuplicateInitialBalanceAddresses);
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use curve::SaturatingLinear;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::vec;

    use crate::storage::DistributionInfo;

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
                rcpt_address: address1.clone(),
                distribution_info: DistributionInfo {
                    start_timestamp: 15,
                    end_timestamp: 60,
                    amount: 120,
                },
            },
            VestingBalance {
                rcpt_address: address2.clone(),
                distribution_info: DistributionInfo {
                    start_timestamp: 15,
                    end_timestamp: 60,
                    amount: 120,
                },
            },
            VestingBalance {
                rcpt_address: address3.clone(),
                distribution_info: DistributionInfo {
                    start_timestamp: 15,
                    end_timestamp: 60,
                    amount: 120,
                },
            },
        ];

        assert_eq!(validate_accounts(&env, accounts), Ok(()));
    }

    #[test]
    #[should_panic(expected = "Vesting: Initialize: Duplicate addresses found")]
    fn validate_accounts_should_panic() {
        let env = Env::default();
        let duplicate_address = Address::generate(&env);
        let accounts = vec![
            &env,
            VestingBalance {
                rcpt_address: duplicate_address.clone(),
                distribution_info: DistributionInfo {
                    start_timestamp: 15,
                    end_timestamp: 60,
                    amount: 120,
                },
            },
            VestingBalance {
                rcpt_address: duplicate_address,
                distribution_info: DistributionInfo {
                    start_timestamp: 15,
                    end_timestamp: 60,
                    amount: 120,
                },
            },
            VestingBalance {
                rcpt_address: Address::generate(&env),
                distribution_info: DistributionInfo {
                    start_timestamp: 15,
                    end_timestamp: 60,
                    amount: 120,
                },
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
        const AMOUNT: u128 = 1;
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
