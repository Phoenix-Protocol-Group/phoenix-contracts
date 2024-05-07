use soroban_sdk::{contracttype, log, panic_with_error, Address, Env, String, Vec};

use crate::error::ContractError;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Swap {
    pub ask_asset: Address,
    pub offer_asset: Address,
    pub ask_asset_min_amount: Option<i128>,
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
    FactoryKey,
    Admin,
    Initialized,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asset {
    /// Address of the asset
    pub address: Address,
    /// The total amount of those tokens in the pool
    pub amount: i128,
}

/// This struct is used to return a query result with the total amount of LP tokens and assets in a specific pool.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolResponse {
    /// The asset A in the pool together with asset amounts
    pub asset_a: Asset,
    /// The asset B in the pool together with asset amounts
    pub asset_b: Asset,
    /// The total amount of LP tokens currently issued
    pub asset_lp_share: Asset,
}

pub fn save_factory(env: &Env, factory: Address) {
    env.storage().instance().set(&DataKey::FactoryKey, &factory);
}

pub fn get_factory(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::FactoryKey).unwrap()
}

pub fn save_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| {
            log!(env, "Admin not set");
            panic_with_error!(&env, ContractError::AdminNotSet)
        })
}
pub fn is_initialized(e: &Env) -> bool {
    e.storage()
        .persistent()
        .get(&DataKey::Initialized)
        .unwrap_or(false)
}

pub fn set_initialized(e: &Env) {
    e.storage().persistent().set(&DataKey::Initialized, &true);
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimulateSwapResponse {
    pub ask_amount: i128,
    /// tuple of ask_asset denom and commission amount for the swap
    pub commission_amounts: Vec<(String, i128)>,
    pub spread_amount: Vec<i128>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimulateReverseSwapResponse {
    pub offer_amount: i128,
    /// tuple of offer_asset denom and commission amount for the swap
    pub commission_amounts: Vec<(String, i128)>,
    pub spread_amount: Vec<i128>,
}
