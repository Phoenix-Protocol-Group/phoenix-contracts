use curve::Curve;
use soroban_sdk::{
    contracttype, log, panic_with_error, symbol_short, Address, Env, String, Symbol, Vec,
};

use crate::error::ContractError;

const CONFIG_KEY: Symbol = symbol_short!("config");
const ADMIN: Symbol = symbol_short!("admin");

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
    pub max_vesting_complexity: u64,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingTokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingInitialBalance {
    pub address: Address,
    pub amount: i128,
    /// this has to be an enum
    pub curve: Curve,
}

pub struct BalanceInfo {
    pub address: Address,
    pub balance: i128,
    pub curve: Option<Curve>,
}

pub fn save_config(env: &Env, config: &Config) {
    env.storage().persistent().set(&CONFIG_KEY, config);
}

pub fn get_config(env: &Env) -> Config {
    env.storage()
        .persistent()
        .get(&CONFIG_KEY)
        .unwrap_or_else(|| {
            log!(
                &env,
                "Vesting: Get config: Critical error - No config found"
            );
            panic_with_error!(env, ContractError::NoConfigFound);
        })
}

pub fn save_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&ADMIN, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage().persistent().get(&ADMIN).unwrap_or_else(|| {
        log!(&env, "Vesting: Get admin: Critical error - No admin found");
        panic_with_error!(env, ContractError::NoAdminFound);
    })
}

pub fn save_vesting_schedule(env: &Env, address: &Address, vesting: &Curve) {
    env.storage().persistent().set(address, vesting);
}

pub fn get_vesting_schedule(env: &Env, address: &Address) -> Curve {
    env.storage().persistent().get(address).unwrap_or_else(|| {
        log!(&env, "Vesting: Get vesting schedule: Critical error - No vesting schedule found for the given address");
        panic_with_error!(env, ContractError::VestingScheduleNotFoundForAddress);
    })
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

pub fn save_balance(env: &Env, address: &Address, balance: &i128) {
    env.storage().persistent().set(address, balance);
}

pub fn get_balance(env: &Env, address: &Address) -> i128 {
    env.storage().persistent().get(address).unwrap_or_else(|| {
        log!(
            &env,
            "Vesting: Get balance: Critical error - No balance found for the given address"
        );
        panic_with_error!(env, ContractError::NoBalanceFoundForAddress);
    })
}

pub fn save_minter(env: &Env, minter: &Address, curve: &Option<Curve>) {
    env.storage().persistent().set(minter, curve);
}

pub fn get_minter(env: &Env, minter: &Address) -> Option<Curve> {
    env.storage().persistent().get(minter).unwrap_or_else(|| {
        log!(
            &env,
            "Vesting: Get minter: Critical error - No minter found for the given address"
        );
        panic_with_error!(env, ContractError::MinterNotFoundForAddress);
    })
}
