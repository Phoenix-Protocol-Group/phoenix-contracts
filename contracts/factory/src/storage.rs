use crate::error::ContractError;
use soroban_sdk::{
    contracttype, symbol_short, Address, BytesN, ConversionError, Env, Symbol, TryFromVal, Val,
};

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    Admin = 1,
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
    pub quiet_please: i32, // to satisfy the compiler
}

const CONFIG: Symbol = symbol_short!("CONFIG");

pub fn get_config(env: &Env) -> Result<Config, ContractError> {
    env.storage()
        .instance()
        .get(&CONFIG)
        .ok_or(ContractError::ConfigNotSet)
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().instance().set(&CONFIG, &config);
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
