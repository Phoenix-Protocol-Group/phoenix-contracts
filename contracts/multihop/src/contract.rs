extern crate std;
use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, Vec};

use crate::lp_contract::Referral;
use crate::storage::{
    get_factory, is_initialized, save_factory, set_initialized, DataKey,
    SimulateReverseSwapResponse, SimulateSwapResponse, Swap,
};
use crate::utils::{verify_reverse_swap, verify_swap};
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

    fn swap(
        env: Env,
        recipient: Address,
        referral: Option<Referral>,
        operations: Vec<Swap>,
        max_belief_price: Option<i64>,
        max_spread_bps: Option<i64>,
        amount: i128,
    );

    fn simulate_swap(env: Env, operations: Vec<Swap>, amount: i128) -> SimulateSwapResponse;

    fn simulate_reverse_swap(
        env: Env,
        operations: Vec<Swap>,
        amount: i128,
    ) -> SimulateReverseSwapResponse;

    fn get_admin(env: Env) -> Address;
}

#[contractimpl]
impl MultihopTrait for Multihop {
    fn initialize(env: Env, admin: Address, factory: Address) {
        if is_initialized(&env) {
            panic!("Multihop: Initialize: initializing contract twice is not allowed");
        }

        set_initialized(&env);

        env.storage()
            .persistent()
            .set(&DataKey::Admin, &admin.clone());

        save_factory(&env, factory);

        env.events()
            .publish(("initialize", "Multihop factory with admin: "), admin);
    }

    fn swap(
        env: Env,
        recipient: Address,
        referral: Option<Referral>,
        operations: Vec<Swap>,
        max_belief_price: Option<i64>,
        max_spread_bps: Option<i64>,
        amount: i128,
    ) {
        if operations.is_empty() {
            panic!("Multihop: Swap: operations is empty!");
        }
        std::println!("{}", "verify operations");
        env.budget().reset_default();
        verify_swap(&operations);
        env.budget().print();

        recipient.require_auth();

        // first offer amount is an input from the user,
        // subsequent are the results of the previous swap
        let mut next_offer_amount: i128 = amount;

        std::println!("{}", "make factory client");
        env.budget().reset_default();
        let factory_client = factory_contract::Client::new(&env, &get_factory(&env));
        env.budget().print();

        operations.iter().for_each(|op| {
            std::println!("{}", "factory query for lp");
            env.budget().reset_default();
            let liquidity_pool_addr: Address = factory_client
                .query_for_pool_by_token_pair(&op.clone().offer_asset, &op.ask_asset.clone());
            env.budget().print();

            std::println!("{}", "lp contract call for swap");
            env.budget().reset_default();
            let lp_client = lp_contract::Client::new(&env, &liquidity_pool_addr);
            if let Some(referral) = referral.clone() {
                next_offer_amount = lp_client.swap(
                    &recipient,
                    &Some(referral),
                    &op.offer_asset,
                    &next_offer_amount,
                    &max_belief_price,
                    &max_spread_bps,
                );
            } else {
                next_offer_amount = lp_client.swap(
                    &recipient,
                    &None,
                    &op.offer_asset,
                    &next_offer_amount,
                    &max_belief_price,
                    &max_spread_bps,
                );
            }
            env.budget().print();
        });
    }

    fn simulate_swap(env: Env, operations: Vec<Swap>, amount: i128) -> SimulateSwapResponse {
        if operations.is_empty() {
            panic!("Multihop: Simulate swap: operations empty");
        }

        verify_swap(&operations);

        let mut next_offer_amount: i128 = amount;

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

            next_offer_amount = simulate_swap.ask_amount;
        });

        simulate_swap_response
    }

    fn simulate_reverse_swap(
        env: Env,
        operations: Vec<Swap>,
        amount: i128,
    ) -> SimulateReverseSwapResponse {
        if operations.is_empty() {
            panic!("Multihop: Simulate reverse swap: operations empty");
        }

        verify_reverse_swap(&operations);

        let mut next_ask_amount: i128 = amount;

        let mut simulate_swap_response = SimulateReverseSwapResponse {
            offer_amount: 0,
            total_commission_amount: 0,
        };

        let factory_client = factory_contract::Client::new(&env, &get_factory(&env));

        operations.iter().for_each(|op| {
            let liquidity_pool_addr: Address = factory_client
                .query_for_pool_by_token_pair(&op.clone().offer_asset, &op.ask_asset.clone());

            let lp_client = lp_contract::Client::new(&env, &liquidity_pool_addr);
            let simulate_reverse_swap =
                lp_client.simulate_reverse_swap(&op.ask_asset, &next_ask_amount);

            simulate_swap_response.total_commission_amount +=
                simulate_reverse_swap.commission_amount;
            simulate_swap_response.offer_amount = simulate_reverse_swap.offer_amount;

            next_ask_amount = simulate_reverse_swap.offer_amount;
        });

        simulate_swap_response
    }

    fn get_admin(env: Env) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::Admin)
            .expect("Multihop: No admin found")
    }
}
