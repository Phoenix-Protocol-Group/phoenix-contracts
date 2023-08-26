use soroban_sdk::{contract, contractimpl, contractmeta, log, Address, Env, Vec};

use crate::{
    error::ContractError,
    storage::{save_admin},
    utils::deploy_lp_contract,
};

use phoenix::{
    utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo},
    lp_contract,
};

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "Phoenix Protocol Factory");

#[contract]
pub struct Factory;

pub trait FactoryTrait {
    fn initialize(env: Env, admin: Address) -> Result<(), ContractError>;

    fn create_liquidity_pool(
        env: Env,
        lp_init_info: LiquidityPoolInitInfo,
        token_init_info: TokenInitInfo,
        stake_init_info: StakeInitInfo,
    ) -> Result<(), ContractError>;

    fn query_pools(env: Env) -> Result<Vec<Address>, ContractError>;
}

#[contractimpl]
impl FactoryTrait for Factory {
    fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        save_admin(&env, admin.clone());

        env.events()
            .publish(("initialize", "LP factory contract"), admin);
        Ok(())
    }

    fn create_liquidity_pool(
        env: Env,
        lp_init_info: LiquidityPoolInitInfo,
        token_init_info: TokenInitInfo,
        stake_init_info: StakeInitInfo,
    ) -> Result<(), ContractError> {
        validate_token_info(&env, &token_init_info)?;

        //deploy lp contract
        let lp_contract_address = deploy_lp_contract(&env, lp_init_info.lp_wasm_hash);
        //init lp contract
        lp_contract::Client::new(&env, &lp_contract_address).initialize(
            &env.current_contract_address(),
            &lp_init_info.share_token_decimals,
            &lp_init_info.swap_fee_bps,
            &lp_init_info.fee_recipient,
            &lp_init_info.max_allowed_slippage_bps,
            &lp_init_info.max_allowed_spread_bps,
            &token_init_info,
            &stake_init_info,
        );

        Ok(())
    }

    fn query_pools(_env: Env) -> Result<Vec<Address>, ContractError> {
        unimplemented!();
    }
}

fn validate_token_info(env: &Env, token_init_info: &TokenInitInfo) -> Result<(), ContractError> {
    let token_a = &token_init_info.token_a;
    let token_b = &token_init_info.token_b;

    if token_a >= token_b {
        log!(&env, "token_a must be less than token_b");
        return Err(ContractError::FirstTokenMustBeSmallerThenSecond);
    }

    //todo add MinStakeLessOrEqualZero and MinRewardTooSmall checks here to fail early

    Ok(())
}
