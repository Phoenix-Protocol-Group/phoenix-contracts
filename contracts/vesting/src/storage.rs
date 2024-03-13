use curve::Curve;
use soroban_sdk::{contracttype, log, panic_with_error, Address, Env, String, Vec};

use crate::error::ContractError;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    /// `admin` who can manage the contract with administrative privileges.
    pub admin: Address,
    /// `whitelist` list of addresses that can interact with the contract.
    pub whitelist: Vec<Address>,
    /// `token_info` information about the token used in the vesting contract.
    pub token_info: VestingTokenInfo,
    /// `balances` vector of tuples representing the balance of each address
    pub balances: Vec<(Address, i128)>,
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
