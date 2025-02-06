use phoenix::ttl::{INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL};
use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, vec, Address, BytesN, Env, Vec,
};

use crate::error::ContractError;
use crate::factory_contract::PoolType;
// FIXM: Disable Referral struct
// use crate::lp_contract::Referral;
use crate::storage::{
    get_admin_old, get_factory, is_initialized, save_admin_old, save_factory, set_initialized,
    SimulateReverseSwapResponse, SimulateSwapResponse, Swap, ADMIN, MULTIHOP_KEY,
};
use crate::utils::{verify_reverse_swap, verify_swap};
use crate::{factory_contract, stable_pool, token_contract, xyk_pool};

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Contract to enable chaining of multiple swap transactions together"
);

#[contract]
pub struct Multihop;

#[allow(dead_code)]
pub trait MultihopTrait {
    fn initialize(env: Env, admin: Address, factory: Address);

    #[allow(clippy::too_many_arguments)]
    fn swap(
        env: Env,
        recipient: Address,
        // FIXM: Disable Referral struct
        // referral: Option<Referral>,
        operations: Vec<Swap>,
        max_spread_bps: Option<i64>,
        amount: i128,
        pool_type: PoolType,
        deadline: Option<u64>,
        max_allowed_fee_bps: Option<i64>,
    );

    fn simulate_swap(
        env: Env,
        operations: Vec<Swap>,
        amount: i128,
        pool_type: PoolType,
    ) -> SimulateSwapResponse;

    fn simulate_reverse_swap(
        env: Env,
        operations: Vec<Swap>,
        amount: i128,
        pool_type: PoolType,
    ) -> SimulateReverseSwapResponse;

    fn migrate_admin_key(env: Env) -> Result<(), ContractError>;
}

#[contractimpl]
impl MultihopTrait for Multihop {
    fn initialize(env: Env, admin: Address, factory: Address) {
        if is_initialized(&env) {
            log!(
                &env,
                "Multihop: Initialize: initializing contract twice is not allowed"
            );
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }

        set_initialized(&env);

        save_admin_old(&env, &admin);

        save_factory(&env, factory);

        env.storage().persistent().set(&MULTIHOP_KEY, &true);

        env.events()
            .publish(("initialize", "Multihop factory with admin: "), admin);
    }

    #[allow(clippy::too_many_arguments)]
    fn swap(
        env: Env,
        recipient: Address,
        // FIXM: Disable Referral struct
        // referral: Option<Referral>,
        operations: Vec<Swap>,
        max_spread_bps: Option<i64>,
        amount: i128,
        pool_type: PoolType,
        deadline: Option<u64>,
        max_allowed_fee_bps: Option<i64>,
    ) {
        if operations.is_empty() {
            log!(&env, "Multihop: Swap: operations is empty!");
            panic_with_error!(&env, ContractError::OperationsEmpty);
        }
        verify_swap(&env, &operations);

        recipient.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        // first offer amount is an input from the user,
        // subsequent are the results of the previous swap
        let mut next_offer_amount: i128 = amount;

        let factory_client = factory_contract::Client::new(&env, &get_factory(&env));

        operations.iter().for_each(|op| {
            let liquidity_pool_addr: Address = factory_client
                .query_for_pool_by_token_pair(&op.clone().offer_asset, &op.ask_asset.clone());

            match pool_type {
                PoolType::Xyk => {
                    let lp_client = xyk_pool::Client::new(&env, &liquidity_pool_addr);
                    // FIXM: Disable Referral struct
                    next_offer_amount = lp_client.swap(
                        &recipient,
                        // &referral,
                        &op.offer_asset,
                        &next_offer_amount,
                        &op.ask_asset_min_amount,
                        &max_spread_bps,
                        &deadline,
                        &max_allowed_fee_bps,
                    );
                }
                PoolType::Stable => {
                    let lp_client = stable_pool::Client::new(&env, &liquidity_pool_addr);
                    next_offer_amount = lp_client.swap(
                        &recipient,
                        &op.offer_asset,
                        &next_offer_amount,
                        &op.ask_asset_min_amount,
                        &max_spread_bps,
                        &deadline,
                        &max_allowed_fee_bps,
                    );
                }
            }
        });
    }

    fn simulate_swap(
        env: Env,
        operations: Vec<Swap>,
        amount: i128,
        pool_type: PoolType,
    ) -> SimulateSwapResponse {
        if operations.is_empty() {
            log!(&env, "Multihop: Simulate swap: operations empty");
            panic_with_error!(&env, ContractError::OperationsEmpty);
        }

        verify_swap(&env, &operations);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let mut next_offer_amount: i128 = amount;

        let mut simulate_swap_response = SimulateSwapResponse {
            ask_amount: 0,
            commission_amounts: vec![&env],
            spread_amount: vec![&env],
        };

        let factory_client = factory_contract::Client::new(&env, &get_factory(&env));

        operations.iter().for_each(|op| {
            let pool_addres: Address = factory_client
                .query_for_pool_by_token_pair(&op.clone().offer_asset, &op.ask_asset.clone());

            // due to different pool libraries we cannot use shorter match statement.
            match pool_type {
                PoolType::Xyk => {
                    let lp_client = xyk_pool::Client::new(&env, &pool_addres);
                    let simulated_swap =
                        lp_client.simulate_swap(&op.offer_asset, &next_offer_amount);

                    let token_symbol = token_contract::Client::new(&env, &op.offer_asset).symbol();

                    simulate_swap_response
                        .commission_amounts
                        .push_back((token_symbol, simulated_swap.commission_amount));
                    simulate_swap_response.ask_amount = simulated_swap.ask_amount;
                    simulate_swap_response
                        .spread_amount
                        .push_back(simulated_swap.spread_amount);

                    next_offer_amount = simulated_swap.ask_amount;
                }
                PoolType::Stable => {
                    let lp_client = stable_pool::Client::new(&env, &pool_addres);
                    let simulated_swap =
                        lp_client.simulate_swap(&op.offer_asset, &next_offer_amount);

                    let token_symbol = token_contract::Client::new(&env, &op.offer_asset).symbol();

                    simulate_swap_response
                        .commission_amounts
                        .push_back((token_symbol, simulated_swap.commission_amount));
                    simulate_swap_response.ask_amount = simulated_swap.ask_amount;
                    simulate_swap_response
                        .spread_amount
                        .push_back(simulated_swap.spread_amount);

                    next_offer_amount = simulated_swap.ask_amount;
                }
            }
        });

        simulate_swap_response
    }

    fn simulate_reverse_swap(
        env: Env,
        operations: Vec<Swap>,
        amount: i128,
        pool_type: PoolType,
    ) -> SimulateReverseSwapResponse {
        if operations.is_empty() {
            log!(&env, "Multihop: Simulate reverse swap: operations empty");
            panic_with_error!(&env, ContractError::OperationsEmpty);
        }

        verify_reverse_swap(&env, &operations);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let mut next_ask_amount: i128 = amount;

        let mut simulate_swap_response = SimulateReverseSwapResponse {
            offer_amount: 0,
            commission_amounts: vec![&env],
            spread_amount: vec![&env],
        };

        let factory_client = factory_contract::Client::new(&env, &get_factory(&env));

        operations.iter().for_each(|op| {
            let pool_address: Address = factory_client
                .query_for_pool_by_token_pair(&op.clone().offer_asset, &op.ask_asset.clone());

            // due to different pool libraries we cannot use shorter match statement.
            match pool_type {
                PoolType::Xyk => {
                    let lp_client = xyk_pool::Client::new(&env, &pool_address);
                    let simulated_reverse_swap =
                        lp_client.simulate_reverse_swap(&op.ask_asset, &next_ask_amount);

                    let token_symbol = token_contract::Client::new(&env, &op.ask_asset).symbol();

                    simulate_swap_response
                        .commission_amounts
                        .push_back((token_symbol, simulated_reverse_swap.commission_amount));
                    simulate_swap_response.offer_amount = simulated_reverse_swap.offer_amount;
                    simulate_swap_response
                        .spread_amount
                        .push_back(simulated_reverse_swap.spread_amount);

                    next_ask_amount = simulated_reverse_swap.offer_amount;
                }
                PoolType::Stable => {
                    let lp_client = stable_pool::Client::new(&env, &pool_address);
                    let simulated_reverse_swap =
                        lp_client.simulate_reverse_swap(&op.ask_asset, &next_ask_amount);

                    let token_symbol = token_contract::Client::new(&env, &op.ask_asset).symbol();

                    simulate_swap_response
                        .commission_amounts
                        .push_back((token_symbol, simulated_reverse_swap.commission_amount));
                    simulate_swap_response.offer_amount = simulated_reverse_swap.offer_amount;
                    simulate_swap_response
                        .spread_amount
                        .push_back(simulated_reverse_swap.spread_amount);

                    next_ask_amount = simulated_reverse_swap.offer_amount;
                }
            }
        });

        simulate_swap_response
    }

    fn migrate_admin_key(env: Env) -> Result<(), ContractError> {
        let admin = get_admin_old(&env);
        env.storage().instance().set(&ADMIN, &admin);

        Ok(())
    }
}

#[contractimpl]
impl Multihop {
    #[allow(dead_code)]
    pub fn update(env: Env, new_wasm_hash: BytesN<32>) {
        let admin = get_admin_old(&env);
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    #[allow(dead_code)]
    //TODO: Remove after we've added the key to storage
    pub fn add_new_key_to_storage(env: Env) -> Result<(), ContractError> {
        env.storage().persistent().set(&MULTIHOP_KEY, &true);
        Ok(())
    }
}
