use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, Vec};

use crate::error::ContractError;
use crate::storage::{get_liquidity_pool, save_admin, save_liquidity_pool, Pair, Swap};

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
        liquidity_pools: Vec<(Pair, Address)>,
    ) -> Result<(), ContractError>;

    fn swap(env: Env, operations: Vec<Swap>) -> Result<(), ContractError>;
}

#[contractimpl]
impl MultihopTrait for Multihop {
    fn initialize(
        env: Env,
        admin: Address,
        liquidity_pools: Vec<(Pair, Address)>,
    ) -> Result<(), ContractError> {
        save_admin(&env, &admin);

        for lp in liquidity_pools.iter() {
            let pair = lp.0;
            let lp_address = lp.1;
            save_liquidity_pool(&env, pair, lp_address);
        }

        env.events()
            .publish(("initialize", "Multihop factory"), admin);

        Ok(())
    }

    fn swap(env: Env, operations: Vec<Swap>) -> Result<(), ContractError> {
        for op in operations.iter() {
            let _lp_address = get_liquidity_pool(
                &env,
                Pair {
                    token_a: op.ask_asset,
                    token_b: op.offer_asset,
                },
            )?;
        }

        unimplemented!();
    }
}
