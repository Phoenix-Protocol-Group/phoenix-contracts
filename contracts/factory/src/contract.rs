use crate::storage::{
    get_config, is_initialized, save_config, set_initialized, Config, DataKey, LiquidityPoolInfo,
    PairTupleKey,
};
use crate::utils::deploy_multihop_contract;
use crate::{
    storage::{get_lp_vec, save_lp_vec, save_lp_vec_with_tuple_as_key},
    utils::deploy_lp_contract,
};
use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};
use phoenix::validate_bps;
use soroban_sdk::{
    contract, contractimpl, contractmeta, log, Address, BytesN, Env, IntoVal, String, Symbol, Val,
    Vec,
};

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "Phoenix Protocol Factory");

#[contract]
pub struct Factory;

pub trait FactoryTrait {
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        admin: Address,
        multihop_wasm_hash: BytesN<32>,
        lp_wasm_hash: BytesN<32>,
        stake_wasm_hash: BytesN<32>,
        token_wasm_hash: BytesN<32>,
        whitelisted_accounts: Vec<Address>,
        lp_token_decimals: u32,
    );

    fn create_liquidity_pool(
        env: Env,
        caller: Address,
        lp_init_info: LiquidityPoolInitInfo,
        pool_name: String,
        pool_symbol: String,
    ) -> Address;

    fn update_whitelisted_accounts(
        env: Env,
        sender: Address,
        to_add: Vec<Address>,
        to_remove: Vec<Address>,
    );

    fn query_pools(env: Env) -> Vec<Address>;

    fn query_pool_details(env: Env, pool_address: Address) -> LiquidityPoolInfo;

    fn query_all_pools_details(env: Env) -> Vec<LiquidityPoolInfo>;

    fn query_for_pool_by_token_pair(env: Env, token_a: Address, token_b: Address) -> Address;

    fn get_admin(env: Env) -> Address;

    fn get_config(env: Env) -> Config;
}

#[contractimpl]
impl FactoryTrait for Factory {
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        admin: Address,
        multihop_wasm_hash: BytesN<32>,
        lp_wasm_hash: BytesN<32>,
        stake_wasm_hash: BytesN<32>,
        token_wasm_hash: BytesN<32>,
        whitelisted_accounts: Vec<Address>,
        lp_token_decimals: u32,
    ) {
        if is_initialized(&env) {
            panic!("Factory: Initialize: initializing contract twice is not allowed");
        }

        if whitelisted_accounts.is_empty() {
            panic!("Factory: Initialize: there must be at least one whitelisted account able to create liquidity pools.")
        }

        set_initialized(&env);

        let multihop_address =
            deploy_multihop_contract(env.clone(), admin.clone(), multihop_wasm_hash);

        save_config(
            &env,
            Config {
                admin: admin.clone(),
                multihop_address,
                lp_wasm_hash,
                stake_wasm_hash,
                token_wasm_hash,
                whitelisted_accounts,
                lp_token_decimals,
            },
        );

        save_lp_vec(&env, Vec::new(&env));

        env.events()
            .publish(("initialize", "LP factory contract"), admin);
    }

    fn create_liquidity_pool(
        env: Env,
        caller: Address,
        lp_init_info: LiquidityPoolInitInfo,
        pool_name: String,
        pool_symbol: String,
    ) -> Address {
        caller.require_auth();
        if !get_config(&env).whitelisted_accounts.contains(caller) {
            panic!(
                "Factory: Create Liquidity Pool: You are not authorized to create liquidity pool!"
            )
        };

        validate_token_info(
            &env,
            &lp_init_info.token_init_info,
            &lp_init_info.stake_init_info,
        );

        let config = get_config(&env);
        let lp_wasm_hash = config.lp_wasm_hash;
        let stake_wasm_hash = config.stake_wasm_hash;
        let token_wasm_hash = config.token_wasm_hash;

        let lp_contract_address = deploy_lp_contract(
            &env,
            lp_wasm_hash,
            &lp_init_info.token_init_info.token_a,
            &lp_init_info.token_init_info.token_b,
        );

        validate_bps!(
            lp_init_info.swap_fee_bps,
            lp_init_info.max_allowed_slippage_bps,
            lp_init_info.max_allowed_spread_bps,
            lp_init_info.max_referral_bps
        );

        let factory_addr = env.current_contract_address();
        let init_fn: Symbol = Symbol::new(&env, "initialize");
        let init_fn_args: Vec<Val> = (
            stake_wasm_hash,
            token_wasm_hash,
            lp_init_info.clone(),
            factory_addr,
            config.lp_token_decimals,
            pool_name,
            pool_symbol,
        )
            .into_val(&env);

        env.invoke_contract::<Val>(&lp_contract_address, &init_fn, init_fn_args);

        let mut lp_vec = get_lp_vec(&env);

        lp_vec.push_back(lp_contract_address.clone());

        save_lp_vec(&env, lp_vec);
        let token_a = &lp_init_info.token_init_info.token_a;
        let token_b = &lp_init_info.token_init_info.token_b;
        save_lp_vec_with_tuple_as_key(&env, (token_a, token_b), &lp_contract_address);

        env.events()
            .publish(("create", "liquidity_pool"), &lp_contract_address);

        lp_contract_address
    }

    fn update_whitelisted_accounts(
        env: Env,
        sender: Address,
        to_add: Vec<Address>,
        to_remove: Vec<Address>,
    ) {
        sender.require_auth();

        let config = get_config(&env);

        if config.admin != sender {
            panic!(
                "Factory: Create Liquidity Pool: You are not authorized to create liquidity pool!"
            )
        };

        let mut whitelisted_accounts = config.whitelisted_accounts;

        to_add.into_iter().for_each(|addr| {
            if !whitelisted_accounts.contains(addr.clone()) {
                whitelisted_accounts.push_back(addr);
            }
        });

        to_remove.into_iter().for_each(|addr| {
            if let Some(id) = whitelisted_accounts.iter().position(|x| x == addr) {
                whitelisted_accounts.remove(id as u32);
            }
        });

        save_config(
            &env,
            Config {
                whitelisted_accounts,
                ..config
            },
        )
    }

    fn query_pools(env: Env) -> Vec<Address> {
        get_lp_vec(&env)
    }

    fn query_pool_details(env: Env, pool_address: Address) -> LiquidityPoolInfo {
        let pool_response: LiquidityPoolInfo = env.invoke_contract(
            &pool_address,
            &Symbol::new(&env, "query_pool_info_for_factory"),
            Vec::new(&env),
        );
        pool_response
    }

    fn query_all_pools_details(env: Env) -> Vec<LiquidityPoolInfo> {
        let all_lp_vec_addresses = get_lp_vec(&env);
        let mut result = Vec::new(&env);
        for address in all_lp_vec_addresses {
            let pool_response: LiquidityPoolInfo = env.invoke_contract(
                &address,
                &Symbol::new(&env, "query_pool_info_for_factory"),
                Vec::new(&env),
            );

            result.push_back(pool_response);
        }

        result
    }

    fn query_for_pool_by_token_pair(env: Env, token_a: Address, token_b: Address) -> Address {
        let pool_result: Option<Address> = env.storage().persistent().get(&PairTupleKey {
            token_a: token_a.clone(),
            token_b: token_b.clone(),
        });

        if let Some(addr) = pool_result {
            return addr;
        }

        let reverted_pool_resul: Option<Address> = env.storage().persistent().get(&PairTupleKey {
            token_a: token_b,
            token_b: token_a,
        });

        if let Some(addr) = reverted_pool_resul {
            return addr;
        }

        panic!("Factory: query_for_pool_by_token_pair failed: No liquidity pool found");
    }

    fn get_admin(env: Env) -> Address {
        get_config(&env).admin
    }

    fn get_config(env: Env) -> Config {
        env.storage()
            .persistent()
            .get(&DataKey::Config)
            .expect("Factory: No multihop present in storage")
    }
}

fn validate_token_info(
    env: &Env,
    token_init_info: &TokenInitInfo,
    stake_init_info: &StakeInitInfo,
) {
    if token_init_info.token_a >= token_init_info.token_b {
        log!(env, "token_a must be less than token_b");
        panic!("Factory: validate_token_info failed: First token must be smaller then second");
    }

    if stake_init_info.min_bond <= 0 {
        log!(
            env,
            "Minimum amount of lp share tokens to bond can not be smaller or equal to 0"
        );
        panic!("Factory: validate_token_info failed: min stake is less or equal to zero");
    }

    if stake_init_info.min_reward <= 0 {
        log!(env, "min_reward must be bigger then 0!");
        panic!("Factory: validate_token_info failed: min reward too small");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, String};

    #[test]
    #[should_panic(
        expected = "Factory: validate_token_info failed: First token must be smaller then second"
    )]
    fn validate_token_info_should_fail_on_token_a_less_than_token_b() {
        let env = Env::default();

        let token_a = Address::from_string(&String::from_str(
            &env,
            "CBGJMPOZ573XUTIRRFWGWTGSIAOGKJRVMIKBTFYEWTEIU7AEDWKDYMUX",
        ));
        let token_b = Address::from_string(&String::from_str(
            &env,
            "CAOUDQCLN3BYHH4L7GSH3OSQJFVELHKOEVKOPBENVIGZ6WZ5ZRHFC5LN",
        ));

        let token_init_info = TokenInitInfo { token_a, token_b };

        let stake_init_info = StakeInitInfo {
            min_bond: 10,
            min_reward: 10,
            manager: Address::generate(&env),
        };
        validate_token_info(&env, &token_init_info, &stake_init_info);
    }

    #[test]
    #[should_panic(
        expected = "Factory: validate_token_info failed: min stake is less or equal to zero"
    )]
    fn validate_token_info_should_fail_on_min_bond_less_than_zero() {
        let env = Env::default();

        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);

        let token_init_info = TokenInitInfo { token_a, token_b };

        let stake_init_info = StakeInitInfo {
            min_bond: 0,
            min_reward: 10,
            manager: Address::generate(&env),
        };

        validate_token_info(&env, &token_init_info, &stake_init_info);
    }

    #[test]
    #[should_panic(expected = "Factory: validate_token_info failed: min reward too small")]
    fn validate_token_info_should_fail_on_min_reward_less_than_zero() {
        let env = Env::default();

        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);

        let token_init_info = TokenInitInfo { token_a, token_b };

        let stake_init_info = StakeInitInfo {
            min_bond: 10,
            min_reward: 0,
            manager: Address::generate(&env),
        };
        validate_token_info(&env, &token_init_info, &stake_init_info);
    }
}
