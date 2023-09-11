use super::setup::{
    deploy_factory_contract, install_lp_contract, install_stake_wasm, install_token_wasm,
    lp_contract,
};
use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};

use soroban_sdk::arbitrary::std;
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol, Vec};

#[test]
fn test_single_query() {
    let env = Env::default();
    let admin = Address::random(&env);
    let user = Address::random(&env);

    let mut token1 = Address::random(&env);
    let mut token2 = Address::random(&env);
    let mut token3 = Address::random(&env);
    let mut token4 = Address::random(&env);
    let mut token5 = Address::random(&env);
    let mut token6 = Address::random(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    if token2 < token1 {
        std::mem::swap(&mut token1, &mut token2);
    }

    if token4 < token3 {
        std::mem::swap(&mut token3, &mut token4);
    }

    if token6 < token5 {
        std::mem::swap(&mut token5, &mut token6);
    }

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    let first_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: token1.clone(),
        token_b: token2.clone(),
    };
    let first_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 10i128,
        max_distributions: 10u32,
        min_reward: 5i128,
    };

    let second_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: token3.clone(),
        token_b: token4.clone(),
    };
    let second_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 5i128,
        max_distributions: 5u32,
        min_reward: 2i128,
    };

    let lp_wasm_hash = install_lp_contract(&env);

    let first_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 5_000,
        max_allowed_spread_bps: 500,
        share_token_decimals: 7,
        swap_fee_bps: 0,
        token_init_info: first_token_init_info.clone(),
        stake_init_info: first_stake_init_info,
    };

    let second_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 4_000,
        max_allowed_spread_bps: 400,
        share_token_decimals: 6,
        swap_fee_bps: 0,
        token_init_info: second_token_init_info.clone(),
        stake_init_info: second_stake_init_info,
    };

    factory.create_liquidity_pool(&first_lp_init_info);
    // uncommenting the line below brakes the tests
    // we use the same lp_wasm_hash and this causes HostError: Error(Storage, ExistingValue)
    factory.create_liquidity_pool(&second_lp_init_info);
    let lp_contract_addr = factory.query_pools().get(0).unwrap();

    let _first_lp_contract = lp_contract::Client::new(&env, &lp_contract_addr);

    let result = factory.query_pool_details(&lp_contract_addr);
    let share_token_addr: Address = env.invoke_contract(
        &lp_contract_addr,
        &Symbol::new(&env, "query_share_token_address"),
        Vec::new(&env),
    );

    assert_eq!(token1, result.pool_response.asset_a.address);
    assert_eq!(token2, result.pool_response.asset_b.address);
    assert_eq!(share_token_addr, result.pool_response.asset_lp_share.address);
}
