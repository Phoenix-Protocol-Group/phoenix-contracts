use crate::error::ContractError;
use soroban_sdk::{contracttype, symbol_short, Address, ConversionError, Env, Symbol, TryFromVal, Val, Vec};

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
    pub lp_tokens: Vec<Address>,
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

pub mod utils {
    use super::*;
    use soroban_sdk::BytesN;
    pub fn deploy_lp_contract(_env: &Env, _lp_wasm_hash: BytesN<32>) {
        unimplemented!();
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
}

#[cfg(test)]
mod tests {
    use soroban_sdk::testutils::Address;
    use super::*;

    #[test]
    fn test_should_save_admin() {
        let env = Env::default();
        let admin = <soroban_sdk::Address as Address>::random(&env);

        utils::save_admin(&env, admin.clone());
        assert_eq!(utils::get_admin(&env).unwrap(), admin);
    }

    #[test]
    #[should_panic]
    fn test_get_admin_should_panic_when_no_admin_saved() {
        let env = Env::default();

        utils::get_admin(&env).unwrap();
    }
}