use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, Vec};

use crate::error::ContractError;
use crate::storage::{get_factory, save_admin, save_factory, Swap};
use crate::{factory_contract, lp_contract};

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Contract to enable chaining of multiple swap transactions together"
);

#[contract]
pub struct Multihop;

pub trait MultihopTrait {
    fn initialize(env: Env, admin: Address, factory: Address) -> Result<(), ContractError>;

    fn swap(
        env: Env,
        recipient: Address,
        operations: Vec<Swap>,
        amount: i128,
    ) -> Result<(), ContractError>;
}

#[contractimpl]
impl MultihopTrait for Multihop {
    fn initialize(env: Env, admin: Address, factory: Address) -> Result<(), ContractError> {
        save_admin(&env, &admin);

        save_factory(&env, factory);

        env.events()
            .publish(("initialize", "Multihop factory with admin: "), admin);

        Ok(())
    }

    fn swap(
        env: Env,
        recipient: Address,
        operations: Vec<Swap>,
        amount: i128,
    ) -> Result<(), ContractError> {
        if operations.is_empty() {
            return Err(ContractError::OperationsEmpty);
        }

        recipient.require_auth();

        // first offer amount is an input from the user,
        // subsequent are the results of the previous swap
        let mut next_offer_amount: i128 = amount;
        let mut offer_token_addr: Address = operations.get(0).unwrap().offer_asset.clone();

        let factory_client =
            factory_contract::Client::new(&env, &get_factory(&env).expect("factory not found"));

        operations.iter().for_each(|op| {
            let liquidity_pool_addr: Address = factory_client
                .query_for_pool_by_pool_tuple(&op.clone().offer_asset, &op.ask_asset.clone());

            let lp_client = lp_contract::Client::new(&env, &liquidity_pool_addr);
            next_offer_amount = lp_client.swap(
                &recipient,
                &op.offer_asset,
                &next_offer_amount,
                &None::<i64>,
                &Some(5000i64),
            );

            offer_token_addr = op.ask_asset.clone();
        });

        Ok(())
    }
}
