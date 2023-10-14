extern crate std;
use soroban_sdk::{testutils::Address as _, Address, Env};

use super::setup::{deploy_stable_liquidity_pool_contract, deploy_token_contract};
use crate::storage::{Config, PairType};

#[test]
fn update_config() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let mut admin1 = Address::random(&env);
    let mut admin2 = Address::random(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let user1 = Address::random(&env);
    let swap_fees = 0i64;
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        Some(admin1.clone()),
        (&token1.address, &token2.address),
        swap_fees,
        user1.clone(),
        500,
        200,
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
            pool_type: PairType::Xyk,
            total_fee_bps: 0,
            fee_recipient: user1,
            max_allowed_slippage_bps: 500,
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
            pool_type: PairType::Xyk,
            total_fee_bps: 500,
            fee_recipient: admin2.clone(),
            max_allowed_slippage_bps: 500,
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
            pool_type: PairType::Xyk,
            total_fee_bps: 500,
            fee_recipient: admin2,
            max_allowed_slippage_bps: 5_000,
            max_allowed_spread_bps: 500,
        }
    );
}

#[test]
#[should_panic(expected = "Pool: UpdateConfig: Unauthorize")]
fn update_config_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let mut admin1 = Address::random(&env);
    let mut admin2 = Address::random(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let user1 = Address::random(&env);
    let swap_fees = 0i64;
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        Some(admin1.clone()),
        (&token1.address, &token2.address),
        swap_fees,
        user1,
        500,
        200,
    );

    pool.update_config(
        &Address::random(&env),
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
    env.budget().reset_unlimited();

    let mut admin1 = Address::random(&env);
    let mut admin2 = Address::random(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let user1 = Address::random(&env);
    let swap_fees = 0i64;
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        Some(admin1.clone()),
        (&token1.address, &token2.address),
        swap_fees,
        user1.clone(),
        500,
        200,
    );

    // update admin to new admin
    pool.update_config(&admin1, &Some(admin2.clone()), &None, &None, &None, &None);

    let share_token_address = pool.query_share_token_address();
    let stake_token_address = pool.query_stake_contract_address();

    // now update succeeds
    pool.update_config(&admin2, &None, &None, &None, &None, &Some(3_000_000));
    assert_eq!(
        pool.query_config(),
        Config {
            token_a: token1.address.clone(),
            token_b: token2.address,
            share_token: share_token_address,
            stake_contract: stake_token_address,
            pool_type: PairType::Xyk,
            total_fee_bps: 0,
            fee_recipient: user1,
            max_allowed_slippage_bps: 500,
            max_allowed_spread_bps: 3_000_000,
        }
    );
}

#[test]
#[should_panic(expected = "Pool: UpdateConfig: Invalid total_fee_bps")]
fn update_config_too_high_fees() {
    let env = Env::default();
    env.mock_all_auths();

    let mut admin1 = Address::random(&env);
    let mut admin2 = Address::random(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let user1 = Address::random(&env);
    let swap_fees = 0i64;
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        Some(admin1.clone()),
        (&token1.address, &token2.address),
        swap_fees,
        user1,
        500,
        200,
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
