use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, IntoVal, Symbol, Val, Vec};

use crate::error::ContractError;
use crate::storage::{get_factory, save_admin, save_factory, Swap};
use crate::{factory_contract, lp_contract, token_contract};

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

        let mut offer_amount: i128 = amount;
        let mut offer_token_addr: Address = operations.get(0).unwrap().offer_asset.clone();
        let mut offer_token_client = token_contract::Client::new(&env, &offer_token_addr);

        // first transfer token to multihop contract
        offer_token_client.transfer(&recipient, &env.current_contract_address(), &offer_amount);

        let factory_client =
            factory_contract::Client::new(&env, &get_factory(&env).expect("factory not found"));

        operations.iter().for_each(|op| {
            let liquidity_pool_addr: Address = factory_client
                .query_for_pool_by_pair_tuple(&(op.offer_asset, op.ask_asset.clone()));

            let lp_client = lp_contract::Client::new(&env, &liquidity_pool_addr);
            lp_client.swap(
                &env.current_contract_address(),
                &true,
                &offer_amount,
                &None::<i64>,
                &Some(5000i64),
            );

            offer_token_client = token_contract::Client::new(&env, &op.ask_asset);
            offer_amount = offer_token_client.balance(&env.current_contract_address());
            offer_token_addr = op.ask_asset.clone();
        });

        // in each loop iteration, last asked token becomes an offer; after loop we can rename it
        let asked_amount = offer_amount;
        let asked_token_addr = offer_token_addr;

        let token_func_name = &Symbol::new(&env, "transfer");
        let token_call_args: Vec<Val> =
            (env.current_contract_address(), recipient, asked_amount).into_val(&env);
        env.invoke_contract::<Val>(&asked_token_addr, token_func_name, token_call_args);

        Ok(())
    }
}
