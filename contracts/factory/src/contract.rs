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
use soroban_sdk::{
    contract, contractimpl, contractmeta, log, Address, BytesN, Env, IntoVal, Symbol, Val, Vec,
};

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "Phoenix Protocol Factory");

#[contract]
pub struct Factory;

pub trait FactoryTrait {
    fn initialize(env: Env, admin: Address, multihop_wasm_hash: BytesN<32>);

    fn create_liquidity_pool(env: Env, lp_init_info: LiquidityPoolInitInfo) -> Address;

    fn query_pools(env: Env) -> Vec<Address>;

    fn query_pool_details(env: Env, pool_address: Address) -> LiquidityPoolInfo;

    fn query_all_pools_details(env: Env) -> Vec<LiquidityPoolInfo>;

    fn query_for_pool_by_token_pair(env: Env, token_a: Address, token_b: Address) -> Address;

    fn get_admin(env: Env) -> Address;

    fn get_config(env: Env) -> Config;
}

#[contractimpl]
impl FactoryTrait for Factory {
    fn initialize(env: Env, admin: Address, multihop_wasm_hash: BytesN<32>) {
        if is_initialized(&env) {
            panic!("Factory: Initialize: initializing contract twice is not allowed");
        }

        set_initialized(&env);

        let multihop_address =
            deploy_multihop_contract(env.clone(), admin.clone(), multihop_wasm_hash);

        save_config(
            &env,
            Config {
                admin: admin.clone(),
                multihop_address,
            },
        );

        save_lp_vec(&env, Vec::new(&env));

        env.events()
            .publish(("initialize", "LP factory contract"), admin);
    }

    fn create_liquidity_pool(env: Env, lp_init_info: LiquidityPoolInitInfo) -> Address {
        validate_token_info(
            &env,
            &lp_init_info.token_init_info,
            &lp_init_info.stake_init_info,
        );

        let lp_contract_address = deploy_lp_contract(
            &env,
            lp_init_info.lp_wasm_hash,
            &lp_init_info.token_init_info.token_a,
            &lp_init_info.token_init_info.token_b,
        );

        let init_fn: Symbol = Symbol::new(&env, "initialize");
        let init_fn_args: Vec<Val> = (
            lp_init_info.admin,
            lp_init_info.share_token_decimals,
            lp_init_info.swap_fee_bps,
            lp_init_info.fee_recipient,
            lp_init_info.max_allowed_slippage_bps,
            lp_init_info.max_allowed_spread_bps,
            lp_init_info.max_referral_bps,
            lp_init_info.token_init_info.clone(),
            lp_init_info.stake_init_info,
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
        let pool_result: Option<Address> = env.storage().instance().get(&PairTupleKey {
            token_a: token_a.clone(),
            token_b: token_b.clone(),
        });

        if let Some(addr) = pool_result {
            return addr;
        }

        let reverted_pool_resul: Option<Address> = env.storage().instance().get(&PairTupleKey {
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
    use soroban_sdk::BytesN;

    #[test]
    #[should_panic(
        expected = "Factory: validate_token_info failed: First token must be smaller then second"
    )]
    fn validate_token_info_should_fail_on_token_a_less_than_token_b() {
        let env = Env::default();

        let contract1 = BytesN::from_array(&env, &[1u8; 0x20]);
        let contract2 = BytesN::from_array(&env, &[0u8; 0x20]);

        let token_wasm_hash = BytesN::from_array(&env, &[8u8; 0x20]);
        let stake_wasm_hash = BytesN::from_array(&env, &[15u8; 0x20]);

        let token_a = Address::from_contract_id(&contract1);
        let token_b = Address::from_contract_id(&contract2);

        let token_init_info = TokenInitInfo {
            token_a,
            token_b,
            token_wasm_hash,
        };

        let stake_init_info = StakeInitInfo {
            max_distributions: 10,
            min_bond: 10,
            min_reward: 10,
            stake_wasm_hash,
        };
        validate_token_info(&env, &token_init_info, &stake_init_info);
    }

    #[test]
    #[should_panic(
        expected = "Factory: validate_token_info failed: min stake is less or equal to zero"
    )]
    fn validate_token_info_should_fail_on_min_bond_less_than_zero() {
        let env = Env::default();

        let contract1 = BytesN::from_array(&env, &[0u8; 0x20]);
        let contract2 = BytesN::from_array(&env, &[1u8; 0x20]);

        let token_wasm_hash = BytesN::from_array(&env, &[8u8; 0x20]);
        let stake_wasm_hash = BytesN::from_array(&env, &[15u8; 0x20]);

        let token_a = Address::from_contract_id(&contract1);
        let token_b = Address::from_contract_id(&contract2);

        let token_init_info = TokenInitInfo {
            token_a,
            token_b,
            token_wasm_hash,
        };

        let stake_init_info = StakeInitInfo {
            max_distributions: 10,
            min_bond: 0,
            min_reward: 10,
            stake_wasm_hash,
        };

        validate_token_info(&env, &token_init_info, &stake_init_info);
    }

    #[test]
    #[should_panic(expected = "Factory: validate_token_info failed: min reward too small")]
    fn validate_token_info_should_fail_on_min_reward_less_than_zero() {
        let env = Env::default();

        let contract1 = BytesN::from_array(&env, &[0u8; 0x20]);
        let contract2 = BytesN::from_array(&env, &[1u8; 0x20]);

        let token_wasm_hash = BytesN::from_array(&env, &[8u8; 0x20]);
        let stake_wasm_hash = BytesN::from_array(&env, &[15u8; 0x20]);

        let token_a = Address::from_contract_id(&contract1);
        let token_b = Address::from_contract_id(&contract2);

        let token_init_info = TokenInitInfo {
            token_a,
            token_b,
            token_wasm_hash,
        };

        let stake_init_info = StakeInitInfo {
            max_distributions: 10,
            min_bond: 10,
            min_reward: 0,
            stake_wasm_hash,
        };
        validate_token_info(&env, &token_init_info, &stake_init_info);
    }
}
