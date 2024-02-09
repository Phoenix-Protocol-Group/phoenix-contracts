use soroban_sdk::{contracttype, Address, Env, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Swap {
    pub ask_asset: Address,
    pub offer_asset: Address,
    pub max_belief_price: Option<i64>,
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
    pub total_commission_amount: i128,
    pub spread_amount: Vec<i128>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimulateReverseSwapResponse {
    pub offer_amount: i128,
    pub total_commission_amount: i128,
    pub spread_amount: Vec<i128>,
}
