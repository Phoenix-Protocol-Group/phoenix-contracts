use soroban_sdk::{contract, contractimpl, contractmeta, log, Address, Env, Vec};

use crate::{
    error::ContractError,
    lp_contract,
    storage::{utils, LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo},
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
        utils::save_admin(&env, admin.clone());

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

        let lp_contract_address = utils::deploy_lp_contract(
            &env,
            lp_init_info.lp_wasm_hash,
            lp_init_info.share_token_decimals,
            lp_init_info.swap_fee_bps,
            lp_init_info.fee_recipient,
            lp_init_info.max_allowed_slippage_bps,
            lp_init_info.max_allowed_spread_bps,
        );

        lp_contract::Client::new(&env, &lp_contract_address).initialize(
            &utils::get_admin(&env)?,
            &lp_init_info.share_token_decimals,
            &lp_init_info.swap_fee_bps,
            &lp_init_info.fee_recipient,
            &lp_init_info.max_allowed_slippage_bps,
            &lp_init_info.max_allowed_spread_bps,
            // pick it up from here
            &token_init_info,
            &stake_init_info,
        );
        // init liquidity_pool with lp specific info
        // pass the token and stake contract info into it
        // let the underlying actions do the work

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
