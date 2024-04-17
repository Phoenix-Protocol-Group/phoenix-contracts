use curve::Curve;
use soroban_sdk::{
    contracttype, log, panic_with_error, Address, ConversionError, Env, String, TryFromVal, Val,
    Vec,
};

use crate::error::ContractError;

impl TryFromVal<Env, DataKey> for Val {
    type Error = ConversionError;

    fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
        Ok((*v as u32).into())
    }
}

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    Admin = 1,
    Config = 2,
    Minter = 3,
    Whitelist = 4,
    VestingTokenInfo = 5,
    MaxVestingComplexity = 6,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    /// `max_vesting_complexity` the maximum complexity an account's vesting curve is allowed to have
    pub max_vesting_complexity: u32,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingTokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub address: Address,
    pub total_supply: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingBalance {
    pub address: Address,
    pub balance: i128,
    pub curve: Curve,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinterInfo {
    pub address: Address,
    pub capacity: Curve,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingInfo {
    pub amount: i128,
    pub curve: Curve,
}

pub fn save_config(env: &Env, config: &Config) {
    env.storage().persistent().set(&DataKey::Config, config);
}

pub fn get_config(env: &Env) -> Config {
    env.storage()
        .persistent()
        .get(&DataKey::Config)
        .unwrap_or_else(|| {
            log!(
                &env,
                "Vesting: Get config: Critical error - No config found"
            );
            panic_with_error!(env, ContractError::NoConfigFound);
        })
}

pub fn save_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .persistent()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| {
            log!(&env, "Vesting: Get admin: Critical error - No admin found");
            panic_with_error!(env, ContractError::NoAdminFound);
        })
}

pub fn save_balance(env: &Env, address: &Address, balance: i128) {
    env.storage().persistent().set(address, &balance);
}

pub fn save_vesting(env: &Env, address: &Address, vesting_info: VestingInfo) {
    env.storage().instance().set(address, &vesting_info);
}

pub fn get_vesting(env: &Env, address: &Address) -> Result<VestingInfo, ContractError> {
    // FIXME why does this throws an error when we try to access the persistent storage?
    let vesting_info = env.storage().instance().get(address).unwrap_or_else(|| {
        log!(&env, "Vesting: Get vesting schedule: Critical error - No vesting schedule found for the given address");
        panic_with_error!(env, ContractError::VestingNotFoundForAddress);
    });

    Ok(vesting_info)
}

pub fn remove_vesting(env: &Env, address: &Address) {
    env.storage().persistent().remove(&address);
}

// TODO: uncomment when needed
// pub fn get_allowances(env: &Env, owner_spender: &(Address, Address)) -> i128 {
//     env.storage().persistent().get(owner_spender).unwrap_or_else(|| {
//             log!(&env, "Vesting: Get allowance: Critical error - No allowance found for the given address pair");
//             panic_with_error!(env, ContractError::AllowanceNotFoundForGivenPair);
//         })
// }

// pub fn save_allowances(env: &Env, owner_spender: &(Address, Address), amount: i128) {
//     env.storage().persistent().set(owner_spender, &amount);
// }

pub fn save_minter(env: &Env, minter: MinterInfo) {
    env.storage().persistent().set(&DataKey::Minter, &minter);
}

pub fn get_minter(env: &Env) -> MinterInfo {
    env.storage()
        .persistent()
        .get(&DataKey::Minter)
        .unwrap_or_else(|| {
            log!(
                &env,
                "Vesting: Get minter: Critical error - No minter found "
            );
            panic_with_error!(env, ContractError::MinterNotFound);
        })
}

pub fn get_vesting_total_supply(env: &Env) -> i128 {
    get_token_info(env).total_supply
}

pub fn update_vesting_total_supply(env: &Env, amount: i128) {
    let mut token_info = get_token_info(env);
    token_info.total_supply = amount;
    save_token_info(env, &token_info);
}

pub fn save_whitelist(env: &Env, whitelist: &Vec<Address>) {
    env.storage()
        .persistent()
        .set(&DataKey::Whitelist, whitelist);
}

pub fn get_whitelist(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::Whitelist)
        .unwrap_or_else(|| {
            log!(
                &env,
                "Vesting: Get whitelist: Critical error - No whitelist found"
            );
            panic_with_error!(env, ContractError::NoWhitelistFound);
        })
}

pub fn save_token_info(env: &Env, token_info: &VestingTokenInfo) {
    env.storage()
        .persistent()
        .set(&DataKey::VestingTokenInfo, token_info);
}

pub fn get_token_info(env: &Env) -> VestingTokenInfo {
    env.storage()
        .persistent()
        .get(&DataKey::VestingTokenInfo)
        .unwrap_or_else(|| {
            log!(
                &env,
                "Vesting: Get token info: Critical error - No token info found"
            );
            panic_with_error!(env, ContractError::NoTokenInfoFound);
        })
}
