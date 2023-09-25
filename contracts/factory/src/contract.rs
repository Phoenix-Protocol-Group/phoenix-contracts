use soroban_sdk::{
    contract, contractimpl, contractmeta, log, Address, Env, IntoVal, Symbol, Val, Vec,
};
use soroban_sdk::arbitrary::std::dbg;

use crate::storage::{LiquidityPoolInfo, PairTupleKey};
use crate::{
    error::ContractError,
    storage::{get_admin, get_lp_vec, save_admin, save_lp_vec, save_lp_vec_with_tuple_as_key},
    utils::deploy_lp_contract,
};
use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "Phoenix Protocol Factory");

#[contract]
pub struct Factory;

pub trait FactoryTrait {
    fn initialize(env: Env, admin: Address) -> Result<(), ContractError>;

    fn create_liquidity_pool(
        env: Env,
        lp_init_info: LiquidityPoolInitInfo,
    ) -> Result<(), ContractError>;

    fn query_pools(env: Env) -> Result<Vec<Address>, ContractError>;

    fn query_pool_details(
        env: Env,
        pool_address: Address,
    ) -> Result<LiquidityPoolInfo, ContractError>;

    fn query_all_pools_details(env: Env) -> Result<Vec<LiquidityPoolInfo>, ContractError>;

    fn query_for_pool_by_pair_tuple(
        env: Env,
        tuple_pair: (Address, Address),
    ) -> Result<Address, ContractError>;

    fn get_admin(env: Env) -> Result<Address, ContractError>;
}

#[contractimpl]
impl FactoryTrait for Factory {
    fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        save_admin(&env, admin.clone());

        dbg!("saved_admin");

        save_lp_vec(&env, Vec::new(&env));

        env.events()
            .publish(("initialize", "LP factory contract"), admin);
        Ok(())
    }

    fn create_liquidity_pool(
        env: Env,
        lp_init_info: LiquidityPoolInitInfo,
    ) -> Result<(), ContractError> {
        validate_token_info(
            &env,
            &lp_init_info.token_init_info,
            &lp_init_info.stake_init_info,
        )?;

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
            lp_init_info.token_init_info.clone(),
            lp_init_info.stake_init_info,
        )
            .into_val(&env);

        let _res: Val = env.invoke_contract(&lp_contract_address, &init_fn, init_fn_args);

        let mut lp_vec = get_lp_vec(&env)?;

        lp_vec.push_back(lp_contract_address.clone());

        save_lp_vec(&env, lp_vec);
        let token_a = &lp_init_info.token_init_info.token_a;
        let token_b = &lp_init_info.token_init_info.token_b;
        save_lp_vec_with_tuple_as_key(&env, (token_a, token_b), &lp_contract_address);

        env.events()
            .publish(("create", "liquidity_pool"), &lp_contract_address);

        Ok(())
    }

    fn query_pools(env: Env) -> Result<Vec<Address>, ContractError> {
        get_lp_vec(&env)
    }

    fn query_pool_details(
        env: Env,
        pool_address: Address,
    ) -> Result<LiquidityPoolInfo, ContractError> {
        let pool_response: LiquidityPoolInfo = env.invoke_contract(
            &pool_address,
            &Symbol::new(&env, "query_pool_info_for_factory"),
            Vec::new(&env),
        );

        Ok(pool_response)
    }

    fn query_all_pools_details(env: Env) -> Result<Vec<LiquidityPoolInfo>, ContractError> {
        let all_lp_vec_addresses = get_lp_vec(&env)?;
        let mut result = Vec::new(&env);
        for address in all_lp_vec_addresses {
            let pool_response: LiquidityPoolInfo = env.invoke_contract(
                &address,
                &Symbol::new(&env, "query_pool_info_for_factory"),
                Vec::new(&env),
            );

            result.push_back(pool_response);
        }

        Ok(result)
    }

    fn query_for_pool_by_pair_tuple(
        env: Env,
        tuple_pair: (Address, Address),
    ) -> Result<Address, ContractError> {
        let pair_result: Option<Address> = env.storage().instance().get(&PairTupleKey {
            token_a: tuple_pair.0.clone(),
            token_b: tuple_pair.1.clone(),
        });

        if let Some(addr) = pair_result {
            return Ok(addr);
        }

        let reverted_pair_resul: Option<Address> = env.storage().instance().get(&PairTupleKey {
            token_a: tuple_pair.1,
            token_b: tuple_pair.0,
        });

        if let Some(addr) = reverted_pair_resul {
            return Ok(addr);
        }

        Err(ContractError::LiquidityPoolPairNotFound)
    }

    fn get_admin(env: Env) -> Result<Address, ContractError> {
        dbg!("got admin");
        get_admin(&env)
    }
}

fn validate_token_info(
    env: &Env,
    token_init_info: &TokenInitInfo,
    stake_init_info: &StakeInitInfo,
) -> Result<(), ContractError> {
    if token_init_info.token_a >= token_init_info.token_b {
        log!(env, "token_a must be less than token_b");
        return Err(ContractError::FirstTokenMustBeSmallerThenSecond);
    }

    if stake_init_info.min_bond <= 0 {
        log!(
            env,
            "Minimum amount of lp share tokens to bond can not be smaller or equal to 0"
        );
        return Err(ContractError::MinStakeLessOrEqualZero);
    }

    if stake_init_info.min_reward <= 0 {
        log!(env, "min_reward must be bigger then 0!");
        return Err(ContractError::MinRewardTooSmall);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::BytesN;

    #[test]
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

        assert_eq!(
            validate_token_info(&env, &token_init_info, &stake_init_info),
            Err(ContractError::FirstTokenMustBeSmallerThenSecond)
        );
    }

    #[test]
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

        assert_eq!(
            validate_token_info(&env, &token_init_info, &stake_init_info),
            Err(ContractError::MinStakeLessOrEqualZero)
        );
    }

    #[test]
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

        assert_eq!(
            validate_token_info(&env, &token_init_info, &stake_init_info),
            Err(ContractError::MinRewardTooSmall)
        );
    }
}
