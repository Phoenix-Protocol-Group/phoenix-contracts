use phoenix::ttl::{
    INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT,
    PERSISTENT_LIFETIME_THRESHOLD,
};
use soroban_sdk::{
    contracttype, log, panic_with_error, symbol_short, Address, BytesN, ConversionError, Env,
    Symbol, TryFromVal, Val, Vec,
};

use crate::error::ContractError;

pub const ADMIN: Symbol = symbol_short!("ADMIN");

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    Config = 1,
    LpVec = 2,
    Initialized = 3,
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
    pub stake_wasm_hash: BytesN<32>,
    pub token_wasm_hash: BytesN<32>,
    pub whitelisted_accounts: Vec<Address>,
    pub lp_token_decimals: u32,
}

const STABLE_WASM_HASH: Symbol = symbol_short!("stabwasm");

pub fn save_stable_wasm_hash(env: &Env, hash: BytesN<32>) {
    env.storage().persistent().set(&STABLE_WASM_HASH, &hash);
    env.storage().persistent().extend_ttl(
        &STABLE_WASM_HASH,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn get_stable_wasm_hash(env: &Env) -> BytesN<32> {
    let hash = env
        .storage()
        .persistent()
        .get(&STABLE_WASM_HASH)
        .expect("Stable wasm hash not set");

    env.storage().persistent().extend_ttl(
        &STABLE_WASM_HASH,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );

    hash
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
    pub total_stake: i128,
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
    env.storage().persistent().extend_ttl(
        &DataKey::Config,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn get_config(env: &Env) -> Config {
    let config = env
        .storage()
        .persistent()
        .get(&DataKey::Config)
        .expect("Config not set");

    env.storage().persistent().extend_ttl(
        &DataKey::Config,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );

    config
}

pub fn _save_admin(env: &Env, admin_addr: Address) {
    env.storage().instance().set(&ADMIN, &admin_addr);

    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn _get_admin(env: &Env) -> Address {
    let admin_addr = env.storage().instance().get(&ADMIN).unwrap_or_else(|| {
        log!(env, "Factory: Admin not set");
        panic_with_error!(&env, ContractError::AdminNotSet)
    });

    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

    admin_addr
}

pub fn get_lp_vec(env: &Env) -> Vec<Address> {
    let lp_vec = env
        .storage()
        .persistent()
        .get(&DataKey::LpVec)
        .expect("Factory: get_lp_vec: Liquidity Pool vector not found");

    env.storage().persistent().extend_ttl(
        &DataKey::LpVec,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );

    lp_vec
}

pub fn save_lp_vec(env: &Env, lp_info: Vec<Address>) {
    env.storage().persistent().set(&DataKey::LpVec, &lp_info);
    env.storage().persistent().extend_ttl(
        &DataKey::LpVec,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
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
    );

    env.storage().persistent().extend_ttl(
        &PairTupleKey {
            token_a: tuple_pool.0.clone(),
            token_b: tuple_pool.1.clone(),
        },
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn is_initialized(e: &Env) -> bool {
    e.storage()
        .persistent()
        .get(&DataKey::Initialized)
        .unwrap_or(false)
}

pub fn set_initialized(e: &Env) {
    e.storage().persistent().set(&DataKey::Initialized, &true);

    e.storage().persistent().extend_ttl(
        &DataKey::Initialized,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}
