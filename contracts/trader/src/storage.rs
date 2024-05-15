use soroban_sdk::{
    contracttype, log, panic_with_error, Address, ConversionError, Env, String, TryFromVal, Val,
};

use crate::error::ContractError;

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    Admin,
    ContractId,
    Pair,
    Token,
    MaxSpread,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asset {
    /// Denom
    pub symbol: String,
    /// The total amount of those tokens in the pool
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BalanceInfo {
    pub output_token: Asset,
    pub token_a: Asset,
    pub token_b: Asset,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputTokenInfo {
    pub address: Address,
    pub name: String,
    pub symbol: String,
    pub decimal: u32,
}

impl TryFromVal<Env, DataKey> for Val {
    type Error = ConversionError;

    fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
        Ok((*v as u32).into())
    }
}

pub fn save_admin(env: &Env, address: &Address) {
    env.storage().persistent().set(&DataKey::Admin, address)
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .persistent()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| {
            log!(&env, "Admin not set");
            panic_with_error!(&env, ContractError::AdminNotFound)
        })
}

pub fn save_name(env: &Env, contract_id: &String) {
    env.storage()
        .persistent()
        .set(&DataKey::ContractId, contract_id)
}

pub fn get_name(env: &Env) -> String {
    env.storage()
        .persistent()
        .get(&DataKey::ContractId)
        .unwrap_or_else(|| {
            log!(&env, "Contract ID not set");
            panic_with_error!(&env, ContractError::ContractIdNotFound)
        })
}

pub fn save_pair(env: &Env, pair: &(Address, Address)) {
    env.storage().persistent().set(&DataKey::Pair, pair)
}

pub fn get_pair(env: &Env) -> (Address, Address) {
    env.storage()
        .persistent()
        .get(&DataKey::Pair)
        .unwrap_or_else(|| {
            log!(&env, "Pair not set");
            panic_with_error!(env, ContractError::PairNotFound)
        })
}

pub fn save_output_token(env: &Env, token: &Address) {
    env.storage().persistent().set(&DataKey::Token, token)
}

pub fn get_output_token(env: &Env) -> Address {
    env.storage()
        .persistent()
        .get(&DataKey::Token)
        .unwrap_or_else(|| {
            log!(&env, "Token not set");
            panic_with_error!(env, ContractError::OutputTokenNotFound)
        })
}
