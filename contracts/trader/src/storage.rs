use phoenix::ttl::{
    INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT,
    PERSISTENT_LIFETIME_THRESHOLD,
};
use soroban_sdk::{
    contracttype, log, panic_with_error, symbol_short, Address, ConversionError, Env, String,
    Symbol, TryFromVal, Val,
};

use crate::error::ContractError;

pub const ADMIN: Symbol = symbol_short!("ADMIN");

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    Admin,
    ContractId,
    Pair,
    Token,
    MaxSpread,
    IsInitialized,
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

pub fn save_admin_old(env: &Env, address: &Address) {
    env.storage().persistent().set(&DataKey::Admin, address);
    env.storage().persistent().extend_ttl(
        &DataKey::Admin,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn _save_admin(env: &Env, address: &Address) {
    env.storage().instance().set(&ADMIN, address);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn get_admin_old(env: &Env) -> Address {
    let admin = env
        .storage()
        .persistent()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| {
            log!(&env, "Admin not set");
            panic_with_error!(&env, ContractError::AdminNotFound)
        });
    env.storage().persistent().extend_ttl(
        &DataKey::Admin,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );

    admin
}

pub fn _get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

    env.storage().instance().get(&ADMIN).unwrap_or_else(|| {
        log!(env, "Trader: Admin not set");
        panic_with_error!(&env, ContractError::AdminNotSet)
    })
}

pub fn save_name(env: &Env, contract_id: &String) {
    env.storage()
        .persistent()
        .set(&DataKey::ContractId, contract_id);
    env.storage().persistent().extend_ttl(
        &DataKey::ContractId,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn get_name(env: &Env) -> String {
    let name = env
        .storage()
        .persistent()
        .get(&DataKey::ContractId)
        .unwrap_or_else(|| {
            log!(&env, "Contract ID not set");
            panic_with_error!(&env, ContractError::ContractIdNotFound)
        });
    env.storage().persistent().extend_ttl(
        &DataKey::ContractId,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );

    name
}

pub fn save_pair(env: &Env, pair: &(Address, Address)) {
    env.storage().persistent().set(&DataKey::Pair, pair);
    env.storage().persistent().extend_ttl(
        &DataKey::Pair,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn get_pair(env: &Env) -> (Address, Address) {
    let pair = env
        .storage()
        .persistent()
        .get(&DataKey::Pair)
        .unwrap_or_else(|| {
            log!(&env, "Pair not set");
            panic_with_error!(env, ContractError::PairNotFound)
        });
    env.storage().persistent().extend_ttl(
        &DataKey::Pair,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );

    pair
}

pub fn save_output_token(env: &Env, token: &Address) {
    env.storage().persistent().set(&DataKey::Token, token);
    env.storage().persistent().extend_ttl(
        &DataKey::Token,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn get_output_token(env: &Env) -> Address {
    let token_addr = env
        .storage()
        .persistent()
        .get(&DataKey::Token)
        .unwrap_or_else(|| {
            log!(&env, "Token not set");
            panic_with_error!(env, ContractError::OutputTokenNotFound)
        });
    env.storage().persistent().extend_ttl(
        &DataKey::Token,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );

    token_addr
}
