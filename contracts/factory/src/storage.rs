use soroban_sdk::{contracttype, Address, ConversionError, Env, TryFromVal, Val, Vec};

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

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiquidityPoolInfo {
    pub pool_address: Address,
    pub pool_response: PoolResponse,
    pub total_fee_bps: i64,
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
        .instance()
        .get(&DataKey::LpVec)
        .expect("Factory: get_lp_vec: Liquidity Pool vector not found")
}

pub fn save_lp_vec(env: &Env, lp_info: Vec<Address>) {
    env.storage().instance().set(&DataKey::LpVec, &lp_info);
}

pub fn save_lp_vec_with_tuple_as_key(
    env: &Env,
    tuple_pool: (&Address, &Address),
    lp_address: &Address,
) {
    env.storage().instance().set(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "HostError: Error(Context, MissingValue)")]
    fn test_get_admin_should_panic_when_no_admin_saved() {
        let env = Env::default();

        get_config(&env);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Context, MissingValue)")]
    fn test_get_lp_vec_should_panic_when_no_vec_saved() {
        let env = Env::default();

        get_lp_vec(&env);
    }
}
