use super::setup::{
    deploy_factory_contract, install_lp_contract, install_stake_wasm, install_token_wasm,
    lp_contract,
};
use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};

use crate::contract::FactoryClient;
use soroban_sdk::arbitrary::std;
use soroban_sdk::arbitrary::std::dbg;
use soroban_sdk::{
    testutils::{Address as _, BytesN as bN},
    Address, BytesN, Env, Symbol, Vec,
};
use crate::tests::setup::{install_second_lp_contract, install_third_lp_contract};

#[test]
fn test_single_query() {
    let env = Env::default();
    let admin = Address::random(&env);
    let mut token1_admin = Address::random(&env);
    let mut token2_admin = Address::random(&env);
    let user = Address::random(&env);

    let mut token1 = Address::random(&env);
    let mut token2 = Address::random(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    if token2 < token1 {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut token1_admin, &mut token2_admin);
    }

    let factory = deploy_factory_contract(&env, Some(admin.clone()));
    assert_eq!(factory.get_admin(), admin);

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

    let lp_wasm_hash = install_lp_contract(&env);

    let lp_init_info = LiquidityPoolInitInfo {
        admin,
        fee_recipient: user.clone(),
        lp_wasm_hash,
        max_allowed_slippage_bps: 5_000,
        max_allowed_spread_bps: 500,
        share_token_decimals: 7,
        swap_fee_bps: 0,
        token_init_info: token_init_info.clone(),
        stake_init_info,
    };

    factory.create_liquidity_pool(&lp_init_info);
    let lp_contract_addr = factory.query_pools().get(0).unwrap();

    let _first_lp_contract = lp_contract::Client::new(&env, &lp_contract_addr);

    let result = factory.query_pool_details(&lp_contract_addr);
    let share_token_addr: Address = env.invoke_contract(
        &lp_contract_addr,
        &Symbol::new(&env, "query_share_token_address"),
        Vec::new(&env),
    );

    assert_eq!(token1, result.asset_a.address);
    assert_eq!(token2, result.asset_b.address);
    assert_eq!(share_token_addr, result.asset_lp_share.address);
}

#[test]
fn test() {
    let env = Env::default();
    let admin = Address::random(&env);

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    let (token1, token2) = deploy_and_initialize_liquidity_pool(&env, &admin, &factory);
    let (_token3, _token4) = deploy_and_initialize_liquidity_pool(&env, &admin, &factory);
    dbg!(factory.query_pools());

    let first_address = factory.query_pools().get(0).unwrap();

    let _first_lp_contract = lp_contract::Client::new(&env, &first_address);

    let result = factory.query_pool_details(&first_address);
    let share_token_addr: Address = env.invoke_contract(
        &first_address,
        &Symbol::new(&env, "query_share_token_address"),
        Vec::new(&env),
    );

    assert_eq!(token1, result.asset_a.address);
    assert_eq!(token2, result.asset_b.address);
    assert_eq!(share_token_addr, result.asset_lp_share.address);
}

fn deploy_and_initialize_liquidity_pool(
    env: &Env,
    admin: &Address,
    factory: &FactoryClient,
) -> (Address, Address) {
    let mut run_counter = 0;
    let user = Address::random(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    let mut token1 = Address::random(&env);
    let mut token2 = Address::random(&env);

    if token2 < token1 {
        std::mem::swap(&mut token1, &mut token2);
    }

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

    let lp_wasm_hash;

    if run_counter == 0 {
        lp_wasm_hash = install_lp_contract(&env);
    } else if run_counter == 1{
        lp_wasm_hash = install_second_lp_contract(&env);
    } else {
        lp_wasm_hash = install_third_lp_contract(&env);
    }

    // let lp_wasm_hash = BytesN::random(env);
    let lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        lp_wasm_hash,
        max_allowed_slippage_bps: 5_000,
        max_allowed_spread_bps: 500,
        share_token_decimals: 7,
        swap_fee_bps: 0,
        token_init_info: token_init_info.clone(),
        stake_init_info,
    };

    factory.create_liquidity_pool(&lp_init_info);
    run_counter += 1;

    (token1.clone(), token2.clone())
}
