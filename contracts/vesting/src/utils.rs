use curve::Curve;
use soroban_sdk::{log, panic_with_error, Address, Env, Vec};

use crate::{error::ContractError, storage::VestingSchedule};

pub fn check_duplications(env: &Env, accounts: Vec<VestingSchedule>) {
    let mut addresses: Vec<Address> = Vec::new(env);
    for account in accounts.iter() {
        if addresses.contains(&account.recipient) {
            log!(&env, "Vesting: Initialize: Duplicate addresses found");
            panic_with_error!(env, ContractError::DuplicateInitialBalanceAddresses);
        }
        addresses.push_back(account.recipient.clone());
    }
}

/// Asserts the vesting schedule decreases to 0 eventually
/// returns the total vested amount
pub fn validate_vesting_schedule(env: &Env, schedule: &Curve) -> Result<u128, ContractError> {
    schedule.validate_monotonic_decreasing()?;
    match schedule {
        Curve::Constant(_) => {
            log!(
                &env,
                "Vesting: Constant curve is not valid for a vesting schedule"
            );
            panic_with_error!(&env, ContractError::CurveConstant)
        }
        Curve::SaturatingLinear(sl) => {
            // Check range
            let (low, high) = (sl.max_y, sl.min_y);
            if low != 0 {
                log!(
                    &env,
                    "Vesting: Transfer Vesting: Cannot transfer when non-fully vested"
                );
                panic_with_error!(&env, ContractError::NeverFullyVested)
            } else {
                Ok(high) // return the total amount to be transferred
            }
        }
        Curve::PiecewiseLinear(pl) => {
            // Check the last step value
            if pl.end_value().unwrap() != 0 {
                log!(
                    &env,
                    "Vesting: Transfer Vesting: Cannot transfer when non-fully vested"
                );
                panic_with_error!(&env, ContractError::NeverFullyVested)
            }

            // Return the amount to be distributed (value of the first step)
            Ok(pl.first_value().unwrap())
        }
    }
}

#[cfg(test)]
mod test {
    use curve::{PiecewiseLinear, SaturatingLinear, Step};
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::vec;

    use super::*;

    #[test]
    fn check_duplications_works() {
        let env = Env::default();
        let address1 = Address::generate(&env);
        let address2 = Address::generate(&env);
        let address3 = Address::generate(&env);

        let accounts = vec![
            &env,
            VestingSchedule {
                recipient: address1.clone(),
                curve: Curve::Constant(1),
            },
            VestingSchedule {
                recipient: address2.clone(),
                curve: Curve::Constant(1),
            },
            VestingSchedule {
                recipient: address3.clone(),
                curve: Curve::Constant(1),
            },
        ];

        // not panicking should be enough to pass the test
        check_duplications(&env, accounts);
    }

    #[test]
    #[should_panic(expected = "Vesting: Initialize: Duplicate addresses found")]
    fn check_duplications_should_panic() {
        let env = Env::default();
        let duplicate_address = Address::generate(&env);
        let accounts = vec![
            &env,
            VestingSchedule {
                recipient: duplicate_address.clone(),
                curve: Curve::Constant(1),
            },
            VestingSchedule {
                recipient: Address::generate(&env),
                curve: Curve::Constant(1),
            },
            VestingSchedule {
                recipient: duplicate_address,
                curve: Curve::Constant(1),
            },
        ];

        check_duplications(&env, accounts);
    }

    #[test]
    fn validate_saturating_linear_vesting() {
        let env = Env::default();
        let curve = Curve::SaturatingLinear(SaturatingLinear {
            min_x: 15,
            min_y: 120,
            max_x: 60,
            max_y: 0,
        });

        assert_eq!(validate_vesting_schedule(&env, &curve), Ok(120));
    }

    #[test]
    fn validate_piecewise_linear_vesting() {
        let env = Env::default();
        let curve = Curve::PiecewiseLinear(PiecewiseLinear {
            steps: vec![
                &env,
                Step {
                    time: 60,
                    value: 120,
                },
                Step {
                    time: 120,
                    value: 0,
                },
            ],
        });

        assert_eq!(validate_vesting_schedule(&env, &curve), Ok(120));
    }

    #[test]
    #[should_panic(expected = "Vesting: Transfer Vesting: Cannot transfer when non-fully vested")]
    fn saturating_linear_schedule_fails_when_not_fully_vested() {
        let env = Env::default();
        let curve = Curve::SaturatingLinear(SaturatingLinear {
            min_x: 15,
            min_y: 120,
            max_x: 60,
            max_y: 1, // leave 1 token at the end
        });

        validate_vesting_schedule(&env, &curve).unwrap();
    }

    #[test]
    #[should_panic(expected = "Vesting: Transfer Vesting: Cannot transfer when non-fully vested")]
    fn piecewise_linear_schedule_fails_when_not_fully_vested() {
        let env = Env::default();
        let curve = Curve::PiecewiseLinear(PiecewiseLinear {
            steps: vec![
                &env,
                Step {
                    time: 60,
                    value: 120,
                },
                Step {
                    time: 120,
                    value: 10,
                },
            ],
        });

        validate_vesting_schedule(&env, &curve).unwrap();
    }
}
