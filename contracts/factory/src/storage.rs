use crate::error::ContractError;
use soroban_sdk::{contracttype, Address, ConversionError, Env, Symbol, TryFromVal, Val, Vec};

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    Admin = 1,
    Config = 2,
    LpVec = 3,
}

impl TryFromVal<Env, DataKey> for Val {
    type Error = ConversionError;

    fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
        Ok((*v as u32).into())
    }
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
    pub pool_response: PoolResponse,
    pub total_fee_bps: i64,
}

pub fn save_admin(env: &Env, address: Address) {
    env.storage().instance().set(&DataKey::Admin, &address);
}

pub fn get_admin(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(ContractError::FailedToGetAdminAddrFromStorage)
}

pub fn get_lp_vec(env: &Env) -> Result<Vec<Address>, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::LpVec)
        .ok_or(ContractError::LiquidityPoolVectorNotFound)
}

pub fn query_pool_details(
    env: Env,
    pool_address: Address,
) -> Result<LiquidityPoolInfo, ContractError> {
    let pool_response: PoolResponse = query_pool_info(&env, &pool_address);
    let total_fee_bps = query_pool_total_fee_bps(&env, &pool_address);
    let lp_info = LiquidityPoolInfo {
        pool_response,
        total_fee_bps,
    };

    Ok(lp_info)
}

pub fn query_all_pool_details(env: Env) -> Result<Vec<LiquidityPoolInfo>, ContractError> {
    let all_lp_vec_addresses = get_lp_vec(&env)?;
    let mut result = Vec::new(&env);
    for address in all_lp_vec_addresses {
        let pool_response: PoolResponse = query_pool_info(&env, &address);
        let total_fee_bps = query_pool_total_fee_bps(&env, &address);

        let lp_info = LiquidityPoolInfo {
            pool_response,
            total_fee_bps,
        };

        result.push_back(lp_info);
    }

    Ok(result)
}

pub fn save_lp_vec(env: &Env, lp_info: Vec<Address>) {
    env.storage().instance().set(&DataKey::LpVec, &lp_info);
}

fn query_pool_info(env: &Env, address: &Address) -> PoolResponse {
    env.invoke_contract(address, &Symbol::new(env, "query_pool_info"), Vec::new(env))
}

fn query_pool_total_fee_bps(env: &Env, address: &Address) -> i64 {
    env.invoke_contract(
        address,
        &Symbol::new(env, "get_total_fee_bps"),
        Vec::new(env),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "HostError: Error(Context, MissingValue)")]
    fn test_get_admin_should_panic_when_no_admin_saved() {
        let env = Env::default();

        get_admin(&env).unwrap();
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Context, MissingValue)")]
    fn test_get_lp_vec_should_panic_when_no_vec_saved() {
        let env = Env::default();

        get_lp_vec(&env).unwrap();
    }
}
