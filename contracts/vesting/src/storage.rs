use core::ops::Add;

use curve::Curve;
use soroban_sdk::{
    contracttype, log, panic_with_error, symbol_short, Address, ConversionError, Env, String,
    Symbol, TryFromVal, Val, Vec,
};

use crate::error::ContractError;

// const CONFIG_KEY: Symbol = symbol_short!("config");
// const ADMIN: Symbol = symbol_short!("admin");

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
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    /// `admin` who can manage the contract with administrative privileges.
    pub admin: Address,
    /// `whitelist` list of addresses that can interact with the contract.
    pub whitelist: Vec<Address>,
    /// `token_info` information about the token used in the vesting contract.
    pub token_info: VestingTokenInfo,
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
    pub cap: Curve,
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

pub fn save_vesting(env: &Env, address: &Address, balance_curve: (i128, Curve)) {
    env.storage().persistent().set(address, &balance_curve);
}

pub fn get_vesting(env: &Env, address: &Address) -> (i128, Curve) {
    env.storage().persistent().get(address).unwrap_or_else(|| {
        log!(&env, "Vesting: Get vesting schedule: Critical error - No vesting schedule found for the given address");
        panic_with_error!(env, ContractError::VestingNotFoundForAddress);
    })
}

pub fn remove_vesting(env: &Env, address: &Address) {
    env.storage().persistent().remove(&address);
}

pub fn update_vesting(env: &Env, address: &Address, new_curve: Curve) -> Result<(), ContractError> {
    let max_complexity = get_config(&env).max_vesting_complexity;
    env.storage()
        .persistent()
        .update(&address, |current_value: Option<Curve>| {
            let new_curve_schedule = current_value
                .map(|current_value| current_value.combine(&env, &new_curve))
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

            new_curve_schedule
        });

    Ok(())
}

pub fn update_allowances(env: &Env, owner_spender: &(&Address, &Address), allowance: &i128) {
    env.storage().persistent().set(owner_spender, allowance);
}

pub fn get_allowances(env: &Env, owner_spender: &(&Address, &Address)) -> i128 {
    env.storage().persistent().get(owner_spender).unwrap_or_else(|| {
            log!(&env, "Vesting: Get allowance: Critical error - No allowance found for the given address pair");
            panic_with_error!(env, ContractError::AllowanceNotFoundForGivenPair);
        })
}

pub fn save_minter(env: &Env, minter_addr: &Address, curve: &Curve) {
    env.storage().persistent().set(&minter_addr, curve);
}

pub fn get_minter_cap(env: &Env, minter_addr: &Address) -> Curve {
    env.storage()
        .persistent()
        .get(&minter_addr)
        .unwrap_or_else(|| {
            log!(
                &env,
                "Vesting: Get minter cap: Critical error - No minter found "
            );
            panic_with_error!(env, ContractError::MinterNotFound);
        })
}
