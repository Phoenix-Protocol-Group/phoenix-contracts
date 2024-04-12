use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, Vec};

use curve::Curve;

use crate::storage::{get_vesting_from_instance, get_vesting_from_persistent, save_vesting_in_instance, save_vesting_in_persistent, VestingInfo};
use crate::{error::ContractError, storage::VestingBalance};

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "Phoenix Protocol Vesting");

#[contract]
pub struct Sample;

pub trait SampleTrait {
    fn initialize(env: Env, vesting_balances: Vec<VestingBalance>) -> Result<(), ContractError>;

    fn query_vesting_in_persistent(env: Env, address: Address) -> Result<Curve, ContractError>;

    fn query_vesting_in_instance(env: Env, address: Address) -> Result<Curve, ContractError>;
}

#[contractimpl]
impl SampleTrait for Sample {
    fn initialize(env: Env, vesting_balances: Vec<VestingBalance>) -> Result<(), ContractError> {
        vesting_balances.into_iter().for_each(|vb| {
            save_vesting_in_persistent(
                &env,
                &vb.address,
                VestingInfo {
                    amount: vb.balance,
                    curve: vb.curve.clone(),
                },
            );

            save_vesting_in_instance(
                &env,
                &vb.address,
                VestingInfo {
                    amount: vb.balance,
                    curve: vb.curve,
                }
            );
        });

        Ok(())
    }

    fn query_vesting_in_persistent(env: Env, address: Address) -> Result<Curve, ContractError> {
        let curve = get_vesting_from_persistent(&env, &address)?.curve;
        Ok(curve)
    }

    fn query_vesting_in_instance(env: Env, address: Address) -> Result<Curve, ContractError> {
        let curve = get_vesting_from_instance(&env, &address)?.curve;
        Ok(curve)
    }
}
