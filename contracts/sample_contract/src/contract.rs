use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env};

use curve::Curve;

use crate::storage::{
    get_vesting_from_instance, get_vesting_from_persistent, save_vesting_in_instance,
    save_vesting_in_persistent, VestingInfo,
};
use crate::{error::ContractError, storage::VestingBalance};

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "Phoenix Protocol Vesting");

#[contract]
pub struct Sample;

pub trait SampleTrait {
    fn initialize(env: Env) -> Result<(), ContractError>;

    fn save_vesting_in_persistent(env: Env, user: Address, vesting: VestingInfo);

    fn save_vesting_in_instance(env: Env, user: Address, vesting: VestingInfo);

    fn query_vesting_in_persistent(env: Env, address: Address) -> Result<VestingInfo, ContractError>;

    fn query_vesting_in_instance(env: Env, address: Address) -> Result<VestingInfo, ContractError>;
}

#[contractimpl]
impl SampleTrait for Sample {
    fn initialize(env: Env) -> Result<(), ContractError> {
        env.events()
            .publish(("sample", "initialized at: "), env.ledger().timestamp());

        Ok(())
    }

    fn query_vesting_in_persistent(env: Env, address: Address) -> Result<VestingInfo, ContractError> {
        let curve = get_vesting_from_persistent(&env, &address)?;
        Ok(curve)
    }

    fn query_vesting_in_instance(env: Env, address: Address) -> Result<VestingInfo, ContractError> {
        let curve = get_vesting_from_instance(&env, &address)?;
        Ok(curve)
    }

    fn save_vesting_in_persistent(env: Env, user: Address, vesting: VestingInfo) {
        save_vesting_in_persistent(&env, &user, vesting);
    }

    fn save_vesting_in_instance(env: Env, user: Address, vesting: VestingInfo) {
        save_vesting_in_instance(&env, &user, vesting);
    }
}
