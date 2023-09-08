use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env};

use crate::error::ContractError;
use crate::storage::Swap;

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Contract to enable chaining of multiple swap transactions together"
);

#[contract]
pub struct Multihop;

pub trait MultihopTrait {
    fn initialize(
        env: Env,
        admin: Address,
        liquidity_pools: Vec<Address>,
    ) -> Result<(), ContractError>;

    fn swap(env: Env, operations: Vec<Swap>) -> Result<(), ContractError>;
}

#[contractimpl]
impl MultihopTrait for Multihop {
    fn initialize(
        _env: Env,
        _admin: Address,
        _liquidity_pools: Vec<Address>,
    ) -> Result<(), ContractError> {
        unimplemented!();
    }

    fn swap(_env: Env, _operations: Vec<Swap>) -> Result<(), ContractError> {
        unimplemented!();
    }
}
