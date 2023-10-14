extern crate std;

use pretty_assertions::assert_eq;
use soroban_sdk::testutils::{AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, IntoVal};

use super::setup::{deploy_stable_liquidity_pool_contract, deploy_token_contract};
use crate::storage::{Asset, PoolResponse, SimulateReverseSwapResponse, SimulateSwapResponse};
use decimal::Decimal;

#[test]
fn simple_swap() {
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
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
    );

    token1.mint(&user1, &1_001_000);
    token2.mint(&user1, &1_001_000);
    pool.provide_liquidity(
        &user1,
        &Some(1_000_000),
        &Some(1_000_000),
        &Some(1_000_000),
        &Some(1_000_000),
        &None,
    );

    // true means "selling A token"
    // selling just one token with 1% max spread allowed
    let spread = 100i64; // 1% maximum spread allowed
    pool.swap(&user1, &token1.address, &1, &None, &Some(spread));
    assert_eq!(
        env.auths(),
        [(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    pool.address.clone(),
                    symbol_short!("swap"),
                    (&user1, token1.address.clone(), 1_i128, None::<i64>, spread).into_val(&env)
                )),
                sub_invocations: std::vec![
                    (AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            token1.address.clone(),
                            symbol_short!("transfer"),
                            (&user1, &pool.address, 1_i128).into_val(&env)
                        )),
                        sub_invocations: std::vec![],
                    }),
                ],
            }
        )]
    );

    let share_token_address = pool.query_share_token_address();
    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 1_000_001i128,
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 999_999i128,
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: 1_000_000i128,
            },
        }
    );
    assert_eq!(token1.balance(&user1), 999); // -1 from the swap
    assert_eq!(token2.balance(&user1), 1001); // 1 from the swap

    // false means selling B token
    // this time 100 units
    let output_amount = pool.swap(&user1, &token2.address, &1_000, &None, &Some(spread));
    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 1_000_001 - 1_000, // previous balance minus 1_000
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 999_999 + 1000,
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 1_000_000i128, // this has not changed
            },
        }
    );
    assert_eq!(output_amount, 1000);
    assert_eq!(token1.balance(&user1), 1999); // 999 + 1_000 as a result of swap
    assert_eq!(token2.balance(&user1), 1001 - 1000); // user1 sold 1k of token B on second swap
}

#[test]
fn swap_with_high_fee() {
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

    let swap_fees = 1_000i64; // 10% bps
    let fee_recipient = Address::random(&env);
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        fee_recipient.clone(),
        None,
        None,
    );

    let initial_liquidity = 1_000_000i128;

    token1.mint(&user1, &(initial_liquidity + 100_000));
    token2.mint(&user1, &initial_liquidity);
    pool.provide_liquidity(
        &user1,
        &Some(initial_liquidity),
        &Some(initial_liquidity),
        &Some(initial_liquidity),
        &Some(initial_liquidity),
        &None,
    );

    let spread = 1_000; // 10% maximum spread allowed

    // let's swap 100_000 units of Token 1 in 1:1 pool with 10% protocol fee
    pool.swap(&user1, &token1.address, &100_000, &None, &Some(spread));

    // This is XYK LP with constant product formula
    // Y_new = (X_in * Y_old) / (X_in + X_old)
    // Y_new = (100_000 * 1_000_000) / (100_000 + 1_000_000)
    // Y_new = 90_909.0909
    let output_amount = 90_910i128; // rounding
    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: initial_liquidity + 100_000i128,
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: initial_liquidity - output_amount,
            },
            asset_lp_share: Asset {
                address: pool.query_share_token_address(),
                amount: 1_000_000i128,
            },
        }
    );
    // 10% fees are deducted from the swap result and sent to fee recipient address
    let fees = Decimal::percent(10) * output_amount;
    assert_eq!(token2.balance(&user1), output_amount - fees);
    assert_eq!(token2.balance(&fee_recipient), fees);
}

#[test]
fn swap_simulation_even_pool() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let mut token1 = deploy_token_contract(&env, &Address::random(&env));
    let mut token2 = deploy_token_contract(&env, &Address::random(&env));
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let swap_fees = 1_000i64; // 10% bps
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        Address::random(&env),
        None,
        None,
    );

    let initial_liquidity = 1_000_000i128;
    let user1 = Address::random(&env);
    token1.mint(&user1, &initial_liquidity);
    token2.mint(&user1, &initial_liquidity);
    pool.provide_liquidity(
        &user1,
        &Some(initial_liquidity),
        &Some(initial_liquidity),
        &Some(initial_liquidity),
        &Some(initial_liquidity),
        &None,
    );

    // let's simulate swap 100_000 units of Token 1 in 1:1 pool with 10% protocol fee
    let offer_amount = 100_000i128;
    let result = pool.simulate_swap(&token1.address, &offer_amount);

    // This is XYK LP with constant product formula
    // Y_new = (X_in * Y_old) / (X_in + X_old)
    // Y_new = (100_000 * 1_000_000) / (100_000 + 1_000_000)
    // Y_new = 90_909.0909
    let output_amount = 90_910i128; // rounding
    let fees = Decimal::percent(10) * output_amount;
    assert_eq!(
        result,
        SimulateSwapResponse {
            ask_amount: output_amount - fees,
            // spread_amount: Decimal::from_ratio(100_000, 1_000_000) * output_amount, // since it's 10% of the pool
            spread_amount: 9090, // rounding error, one less then ^
            commission_amount: fees,
            total_return: offer_amount,
        }
    );

    // now reverse swap querie should give us similar results
    // User wants to buy output_amount of tokens
    let result = pool.simulate_reverse_swap(&token1.address, &(output_amount - fees));
    assert_eq!(
        result,
        SimulateReverseSwapResponse {
            // offer_amount,
            offer_amount: 99_999i128, // rounding error
            // spread_amount: Decimal::from_ratio(100_000, 1_000_000) * output_amount, // since it's 10% of the pool
            spread_amount: 9090, // rounding error, one less then ^
            // commission_amount: fees,
            commission_amount: 9090,
        }
    );

    // false indicates selling the other asset - transaction goes the same
    let result = pool.simulate_swap(&token2.address, &offer_amount);
    assert_eq!(
        result,
        SimulateSwapResponse {
            ask_amount: output_amount - fees,
            spread_amount: 9090, // spread amount is basically 10%, since it's basically 10% of the
            // first token
            commission_amount: fees,
            total_return: offer_amount,
        }
    );

    // again reverse swap should show the same values
    let result = pool.simulate_reverse_swap(&token2.address, &(output_amount - fees));
    assert_eq!(
        result,
        SimulateReverseSwapResponse {
            // offer_amount,
            offer_amount: 99_999i128, // rounding error
            // spread_amount: Decimal::from_ratio(100_000, 1_000_000) * output_amount, // since it's 10% of the pool
            spread_amount: 9090, // rounding error, one less then ^
            // commission_amount: fees,
            commission_amount: 9090,
        }
    );
}

#[test]
fn swap_simulation_one_third_pool() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let mut token1 = deploy_token_contract(&env, &Address::random(&env));
    let mut token2 = deploy_token_contract(&env, &Address::random(&env));
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let swap_fees = 500i64; // 5% bps
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        Address::random(&env),
        None,
        None,
    );

    let initial_liquidity = 1_000_000i128;
    let user1 = Address::random(&env);
    token1.mint(&user1, &initial_liquidity);
    token2.mint(&user1, &(3 * initial_liquidity));
    pool.provide_liquidity(
        &user1,
        &Some(initial_liquidity),
        &Some(initial_liquidity),
        &Some(3 * initial_liquidity),
        &Some(3 * initial_liquidity),
        &None,
    );

    // let's simulate swap 100_000 units of Token 1 in 1:3 pool with 5% protocol fee
    let offer_amount = 100_000i128;
    let result = pool.simulate_swap(&token1.address, &offer_amount);

    // This is XYK LP with constant product formula
    // Y_new = (X_in * Y_old) / (X_in + X_old)
    // Y_new = (100_000 * 3_000_000) / (100_000 + 1_000_000)
    // Y_new = 272_727.27
    let output_amount = 272_728i128; // rounding
    let fees = Decimal::percent(5) * output_amount;
    assert_eq!(
        result,
        SimulateSwapResponse {
            ask_amount: output_amount - fees,
            spread_amount: Decimal::from_ratio(offer_amount, 1_000_000) * output_amount, // since it's 10% of the pool
            commission_amount: fees,
            total_return: 300_000,
        }
    );

    // now reverse swap querie should give us similar results
    // User wants to buy output_amount of tokens
    let result = pool.simulate_reverse_swap(&token1.address, &(output_amount - fees));
    assert_eq!(
        result,
        SimulateReverseSwapResponse {
            offer_amount,
            spread_amount: Decimal::from_ratio(offer_amount, 1_000_000) * output_amount, // since it's 10% of the pool
            commission_amount: fees,
        }
    );

    // false indicates selling the other asset - transaction goes the same
    let result = pool.simulate_swap(&token2.address, &100_000);
    // Y_new = (X_in * Y_old) / (X_in + X_old)
    // Y_new = (100_000 * 1_000_000) / (100_000 + 3_000_000)
    // Y_new = 32_258.06
    let output_amount = 32_259i128; // rounding
    let fees = Decimal::percent(5) * output_amount;
    assert_eq!(
        result,
        SimulateSwapResponse {
            ask_amount: output_amount - fees,
            // spread_amount: Decimal::from_ratio(100_000i128, 3_000_000i128) * output_amount,
            spread_amount: 1074, // rounding error, one less then ^
            commission_amount: fees,
            total_return: 33_333,
        }
    );

    // reverse should mirror it
    let result = pool.simulate_reverse_swap(&token2.address, &(output_amount - fees));
    assert_eq!(
        result,
        SimulateReverseSwapResponse {
            // offer_amount,
            offer_amount: 100_002i128, // rounding error
            spread_amount: Decimal::from_ratio(offer_amount, 3_000_000) * output_amount, // since it's 10% of the pool
            commission_amount: fees,
        }
    );
}
