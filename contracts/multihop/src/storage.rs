use crate::error::ContractError;
use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Swap {
    pub ask_asset: Address,
    pub offer_asset: Address,
    pub amount: i128,
}

#[derive(Clone)]
#[contracttype]
pub struct Pair {
    pub token_a: Address,
    pub token_b: Address,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    PairKey(Pair),
    Admin,
}

pub fn save_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, &admin);
}

pub fn _get_admin(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(ContractError::AdminNotFound)
}

pub fn save_liquidity_pool(env: &Env, pair: Pair, liquidity_pool: Address) {
    env.storage()
        .instance()
        .set(&DataKey::PairKey(pair), &liquidity_pool);
}

pub fn get_liquidity_pool(env: &Env, pair: Pair) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::PairKey(pair))
        .ok_or(ContractError::LiquidityPoolNotFound)
}
