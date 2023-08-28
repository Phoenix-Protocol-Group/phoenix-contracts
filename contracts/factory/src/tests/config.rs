use super::setup::{
    deploy_factory_contract, install_lp_contract, install_stake_wasm, install_token_wasm,
};
use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn init_config() {
    let env = Env::default();
    let admin = Address::random(&env);
    let user = Address::random(&env);

    let token1 = Address::random(&env);
    let token2 = Address::random(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    let token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: token1.clone(),
        token_b: token2.clone(),
    };
    let stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 10i128,
        max_distributions: 10u32,
        min_reward: 5i128,
    };

    let lp_init_info = LiquidityPoolInitInfo {
        admin,
        fee_recipient: user.clone(),
        lp_wasm_hash: install_lp_contract(&env),
        max_allowed_slippage_bps: 5_000,
        max_allowed_spread_bps: 500,
        share_token_decimals: 7,
        stake_init_info,
        swap_fee_bps: 0,
        token_init_info,
    };

    factory.create_liquidity_pool(&lp_init_info, &token_init_info, &stake_init_info);
}
