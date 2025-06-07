use phoenix::{
    ttl::{INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL},
    utils::AdminChange,
};
use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, vec, Address, BytesN, Env, String,
    Vec,
};

use crate::error::ContractError;
use crate::factory_contract::PoolType;
// FIXM: Disable Referral struct
// use crate::lp_contract::Referral;
use crate::storage::{
    get_admin_old, get_factory, save_admin_old, save_factory, SimulateReverseSwapResponse,
    SimulateSwapResponse, Swap, ADMIN, MULTIHOP_KEY, PENDING_ADMIN,
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

    fn propose_admin(
        env: Env,
        new_admin: Address,
        time_limit: Option<u64>,
    ) -> Result<Address, ContractError>;

    fn revoke_admin_change(env: Env) -> Result<(), ContractError>;

    fn accept_admin(env: Env) -> Result<Address, ContractError>;

    fn query_admin(env: Env) -> Result<Address, ContractError>;
}

#[contractimpl]
impl MultihopTrait for Multihop {
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

    fn propose_admin(
        env: Env,
        new_admin: Address,
        time_limit: Option<u64>,
    ) -> Result<Address, ContractError> {
        let current_admin = get_admin_old(&env);
        current_admin.require_auth();

        if current_admin == new_admin {
            log!(&env, "Trying to set new admin as new");
            panic_with_error!(&env, ContractError::SameAdmin);
        }

        env.storage().instance().set(
            &PENDING_ADMIN,
            &AdminChange {
                new_admin: new_admin.clone(),
                time_limit,
            },
        );

        env.events().publish(
            ("Multihop: ", "Admin replacement requested by old admin: "),
            &current_admin,
        );
        env.events()
            .publish(("Multihop: ", "Replace with new admin: "), &new_admin);

        Ok(new_admin)
    }

    fn revoke_admin_change(env: Env) -> Result<(), ContractError> {
        let current_admin = get_admin_old(&env);
        current_admin.require_auth();

        if !env.storage().instance().has(&PENDING_ADMIN) {
            log!(&env, "No admin change in place");
            panic_with_error!(&env, ContractError::NoAdminChangeInPlace);
        }

        env.storage().instance().remove(&PENDING_ADMIN);

        env.events()
            .publish(("Multihop: ", "Undo admin change: "), ());

        Ok(())
    }

    fn accept_admin(env: Env) -> Result<Address, ContractError> {
        let admin_change_info: AdminChange = env
            .storage()
            .instance()
            .get(&PENDING_ADMIN)
            .unwrap_or_else(|| {
                log!(&env, "No admin change request is in place");
                panic_with_error!(&env, ContractError::NoAdminChangeInPlace);
            });

        let pending_admin = admin_change_info.new_admin;
        pending_admin.require_auth();

        if let Some(time_limit) = admin_change_info.time_limit {
            if env.ledger().timestamp() > time_limit {
                log!(&env, "Admin change expired");
                panic_with_error!(&env, ContractError::AdminChangeExpired);
            }
        }

        env.storage().instance().remove(&PENDING_ADMIN);

        save_admin_old(&env, &pending_admin);

        env.events()
            .publish(("Multihop: ", "Accepted new admin: "), &pending_admin);

        Ok(pending_admin)
    }

    fn query_admin(env: Env) -> Result<Address, ContractError> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let admin = get_admin_old(&env);

        Ok(admin)
    }
}

#[contractimpl]
impl Multihop {
    pub fn __constructor(env: Env, admin: Address, factory: Address) {
        save_admin_old(&env, &admin);

        save_factory(&env, factory);

        env.storage().persistent().set(&MULTIHOP_KEY, &true);

        env.events()
            .publish(("initialize", "Multihop factory with admin: "), admin);
    }

    #[allow(dead_code)]
    pub fn update(env: Env, new_wasm_hash: BytesN<32>) {
        let admin = get_admin_old(&env);
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    #[allow(dead_code)]
    pub fn query_version(env: Env) -> String {
        String::from_str(&env, env!("CARGO_PKG_VERSION"))
    }

    #[allow(dead_code)]
    #[cfg(not(tarpaulin_include))]
    //TODO: Remove after we've added the key to storage
    pub fn add_new_key_to_storage(env: Env) -> Result<(), ContractError> {
        env.storage().persistent().set(&MULTIHOP_KEY, &true);
        Ok(())
    }
}
