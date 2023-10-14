use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, Vec};

use crate::storage::{get_factory, save_admin, save_factory, SimulateSwapResponse, Swap};
use crate::{factory_contract, lp_contract};

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Contract to enable chaining of multiple swap transactions together"
);

#[contract]
pub struct Multihop;

pub trait MultihopTrait {
    fn initialize(env: Env, admin: Address, factory: Address);

    fn swap(env: Env, recipient: Address, operations: Vec<Swap>, amount: i128);

    fn simulate_swap(env: Env, operations: Vec<Swap>, amount: i128) -> SimulateSwapResponse;
}

#[contractimpl]
impl MultihopTrait for Multihop {
    fn initialize(env: Env, admin: Address, factory: Address) {
        save_admin(&env, &admin);

        save_factory(&env, factory);

        env.events()
            .publish(("initialize", "Multihop factory with admin: "), admin);
    }

    fn swap(env: Env, recipient: Address, operations: Vec<Swap>, amount: i128) {
        if operations.is_empty() {
            panic!("Multihop: Swap: Operations empty");
        }

        recipient.require_auth();

        // first offer amount is an input from the user,
        // subsequent are the results of the previous swap
        let mut next_offer_amount: i128 = amount;
        let mut offer_token_addr: Address = operations.get(0).unwrap().offer_asset.clone();

        let factory_client = factory_contract::Client::new(&env, &get_factory(&env));

        operations.iter().for_each(|op| {
            let liquidity_pool_addr: Address = factory_client
                .query_for_pool_by_token_pair(&op.clone().offer_asset, &op.ask_asset.clone());

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
    }

    fn simulate_swap(env: Env, operations: Vec<Swap>, amount: i128) -> SimulateSwapResponse {
        if operations.is_empty() {
            panic!("Multihop: Swap: Operations empty");
        }

        let next_offer_amount: i128 = amount;
        let mut offer_token_addr: Address = operations.get(0).unwrap().offer_asset.clone();

        let mut simulate_swap_response = SimulateSwapResponse {
            ask_amount: 0,
            total_commission_amount: 0,
        };

        let factory_client = factory_contract::Client::new(&env, &get_factory(&env));

        operations.iter().for_each(|op| {
            let liquidity_pool_addr: Address = factory_client
                .query_for_pool_by_token_pair(&op.clone().offer_asset, &op.ask_asset.clone());

            let lp_client = lp_contract::Client::new(&env, &liquidity_pool_addr);
            let simulate_swap = lp_client.simulate_swap(&op.offer_asset, &next_offer_amount);

            simulate_swap_response.total_commission_amount += simulate_swap.commission_amount;
            simulate_swap_response.ask_amount = simulate_swap.ask_amount;

            offer_token_addr = op.ask_asset.clone();
        });

        simulate_swap_response
    }
}
