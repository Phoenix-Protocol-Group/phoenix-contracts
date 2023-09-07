use soroban_sdk::{
    contract, contractimpl, contractmeta, log, Address, Env, IntoVal, Symbol, Val, Vec,
};

use crate::storage::{Asset, PoolResponse};
use crate::{
    error::ContractError,
    storage::{get_admin, get_lp_vec, save_admin, save_lp_vec},
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

    fn query_pools(env: Env) -> Result<Vec<PoolResponse>, ContractError>;

    fn get_admin(env: Env) -> Result<Address, ContractError>;
}

#[contractimpl]
impl FactoryTrait for Factory {
    fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        save_admin(&env, admin.clone());

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

        let lp_contract_address = deploy_lp_contract(&env, lp_init_info.lp_wasm_hash);

        let init_fn: Symbol = Symbol::new(&env, "initialize");
        let init_fn_args: Vec<Val> = (
            lp_init_info.admin,
            lp_init_info.share_token_decimals,
            lp_init_info.swap_fee_bps,
            lp_init_info.fee_recipient,
            lp_init_info.max_allowed_slippage_bps,
            lp_init_info.max_allowed_spread_bps,
            lp_init_info.token_init_info,
            lp_init_info.stake_init_info,
        )
            .into_val(&env);
        let _res: Val = env.invoke_contract(&lp_contract_address, &init_fn, init_fn_args);

        let pool_response: PoolResponse = env.invoke_contract(
            &lp_contract_address,
            &Symbol::new(&env, "query_pool_info"),
            Vec::new(&env),
        );

        let mut lp_vec = get_lp_vec(&env)?;

        // move PoolResponse and Asset to the phoenix util library?
        let lp_to_save = PoolResponse {
            asset_a: Asset {
                address: pool_response.asset_a.address,
                amount: pool_response.asset_a.amount,
            },
            asset_b: Asset {
                address: pool_response.asset_b.address,
                amount: pool_response.asset_b.amount,
            },
            asset_lp_share: Asset {
                address: pool_response.asset_lp_share.address,
                amount: pool_response.asset_lp_share.amount,
            },
        };

        lp_vec.push_back(lp_to_save);

        // A few things are blurry for me
        // 1.   Which fees exactly are we gonna send back to the client (my guess it's swap fees,
        //      but not 100% sure).
        // 2.   Isn't this a bit an expensive operation to make (query all the lp_contract on ledger
        //      Initially it'll be fine, but as soon as our ledger grows it can be costly. Maybe
        //      we can specify which addresses to query for?
        // 2.1. Initially the total supply will be the same so we might just end not calling the
        //      lp_contract rather use the values from the initialization
        // 2.2  Am I correct to think that in order to have consistent information on ledger we need
        //      to call some new method update_lp_info() everytime a swap is made?
        // 3.   Is the factory contract the one that should be responsible for this type of
        //      operations - update the storage associated with the liquidity pools bookkeeping?
        //      Isn't this contract supposed to only create new liquidity_pools.
        // 4.   Off-topic - can we rename pair contract to liquidity_pool.

        save_lp_vec(&env, lp_vec);

        env.events()
            .publish(("create", "liquidity_pool"), &lp_contract_address);

        Ok(())
    }

    fn query_pools(env: Env) -> Result<Vec<PoolResponse>, ContractError> {
        get_lp_vec(&env)
    }

    fn get_admin(env: Env) -> Result<Address, ContractError> {
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
