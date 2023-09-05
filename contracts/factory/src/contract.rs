use soroban_sdk::{contract, contractimpl, contractmeta, log, Address, Env, Vec};

use crate::{
    error::ContractError,
    lp_contract,
    storage::{get_admin, get_lp_vec, save_admin, save_lp_vec},
    utils::deploy_lp_contract,
};

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "Phoenix Protocol Factory");

#[contract]
pub struct Factory;

pub trait FactoryTrait {
    fn initialize(env: Env, admin: Address) -> Result<(), ContractError>;

    fn create_liquidity_pool(
        env: Env,
        lp_init_info: lp_contract::LiquidityPoolInitInfo,
        token_init_info: lp_contract::TokenInitInfo,
        stake_init_info: lp_contract::StakeInitInfo,
    ) -> Result<(), ContractError>;

    fn query_pools(env: Env) -> Result<Vec<Address>, ContractError>;

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
        lp_init_info: lp_contract::LiquidityPoolInitInfo,
        token_init_info: lp_contract::TokenInitInfo,
        stake_init_info: lp_contract::StakeInitInfo,
    ) -> Result<(), ContractError> {
        validate_token_info(&env, &token_init_info, &stake_init_info)?;

        let lp_contract_address = deploy_lp_contract(&env, lp_init_info.lp_wasm_hash);

        lp_contract::Client::new(&env, &lp_contract_address).initialize(
            &get_admin(&env)?,
            &lp_init_info.share_token_decimals,
            &lp_init_info.swap_fee_bps,
            &lp_init_info.fee_recipient,
            &lp_init_info.max_allowed_slippage_bps,
            &lp_init_info.max_allowed_spread_bps,
            &token_init_info,
            &stake_init_info,
        );

        let mut lp_vec = get_lp_vec(&env)?;

        lp_vec.push_back(lp_contract_address.clone());

        save_lp_vec(&env, lp_vec);

        env.events()
            .publish(("create", "liquidity_pool"), &lp_contract_address);

        Ok(())
    }

    fn query_pools(env: Env) -> Result<Vec<Address>, ContractError> {
        get_lp_vec(&env)
    }

    fn get_admin(env: Env) -> Result<Address, ContractError> {
        get_admin(&env)
    }
}

fn validate_token_info(
    env: &Env,
    token_init_info: &lp_contract::TokenInitInfo,
    stake_init_info: &lp_contract::StakeInitInfo,
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

        let token_init_info = lp_contract::TokenInitInfo {
            token_a,
            token_b,
            token_wasm_hash,
        };

        let stake_init_info = lp_contract::StakeInitInfo {
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

        let token_init_info = lp_contract::TokenInitInfo {
            token_a,
            token_b,
            token_wasm_hash,
        };

        let stake_init_info = lp_contract::StakeInitInfo {
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

        let token_init_info = lp_contract::TokenInitInfo {
            token_a,
            token_b,
            token_wasm_hash,
        };

        let stake_init_info = lp_contract::StakeInitInfo {
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
