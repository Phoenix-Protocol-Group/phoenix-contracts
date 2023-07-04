extern crate std;
use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal, Symbol};

use super::setup::{deploy_liquidity_pool_contract, deploy_token_contract};
use crate::{
    storage::{Asset, PoolResponse},
    token_contract,
};

// TODO: add more edge cases
// - exceed slippage
// - try to exceed spread during single amount liqudity providing

#[test]
fn provide_liqudity() {
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
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        &token1.address,
        &token2.address,
        swap_fees,
        None,
        None,
        None,
    );

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    token1.mint(&user1, &1000);
    assert_eq!(token1.balance(&user1), 1000);

    token2.mint(&user1, &1000);
    assert_eq!(token2.balance(&user1), 1000);

    pool.provide_liquidity(
        &user1,
        &Some(100),
        &Some(100),
        &Some(100),
        &Some(100),
        &None,
    );
    assert_eq!(
        env.auths(),
        [
            (
                user1.clone(),
                pool.address.clone(),
                Symbol::new(&env, "provide_liquidity"),
                (&user1, 100_i128, 100_i128, 100_i128, 100_i128, None::<i64>).into_val(&env)
            ),
            (
                user1.clone(),
                token1.address.clone(),
                Symbol::short("transfer"),
                (&user1, &pool.address, 100_i128).into_val(&env)
            ),
            (
                user1.clone(),
                token2.address.clone(),
                Symbol::short("transfer"),
                (&user1, &pool.address, 100_i128).into_val(&env)
            ),
        ]
    );

    assert_eq!(token_share.balance(&user1), 100);
    assert_eq!(token_share.balance(&pool.address), 0);
    assert_eq!(token1.balance(&user1), 900);
    assert_eq!(token1.balance(&pool.address), 100);
    assert_eq!(token2.balance(&user1), 900);
    assert_eq!(token2.balance(&pool.address), 100);

    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address,
                amount: 100i128
            },
            asset_b: Asset {
                address: token2.address,
                amount: 100i128
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 100i128
            }
        }
    );
}

#[test]
fn withdraw_liqudity() {
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
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        &token1.address,
        &token2.address,
        swap_fees,
        None,
        None,
        None,
    );

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    token1.mint(&user1, &100);
    token2.mint(&user1, &100);
    pool.provide_liquidity(
        &user1,
        &Some(100),
        &Some(100),
        &Some(100),
        &Some(100),
        &None,
    );

    assert_eq!(token1.balance(&user1), 0);
    assert_eq!(token1.balance(&pool.address), 100);
    assert_eq!(token2.balance(&user1), 0);
    assert_eq!(token2.balance(&pool.address), 100);

    let share_amount = 50;
    let min_a = 50;
    let min_b = 50;
    pool.withdraw_liquidity(&user1, &share_amount, &min_a, &min_b);
    assert_eq!(
        env.auths(),
        [
            (
                user1.clone(),
                pool.address.clone(),
                Symbol::new(&env, "withdraw_liquidity"),
                (&user1, 50_i128, 50_i128, 50_i128).into_val(&env)
            ),
            (
                user1.clone(),
                share_token_address.clone(),
                Symbol::short("transfer"),
                (&user1, &pool.address, 50_i128).into_val(&env)
            )
        ]
    );

    assert_eq!(token_share.balance(&user1), 50);
    assert_eq!(token_share.balance(&pool.address), 0); // sanity check
    assert_eq!(token1.balance(&user1), 50);
    assert_eq!(token1.balance(&pool.address), 50);
    assert_eq!(token2.balance(&user1), 50);
    assert_eq!(token2.balance(&pool.address), 50);

    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 50i128
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 50i128
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 50i128
            }
        }
    );

    // clear the pool
    pool.withdraw_liquidity(&user1, &share_amount, &min_a, &min_b);
    assert_eq!(token_share.balance(&user1), 0);
    assert_eq!(token_share.balance(&pool.address), 0); // sanity check
    assert_eq!(token1.balance(&user1), 100);
    assert_eq!(token1.balance(&pool.address), 0);
    assert_eq!(token2.balance(&user1), 100);
    assert_eq!(token2.balance(&pool.address), 0);
}

#[test]
#[should_panic = "Status(ContractError(12))"]
fn provide_liqudity_single_asset_on_empty_pool() {
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
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        &token1.address,
        &token2.address,
        swap_fees,
        None,
        None,
        None,
    );

    token1.mint(&user1, &1_000_000);

    // providing liquidity with single asset is not allowed on an empty pool
    pool.provide_liquidity(
        &user1,
        &Some(1_000_000),
        &Some(1_000_000),
        &None,
        &None,
        &None,
    );
}

#[test]
fn provide_liqudity_single_asset_equal() {
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
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        &token1.address,
        &token2.address,
        swap_fees,
        None,
        None,
        None,
    );

    token1.mint(&user1, &10_000_000);
    token2.mint(&user1, &10_000_000);

    // providing liquidity with single asset is not allowed on an empty pool
    pool.provide_liquidity(
        &user1,
        &Some(10_000_000),
        &Some(10_000_000),
        &Some(10_000_000),
        &Some(10_000_000),
        &None,
    );
    assert_eq!(token1.balance(&pool.address), 10_000_000);
    assert_eq!(token2.balance(&pool.address), 10_000_000);

    token1.mint(&user1, &100_000);
    // Providing 100k of token1 to 1:1 pool will perform swap which will create imbalance
    pool.provide_liquidity(
        &user1,
        &Some(100_000),
        &Some(50_000),
        &None,
        &Some(49_000),
        &None,
    );
    // before swap : A(10_000_000), B(10_000_000)
    // since pool is equal divides 50/50 sum for swap
    // swap 50k A for B = 49752
    // after swap : A(10_050_000), B(9_950_248)
    // after providing liquidity
    // A(1_100_000), B(1_000_000)

    assert_eq!(token1.balance(&pool.address), 10_100_000);
    // because of lack of fees, first swap took from pool b exact amount
    // that was provided to the pool in the next step
    assert_eq!(token2.balance(&pool.address), 10_000_000);
    assert_eq!(token1.balance(&user1), 0);
    assert_eq!(token2.balance(&user1), 0);
}

// FIXME
// This test  won't work now because my algorithm to split tokens is invalid.
#[test]
fn provide_liqudity_single_asset_one_third() {
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
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        &token1.address,
        &token2.address,
        swap_fees,
        None,
        None,
        None,
    );

    token1.mint(&user1, &10_000_000);
    token2.mint(&user1, &30_000_000);

    // providing liquidity with single asset is not allowed on an empty pool
    pool.provide_liquidity(
        &user1,
        &Some(10_000_000),
        &Some(10_000_000),
        &Some(30_000_000),
        &Some(30_000_000),
        &None,
    );
    assert_eq!(token1.balance(&pool.address), 10_000_000);
    assert_eq!(token2.balance(&pool.address), 30_000_000);

    token2.mint(&user1, &100_000);
    // Providing 100k of token2 to 1:3 pool will perform swap which will create imbalance
    let slippage_tolerance_bps = 5000; // 3%
    pool.provide_liquidity(
        &user1,
        &None,
        &None,
        &Some(100_000),
        &None,
        &Some(slippage_tolerance_bps),
    );
    // before swap : A(10_000_000), B(30_000_000)
    // since pool is 1/3 divides 75k/25k sum for swap
    // swap 25k B for A = 8327
    // after swap : A(9_991_673), B(30_025_000)
    // after providing liquidity
    // A(10_000_000), B(30_100_000)

    assert_eq!(token2.balance(&pool.address), 30_100_000);
    assert_eq!(token1.balance(&pool.address), 10_000_000);
}
