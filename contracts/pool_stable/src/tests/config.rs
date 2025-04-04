extern crate std;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

use super::setup::{deploy_stable_liquidity_pool_contract, deploy_token_contract};
use crate::storage::{Config, PairType};

#[test]
fn update_config() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let user1 = Address::generate(&env);
    let stake_manager = Address::generate(&env);
    let factory = Address::generate(&env);
    let swap_fees = 0i64;
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        Some(admin1.clone()),
        (&token1.address, &token2.address),
        swap_fees,
        user1.clone(),
        500,
        200,
        stake_manager,
        factory,
        None,
    );

    let share_token_address = pool.query_share_token_address();
    let stake_token_address = pool.query_stake_contract_address();

    assert_eq!(
        pool.query_config(),
        Config {
            token_a: token1.address.clone(),
            token_b: token2.address.clone(),
            share_token: share_token_address.clone(),
            stake_contract: stake_token_address.clone(),
            pool_type: PairType::Stable,
            total_fee_bps: 0,
            fee_recipient: user1,
            max_allowed_slippage_bps: 500,
            default_slippage_bps: 2_500,
            max_allowed_spread_bps: 200,
        }
    );

    // update fees and recipient
    pool.update_config(
        &admin1,
        &None,
        &Some(500i64), // 5% fees
        &Some(admin2.clone()),
        &None,
        &None,
    );
    assert_eq!(
        pool.query_config(),
        Config {
            token_a: token1.address.clone(),
            token_b: token2.address.clone(),
            share_token: share_token_address.clone(),
            stake_contract: stake_token_address.clone(),
            pool_type: PairType::Stable,
            total_fee_bps: 500,
            fee_recipient: admin2.clone(),
            max_allowed_slippage_bps: 500,
            default_slippage_bps: 2_500,
            max_allowed_spread_bps: 200,
        }
    );

    // update slippage and spread
    pool.update_config(&admin1, &None, &None, &None, &Some(5_000i64), &Some(500));
    assert_eq!(
        pool.query_config(),
        Config {
            token_a: token1.address.clone(),
            token_b: token2.address,
            share_token: share_token_address,
            stake_contract: stake_token_address,
            pool_type: PairType::Stable,
            total_fee_bps: 500,
            fee_recipient: admin2,
            max_allowed_slippage_bps: 5_000,
            default_slippage_bps: 2_500,
            max_allowed_spread_bps: 500,
        }
    );
}

#[test]
#[should_panic(expected = "Pool Stable: UpdateConfig: Unauthorize")]
fn update_config_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);
    let user1 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let swap_fees = 0i64;
    let stake_manager = Address::generate(&env);
    let factory = Address::generate(&env);
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        Some(admin1.clone()),
        (&token1.address, &token2.address),
        swap_fees,
        user1,
        500,
        200,
        stake_manager,
        factory,
        None,
    );

    pool.update_config(
        &Address::generate(&env),
        &None,
        &Some(500i64), // 5% fees
        &Some(admin2.clone()),
        &None,
        &None,
    );
}

#[test]
fn update_config_update_admin() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);
    let user1 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let swap_fees = 0i64;
    let stake_manager = Address::generate(&env);
    let factory = Address::generate(&env);
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        Some(admin1.clone()),
        (&token1.address, &token2.address),
        swap_fees,
        user1.clone(),
        500,
        200,
        stake_manager,
        factory,
        None,
    );

    // update admin to new admin
    pool.update_config(&admin1, &Some(admin2.clone()), &None, &None, &None, &None);

    let share_token_address = pool.query_share_token_address();
    let stake_token_address = pool.query_stake_contract_address();

    // now update succeeds
    pool.update_config(&admin2, &None, &None, &None, &None, &Some(3_000));
    assert_eq!(
        pool.query_config(),
        Config {
            token_a: token1.address.clone(),
            token_b: token2.address,
            share_token: share_token_address,
            stake_contract: stake_token_address,
            pool_type: PairType::Stable,
            total_fee_bps: 0,
            fee_recipient: user1,
            max_allowed_slippage_bps: 500,
            default_slippage_bps: 2_500,
            max_allowed_spread_bps: 3_000,
        }
    );
}

#[test]
#[should_panic(expected = "The value 10100 is out of range. Must be between 0 and 10000 bps.")]
fn update_config_too_high_fees() {
    let env = Env::default();
    env.mock_all_auths();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);
    let user1 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let swap_fees = 0i64;
    let stake_manager = Address::generate(&env);
    let factory = Address::generate(&env);
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        Some(admin1.clone()),
        (&token1.address, &token2.address),
        swap_fees,
        user1,
        500,
        200,
        stake_manager,
        factory,
        None,
    );

    // update fees and recipient
    pool.update_config(
        &admin1,
        &None,
        &Some(10_100i64), // 101% fees
        &Some(admin2.clone()),
        &None,
        &None,
    );
}

#[test]
#[should_panic(expected = "Pool Stable: Initialize: AMP parameter is incorrect")]
fn initialize_with_incorrect_amp() {
    let env = Env::default();
    env.mock_all_auths();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);
    let user1 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let swap_fees = 0i64;
    let stake_manager = Address::generate(&env);
    let factory = Address::generate(&env);
    deploy_stable_liquidity_pool_contract(
        &env,
        Some(admin1.clone()),
        (&token1.address, &token2.address),
        swap_fees,
        user1,
        500,
        200,
        stake_manager,
        factory,
        0, // init AMP
    );
}

#[test]
fn update_config_all_bps_params_should_work() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let user1 = Address::generate(&env);
    let stake_manager = Address::generate(&env);
    let factory = Address::generate(&env);
    let swap_fees = 0i64;
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        Some(admin1.clone()),
        (&token1.address, &token2.address),
        swap_fees,
        user1.clone(),
        500,
        200,
        stake_manager,
        factory,
        None,
    );

    let share_token_address = pool.query_share_token_address();
    let stake_token_address = pool.query_stake_contract_address();

    assert_eq!(
        pool.query_config(),
        Config {
            token_a: token1.address.clone(),
            token_b: token2.address.clone(),
            share_token: share_token_address.clone(),
            stake_contract: stake_token_address.clone(),
            pool_type: PairType::Stable,
            total_fee_bps: 0,
            fee_recipient: user1,
            max_allowed_slippage_bps: 500,
            max_allowed_spread_bps: 200,
            default_slippage_bps: 2_500,
        }
    );

    // update all bps to 10%
    pool.update_config(
        &admin1,
        &None,
        &Some(1000i64),
        &Some(admin2.clone()),
        &Some(1000),
        &Some(1000),
    );
    assert_eq!(
        pool.query_config(),
        Config {
            token_a: token1.address.clone(),
            token_b: token2.address.clone(),
            share_token: share_token_address.clone(),
            stake_contract: stake_token_address.clone(),
            pool_type: PairType::Stable,
            total_fee_bps: 1000,
            fee_recipient: admin2.clone(),
            max_allowed_slippage_bps: 1000,
            max_allowed_spread_bps: 1000,
            default_slippage_bps: 2_500,
        }
    );
}

#[test]
#[should_panic(expected = "Pool: Initialize: swap fee is higher than the maximum allowed!")]
fn create_stable_pool_with_too_high_swap_fee_bps_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);
    let user1 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let swap_fees = 2_000; // admin wants %20 swap fee, but maximum allowed is %10
    let stake_manager = Address::generate(&env);
    let factory = Address::generate(&env);
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        Some(admin1.clone()),
        (&token1.address, &token2.address),
        swap_fees,
        user1,
        500,
        200,
        stake_manager,
        factory,
        None,
    );

    // update fees and recipient
    pool.update_config(
        &admin1,
        &None,
        &Some(10_100i64), // 101% fees
        &Some(admin2.clone()),
        &None,
        &None,
    );
}

#[test]
fn test_version_query() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let user1 = Address::generate(&env);
    let stake_manager = Address::generate(&env);
    let factory = Address::generate(&env);
    let swap_fees = 0i64;
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        Some(admin1.clone()),
        (&token1.address, &token2.address),
        swap_fees,
        user1.clone(),
        500,
        200,
        stake_manager,
        factory,
        None,
    );
    let expected_version = env!("CARGO_PKG_VERSION");
    let version = pool.query_version();
    assert_eq!(String::from_str(&env, expected_version), version);
}
