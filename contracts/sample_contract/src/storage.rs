use curve::Curve;
use soroban_sdk::{
    contracttype, log, panic_with_error, Address, ConversionError, Env, TryFromVal, Val,
};

use crate::error::ContractError;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingBalance {
    pub address: Address,
    pub balance: i128,
    pub curve: Curve,
}

pub fn save_vesting_in_persistent(env: &Env, address: &Address, vesting_balance: VestingBalance) {
    env.storage().persistent().set(address, &vesting_balance);
}

pub fn save_vesting_in_instance(env: &Env, address: &Address, vesting_balance: VestingBalance) {
    env.storage().instance().set(address, &vesting_balance);
}

pub fn get_vesting_from_persistent(
    env: &Env,
    address: &Address,
) -> Result<VestingBalance, ContractError> {
    let vesting_info = env.storage().persistent().get(address).unwrap_or_else(|| {
        log!(&env, "Vesting: Get vesting schedule: Critical error - No vesting schedule found for the given address");
        panic_with_error!(env, ContractError::VestingNotFoundForAddress);
    });

    Ok(vesting_info)
}

pub fn get_vesting_from_instance(
    env: &Env,
    address: &Address,
) -> Result<VestingBalance, ContractError> {
    let vesting_info = env.storage().instance().get(address).unwrap_or_else(|| {
        log!(&env, "Vesting: Get vesting schedule: Critical error - No vesting schedule found for the given address");
        panic_with_error!(env, ContractError::VestingNotFoundForAddress);
    });

    Ok(vesting_info)
}
