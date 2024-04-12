use curve::Curve;
use soroban_sdk::{
    contracttype, log, panic_with_error, Address, ConversionError, Env, TryFromVal, Val,
};

use crate::error::ContractError;

// impl TryFromVal<Env, DataKey> for Val {
//     type Error = ConversionError;

//     fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
//         Ok((*v as u32).into())
//     }
// }

// #[derive(Clone, Copy)]
// #[repr(u32)]
// pub enum DataKey {
//     Admin = 1,
//     Config = 2,
//     Minter = 3,
// }

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingBalance {
    pub address: Address,
    pub balance: i128,
    pub curve: Curve,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingInfo {
    pub amount: i128,
    pub curve: Curve,
}

pub fn save_vesting_in_persistent(env: &Env, address: &Address, vesting_info: VestingInfo) {
    env.storage().persistent().set(address, &vesting_info);
}

pub fn save_vesting_in_instance(env: &Env, address: &Address, vesting_info: VestingInfo) {
    env.storage().instance().set(address, &vesting_info);
}

pub fn get_vesting_from_persistent(
    env: &Env,
    address: &Address,
) -> Result<VestingInfo, ContractError> {
    let vesting_info = env.storage().persistent().get(address).unwrap_or_else(|| {
        log!(&env, "Vesting: Get vesting schedule: Critical error - No vesting schedule found for the given address");
        panic_with_error!(env, ContractError::VestingNotFoundForAddress);
    });

    Ok(vesting_info)
}

pub fn get_vesting_from_instance(
    env: &Env,
    address: &Address,
) -> Result<VestingInfo, ContractError> {
    let vesting_info = env.storage().instance().get(address).unwrap_or_else(|| {
        log!(&env, "Vesting: Get vesting schedule: Critical error - No vesting schedule found for the given address");
        panic_with_error!(env, ContractError::VestingNotFoundForAddress);
    });

    Ok(vesting_info)
}
