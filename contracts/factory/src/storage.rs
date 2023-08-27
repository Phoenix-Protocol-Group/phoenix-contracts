use crate::error::ContractError;
use soroban_sdk::{contracttype, Address, ConversionError, Env, TryFromVal, Val, Vec};

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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub liquidity_pools: Vec<Address>,
}

#[allow(dead_code)]
pub fn get_config(env: &Env) -> Result<Config, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(ContractError::ConfigNotSet)
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().instance().set(&DataKey::Config, &config);
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

pub fn save_lp_vec(env: &Env, lp_vec: Vec<Address>) {
    env.storage().instance().set(&DataKey::LpVec, &lp_vec);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_get_admin_should_panic_when_no_admin_saved() {
        let env = Env::default();

        get_admin(&env).unwrap();
    }
}
