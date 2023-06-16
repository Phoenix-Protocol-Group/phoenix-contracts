use soroban_sdk::{
    contracttype, xdr::ToXdr, Address, Bytes, BytesN, ConversionError, Env, RawVal, Symbol,
    TryFromVal,
};

use crate::token_contract;

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    TotalShares = 0,
    ReserveA = 1,
    ReserveB = 2,
}

impl TryFromVal<Env, DataKey> for RawVal {
    type Error = ConversionError;

    fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
        Ok((*v as u32).into())
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Config {
    pub token_a: Address,
    pub token_b: Address,
    pub share_token: Address,
}
const CONFIG: Symbol = Symbol::short("CONFIG");

pub fn get_config(env: &Env) -> Config {
    env.storage().get(&CONFIG).unwrap().unwrap()
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().set(&CONFIG, &config);
}

pub mod utils {
    use super::*;

    pub fn deploy_token_contract(
        e: &Env,
        token_wasm_hash: &BytesN<32>,
        token_a: &Address,
        token_b: &Address,
    ) -> Address {
        let mut salt = Bytes::new(e);
        salt.append(&token_a.to_xdr(e));
        salt.append(&token_b.to_xdr(e));
        let salt = e.crypto().sha256(&salt);
        e.deployer()
            .with_current_contract(&salt)
            .deploy(token_wasm_hash)
    }

    pub fn save_total_shares(e: &Env, amount: i128) {
        e.storage().set(&DataKey::TotalShares, &amount)
    }

    pub fn save_pool_balance_a(e: &Env, amount: i128) {
        e.storage().set(&DataKey::ReserveA, &amount)
    }

    pub fn save_pool_balance_b(e: &Env, amount: i128) {
        e.storage().set(&DataKey::ReserveB, &amount)
    }

    pub fn mint_shares(e: &Env, share_token: Address, to: Address, amount: i128) {
        let total = get_total_shares(e);

        token_contract::Client::new(e, &share_token).mint(&to, &amount);

        save_total_shares(e, total + amount);
    }

    // queries
    pub fn get_total_shares(e: &Env) -> i128 {
        e.storage().get_unchecked(&DataKey::TotalShares).unwrap()
    }
    pub fn get_pool_balance_a(e: &Env) -> i128 {
        e.storage().get_unchecked(&DataKey::ReserveA).unwrap()
    }

    pub fn get_pool_balance_b(e: &Env) -> i128 {
        e.storage().get_unchecked(&DataKey::ReserveB).unwrap()
    }

    pub fn get_balance(e: &Env, contract: Address) -> i128 {
        token_contract::Client::new(e, &contract).balance(&e.current_contract_address())
    }

    pub fn get_deposit_amounts(
        desired_a: i128,
        min_a: i128,
        desired_b: i128,
        min_b: i128,
        pool_balance_a: i128,
        pool_balance_b: i128,
    ) -> (i128, i128) {
        if pool_balance_a == 0 && pool_balance_b == 0 {
            return (desired_a, desired_b);
        }

        let amount_b = desired_a * pool_balance_b / pool_balance_a;
        if amount_b <= desired_b {
            if amount_b < min_b {
                panic!("amount_b less than min")
            }
            (desired_a, amount_b)
        } else {
            let amount_a = desired_b * pool_balance_a / pool_balance_b;
            if amount_a > desired_a || desired_a < min_a {
                panic!("amount_a invalid")
            }
            (amount_a, desired_b)
        }
    }
}
