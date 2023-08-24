use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, Vec};

use crate::error::ContractError;

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "Phoenix Protocol Factory");

#[contract]
pub struct Factory;

pub trait FactoryTrait {
    fn initialize(env: Env, admin: Address) -> Result<(), ContractError>;

    fn create_liquidity_pool(env: Env) -> Result<(), ContractError>;

    fn query_pools(env: Env) -> Result<Vec<Address>, ContractError>;
}

#[contractimpl]
impl FactoryTrait for Factory {
    fn initialize(_env: Env, _admin: Address) -> Result<(), ContractError> {
        unimplemented!();
    }

    fn create_liquidity_pool(_env: Env) -> Result<(), ContractError> {
        unimplemented!();
    }

    fn query_pools(_env: Env) -> Result<Vec<Address>, ContractError> {
        unimplemented!();
    }
}
