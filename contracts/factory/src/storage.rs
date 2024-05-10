use soroban_sdk::{contracttype, Address, BytesN, ConversionError, Env, TryFromVal, Val, Vec};

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    Config = 1,
    LpVec = 2,
    Initialized = 3,
}

#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PoolType {
    Xyk = 0,
    Stable = 1,
}

#[derive(Clone)]
#[contracttype]
pub struct PairTupleKey {
    pub(crate) token_a: Address,
    pub(crate) token_b: Address,
}

impl TryFromVal<Env, DataKey> for Val {
    type Error = ConversionError;

    fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
        Ok((*v as u32).into())
    }
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub admin: Address,
    pub multihop_address: Address,
    pub lp_wasm_hash: BytesN<32>,
    pub stable_wasm_hash: BytesN<32>,
    pub stake_wasm_hash: BytesN<32>,
    pub token_wasm_hash: BytesN<32>,
    pub whitelisted_accounts: Vec<Address>,
    pub lp_token_decimals: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserPortfolio {
    pub lp_portfolio: Vec<LpPortfolio>,
    pub stake_portfolio: Vec<StakePortfolio>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LpPortfolio {
    pub assets: (Asset, Asset),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StakePortfolio {
    pub staking_contract: Address,
    pub stakes: Vec<Stake>,
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
    /// The address of the Stake contract for the liquidity pool
    pub stake_address: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiquidityPoolInfo {
    pub pool_address: Address,
    pub pool_response: PoolResponse,
    pub total_fee_bps: i64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakedResponse {
    pub stakes: Vec<Stake>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stake {
    /// The amount of staked tokens
    pub stake: i128,
    /// The timestamp when the stake was made
    pub stake_timestamp: u64,
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().persistent().set(&DataKey::Config, &config);
}

pub fn get_config(env: &Env) -> Config {
    env.storage()
        .persistent()
        .get(&DataKey::Config)
        .expect("Config not set")
}

pub fn get_lp_vec(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::LpVec)
        .expect("Factory: get_lp_vec: Liquidity Pool vector not found")
}

pub fn save_lp_vec(env: &Env, lp_info: Vec<Address>) {
    env.storage().persistent().set(&DataKey::LpVec, &lp_info);
}

pub fn save_lp_vec_with_tuple_as_key(
    env: &Env,
    tuple_pool: (&Address, &Address),
    lp_address: &Address,
) {
    env.storage().persistent().set(
        &PairTupleKey {
            token_a: tuple_pool.0.clone(),
            token_b: tuple_pool.1.clone(),
        },
        &lp_address,
    )
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
