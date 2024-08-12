extern crate std;
use pretty_assertions::assert_eq;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Ledger},
    Address, Env, IntoVal,
};
use test_case::test_case;

use super::setup::{deploy_liquidity_pool_contract, deploy_token_contract};
use crate::storage::{Asset, PoolResponse, SimulateReverseSwapResponse, SimulateSwapResponse};
use soroban_decimal::Decimal;

#[test]
fn simple_swap() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
    let stake_owner = Address::generate(&env);

    let swap_fees = 0i64;
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    let liquidity_amount = 100_000_000_000_000_i128; // 10 million with 7 decimal places
    token1.mint(&user1, &(liquidity_amount + 10_000_000_000_000)); // 11,000,000.0000000
    token2.mint(&user1, &(liquidity_amount + 10_000_000_000_000)); // 11,000,000.0000000

    pool.provide_liquidity(
        &user1,
        &Some(liquidity_amount),
        &Some(liquidity_amount),
        &Some(liquidity_amount),
        &Some(liquidity_amount),
        &None,
        &None::<u64>,
    );

    // Swapping 100,000 tokens with 7 decimal places
    let swap_amount = 1_000_000_000_000_i128; // 100,000.0000000 with 7 decimals

    // Execute the swap
    let output_amount = pool.swap(
        &user1,
        &token1.address,
        &swap_amount,
        &None,
        &Some(100), // 1% spread as allowed
        &None::<u64>,
        &None,
    );
    assert_eq!(
        env.auths(),
        [(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    pool.address.clone(),
                    symbol_short!("swap"),
                    (
                        &user1,
                        // FIXM: Disable Referral struct
                        // None::<Referral>,
                        token1.address.clone(),
                        1_000_000_000_000_i128,
                        None::<i64>,
                        Some(100i64),
                        None::<u64>,
                        None::<i64>
                    )
                        .into_val(&env)
                )),
                sub_invocations: std::vec![
                    (AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            token1.address.clone(),
                            symbol_short!("transfer"),
                            (&user1, &pool.address, 1_000_000_000_000_i128).into_val(&env)
                        )),
                        sub_invocations: std::vec![],
                    }),
                ],
            }
        )]
    );

    // Query pool info after swap
    let share_token_address = pool.query_share_token_address();
    let result = pool.query_pool_info();

    // Spread amount computed by the DEX contract during the swap
    let spread_amount = 990_0990099i128;

    // Corrected assertion based on the results from the swap
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: liquidity_amount + swap_amount,
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: liquidity_amount - swap_amount + spread_amount
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: liquidity_amount, // Liquidity pool share remains unchanged
            },
            stake_address: result.stake_address.clone(),
        }
    );

    assert_eq!(token1.balance(&user1), 10_000_000_000_000 - swap_amount); // 11,000,000.0000000 - 100,000.0000000
    assert_eq!(token2.balance(&user1), 10_000_000_000_000 + output_amount); // Reflect the swap return after spread

    // This time swapping 100,000 tokens of token2
    let output_amount_2 = pool.swap(
        &user1,
        &token2.address,
        &swap_amount,
        &None,
        &Some(200), // 2% spread as allowed
        &None::<u64>,
        &None,
    );
    let result = pool.query_pool_info();

    // Corrected assertion based on the second swap
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 99_990_099_990_099, // Reflecting the state after the second swap, with spread
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 100_009_900_990_099, // Reflecting the state after the second swap, with spread
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: liquidity_amount, // Liquidity pool share remains unchanged
            },
            stake_address: result.stake_address.clone(),
        }
    );

    assert_eq!(output_amount_2, 1_009_900_009_901); // Expected output after accounting for the spread
    assert_eq!(token1.balance(&user1), 10_009_900_009_901); // Reflecting final balances after swaps
    assert_eq!(token2.balance(&user1), 9_990_099_009_901); // Reflecting final balances after swaps
}

#[test]
fn simple_swap_with_preferred_pool_fee() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
    let stake_owner = Address::generate(&env);

    // the swap fee is set at %3
    let swap_fees = 300;
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
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
        &None::<u64>,
    );

    // selling just one token with 1% max spread allowed
    let spread = 100i64; // 1% maximum spread allowed
    pool.swap(
        &user1,
        // FIXM: Disable Referral struct
        // &None::<Referral>,
        &token1.address,
        &1,
        &None,
        &Some(spread),
        &None::<u64>,
        //user would swap with a pool fee at maximum %5
        &Some(500),
    );
    assert_eq!(
        env.auths(),
        [(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    pool.address.clone(),
                    symbol_short!("swap"),
                    (
                        &user1,
                        // FIXM: Disable Referral struct
                        // None::<Referral>,
                        token1.address.clone(),
                        1_i128,
                        None::<i64>,
                        spread,
                        None::<u64>,
                        Some(500i64)
                    )
                        .into_val(&env)
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
            stake_address: result.clone().stake_address,
        }
    );
    assert_eq!(token1.balance(&user1), 999); // -1 from the swap
    assert_eq!(token2.balance(&user1), 1001); // 1 from the swap

    // this time 100 units
    let output_amount = pool.swap(
        &user1,
        // FIXM: Disable Referral struct
        // &None::<Referral>,
        &token2.address,
        &1_000,
        &None,
        &Some(spread),
        &None::<u64>,
        &None,
    );
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
            stake_address: result.clone().stake_address,
        }
    );
    assert_eq!(output_amount, 970);
    assert_eq!(token1.balance(&user1), 1969); // 969 + 1_000 as a result of swap
    assert_eq!(token2.balance(&user1), 1001 - 1000); // user1 sold 1k of token B on second swap
}

#[test]
#[should_panic(expected = "Pool: do_swap: User agrees to swap at a lower percentage.")]
fn simple_swap_should_panic_when_user_accepted_fee_is_less_than_pool_fee() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
    let stake_owner = Address::generate(&env);

    // the swap fee is set at %5
    let swap_fees = 500;
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
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
        &None::<u64>,
    );

    let spread = 100i64;
    pool.swap(
        &user1,
        // FIXM: Disable Referral struct
        // &None::<Referral>,
        &token1.address,
        &1,
        &None,
        &Some(spread),
        &None::<u64>,
        //user would swap with a pool fee at maximum %1
        &Some(100),
    );
    assert_eq!(
        env.auths(),
        [(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    pool.address.clone(),
                    symbol_short!("swap"),
                    (
                        &user1,
                        // FIXM: Disable Referral struct
                        // None::<Referral>,
                        token1.address.clone(),
                        1_i128,
                        None::<i64>,
                        spread,
                        None::<u64>
                    )
                        .into_val(&env)
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
            stake_address: result.clone().stake_address,
        }
    );
    assert_eq!(token1.balance(&user1), 999); // -1 from the swap
    assert_eq!(token2.balance(&user1), 1001); // 1 from the swap

    pool.swap(
        &user1,
        // FIXM: Disable Referral struct
        // &None::<Referral>,
        &token2.address,
        &1_000,
        &None,
        &Some(spread),
        &None::<u64>,
        &None,
    );
}

// FIXM: Disable Referral struct
#[ignore]
#[test]
fn simple_swap_with_referral_fee() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
    let stake_owner = Address::generate(&env);
    let referral_addr = Address::generate(&env);

    let swap_fees = 0i64;
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
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
        &None::<u64>,
    );

    // selling just one token with 1% max spread allowed
    let spread = 100i64; // 1% maximum spread allowed

    // selling with 10% fee for the big guy
    // FIXM: Disable Referral struct
    // let referral = Referral {
    //     address: referral_addr.clone(),
    //     fee: 1_000,
    // };

    pool.swap(
        &user1,
        //     &Some(referral.clone()),
        &token1.address,
        &1,
        &None,
        &Some(spread),
        &None::<u64>,
        &None,
    );

    // zero referral fee because amount is too low
    assert_eq!(token1.balance(&referral_addr.clone()), 0);

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
            stake_address: result.clone().stake_address,
        }
    );

    assert_eq!(token1.balance(&user1), 999); // -1 from the swap
    assert_eq!(token2.balance(&user1), 1001); // 1 from the swap
    let output_amount = pool.swap(
        &user1,
        // FIXM: Disable Referral struct
        // &Some(referral),
        &token2.address,
        &1_000,
        &None,
        &Some(spread),
        &None::<u64>,
        &None,
    );
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
            stake_address: result.clone().stake_address,
        }
    );
    assert_eq!(output_amount, 900);
    assert_eq!(token1.balance(&user1), 1899); // 999 + 1_000 as a result of swap
                                              // FIXM: Disable Referral struct
                                              // assert_eq!(token1.balance(&referral_addr), 100);
    assert_eq!(token2.balance(&user1), 1001 - 1000); // user1 sold 1k of token B on second swap
}

// FIXM: Disable Referral struct
#[ignore]
#[test]
#[should_panic(expected = "Pool: Swap: Trying to swap with more than the allowed referral fee")]
fn test_swap_should_fail_when_referral_fee_is_larger_than_allowed() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
    let stake_owner = Address::generate(&env);
    let swap_fees = 0i64;
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
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
        &None::<u64>,
    );

    let spread = 100i64; // 1% maximum spread allowed

    // FIXM: Disable Referral struct
    // let referral = Referral {
    //     address: Address::random(&env),
    //     // in tests/setup.rs we hardcoded the max referral fee
    //     // to 5_000 bps (50%), here we try to set it to 10_000 bps (100%)
    //     fee: 10_000,
    // };

    pool.swap(
        &user1,
        //     &Some(referral),
        &token1.address,
        &1,
        &None,
        &Some(spread),
        &None::<u64>,
        &None,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn swap_should_panic_with_bad_max_spread() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
    let stake_owner = Address::generate(&env);
    let swap_fees = 0i64;
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    token1.mint(&user1, &1_001_000);
    token2.mint(&user1, &2_001_000);
    pool.provide_liquidity(
        &user1,
        &Some(5000),
        &None,
        &Some(2_000_000),
        &None,
        &None,
        &None::<u64>,
    );

    // selling just one token with 1% max spread allowed and 50 bps max spread
    // FIXM: Disable Referral struct
    pool.swap(
        &user1,
        &token1.address,
        &50,
        &None,
        &Some(50),
        &None::<u64>,
        &None,
    );
}

#[test]
fn swap_with_high_fee() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
    let stake_owner = Address::generate(&env);

    let swap_fees = 1_000i64; // 10% bps
    let fee_recipient = Address::generate(&env);
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        fee_recipient.clone(),
        None,
        None,
        stake_manager,
        stake_owner,
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
        &None::<u64>,
    );

    let spread = 1_000; // 10% maximum spread allowed

    // let's swap 100_000 units of Token 1 in 1:1 pool with 10% protocol fee
    pool.swap(
        &user1,
        // FIXM: Disable Referral struct
        // &None,
        &token1.address,
        &100_000,
        &None,
        &Some(spread),
        &None::<u64>,
        &None,
    );

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
            stake_address: result.clone().stake_address,
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

    let mut token1 = deploy_token_contract(&env, &Address::generate(&env));
    let mut token2 = deploy_token_contract(&env, &Address::generate(&env));
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let stake_manager = Address::generate(&env);
    let stake_owner = Address::generate(&env);
    let swap_fees = 1_000i64; // 10% bps
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        Address::generate(&env),
        None,
        None,
        stake_manager,
        stake_owner,
    );

    let initial_liquidity = 1_000_000i128;
    let user1 = Address::generate(&env);
    token1.mint(&user1, &initial_liquidity);
    token2.mint(&user1, &initial_liquidity);
    pool.provide_liquidity(
        &user1,
        &Some(initial_liquidity),
        &Some(initial_liquidity),
        &Some(initial_liquidity),
        &Some(initial_liquidity),
        &None,
        &None::<u64>,
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

    let mut token1 = deploy_token_contract(&env, &Address::generate(&env));
    let mut token2 = deploy_token_contract(&env, &Address::generate(&env));
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let stake_manager = Address::generate(&env);
    let stake_owner = Address::generate(&env);
    let swap_fees = 500i64; // 5% bps
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        Address::generate(&env),
        None,
        None,
        stake_manager,
        stake_owner,
    );

    let initial_liquidity = 1_000_000i128;
    let user1 = Address::generate(&env);
    token1.mint(&user1, &initial_liquidity);
    token2.mint(&user1, &(3 * initial_liquidity));
    pool.provide_liquidity(
        &user1,
        &Some(initial_liquidity),
        &Some(initial_liquidity),
        &Some(3 * initial_liquidity),
        &Some(3 * initial_liquidity),
        &None,
        &None::<u64>,
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
    let result = pool.simulate_reverse_swap(&token2.address, &(output_amount - fees));
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
    let result = pool.simulate_reverse_swap(&token1.address, &(output_amount - fees));
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

#[test_case(1_000i64, 99102002 ; "when fee is 10%")]
#[test_case(100, 9910200 ; "when fee is 1%")]
#[test_case(30, 2973060 ; "when fee is 0.3%")]
fn test_swap_fee_variants(swap_fees: i64, commission_fee: i128) {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let mut token1 = deploy_token_contract(&env, &Address::generate(&env));
    let mut token2 = deploy_token_contract(&env, &Address::generate(&env));
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let stake_manager = Address::generate(&env);
    let stake_owner = Address::generate(&env);
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        Address::generate(&env),
        10_000i64,
        10_000i64,
        stake_manager,
        stake_owner,
    );

    let initial_liquidity = 110_358_880_127; // taken from the current amount of tokens in pool
    let user1 = Address::generate(&env);
    token1.mint(&user1, &initial_liquidity);
    token2.mint(&user1, &initial_liquidity);
    pool.provide_liquidity(
        &user1,
        &Some(initial_liquidity),
        &Some(initial_liquidity),
        &Some(initial_liquidity),
        &Some(initial_liquidity),
        &None,
        &None::<u64>,
    );

    // simulating a swap with 1_000_000_000 units
    let offer_amount = 1_000_000_000i128;
    let result = pool.simulate_swap(&token1.address, &offer_amount);

    // XYK pool formula as follows
    // Y_new = (X_in * Y_old) / (X_in + X_old)
    // Y_new = (1_000_000_000 * 110358880127) / (1_000_000_000 + 110358880127)
    // Y_new = 991020024.637
    // Y_rnd = 991020025
    let output_amount = 991020025; // rounding

    let fees = Decimal::bps(swap_fees) * output_amount;

    assert_eq!(
        result,
        SimulateSwapResponse {
            ask_amount: output_amount - fees,
            spread_amount: 8979975,
            commission_amount: fees,
            total_return: 1000000000,
        }
    );

    // 991020025 is the request, so 10% of that should be what's on the left hand side
    assert_eq!(commission_fee, result.commission_amount);

    let result = pool.simulate_reverse_swap(&token1.address, &(output_amount - fees));
    let output_amount = 991020025i128;
    // let fees = Decimal::percent(fee_percentage) * output_amount;
    assert_eq!(
        result,
        SimulateReverseSwapResponse {
            offer_amount: 1000000000i128,
            spread_amount: Decimal::from_ratio(offer_amount, initial_liquidity) * output_amount,
            commission_amount: fees,
        }
    );
}

#[test_case(Some(-100))]
#[test_case(Some(600))]
#[test_case(Some(501))]
#[should_panic(expected = "max spread is out of bounds")]
fn test_v_phx_vul_021_should_panic_when_max_spread_invalid_range(max_spread: Option<i64>) {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let user1 = Address::generate(&env);
    let swap_fees = 0i64;
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        Some(500i64),
        Address::generate(&env),
        Address::generate(&env),
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
        &None::<u64>,
    );

    pool.swap(
        &user1,
        // FIXM: Disable Referral struct
        // &None::<Referral>,
        &token1.address,
        &1,
        &None,
        &max_spread,
        &None::<u64>,
        &None,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #16)")]
fn test_v_phx_vul_017_should_panic_when_swapping_non_existing_token_in_pool() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    let bad_token = deploy_token_contract(&env, &Address::generate(&env));

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let user1 = Address::generate(&env);
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        Address::generate(&env),
        Address::generate(&env),
    );
    // Swap fails because we provide incorrect token as offer token.
    pool.swap(
        &user1,
        // FIXM: Disable Referral struct
        // &None::<Referral>,
        &bad_token.address,
        &1,
        &None,
        &Some(100),
        &None::<u64>,
        &None,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #16)")]
fn test_v_phx_vul_017_should_panic_when_simulating_swap_for_non_existing_token_in_pool() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    let bad_token = deploy_token_contract(&env, &Address::generate(&env));

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        Address::generate(&env),
        Address::generate(&env),
    );
    // Simulate swap fails because we provide incorrect token as offer token.
    pool.simulate_swap(
        // FIXM: Disable Referral struct
        // &None::<Referral>,
        &bad_token.address,
        &1,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #16)")]
fn test_v_phx_vul_017_should_panic_when_simulating_reverse_swap_for_non_existing_token_in_pool() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    let bad_token = deploy_token_contract(&env, &Address::generate(&env));

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        Address::generate(&env),
        Address::generate(&env),
    );
    // Simulate swap fails because we provide incorrect token as offer token.
    pool.simulate_reverse_swap(
        // FIXM: Disable Referral struct
        // &None::<Referral>,
        &bad_token.address,
        &1,
    );
}

#[test]
fn test_should_swap_with_valid_ask_asset_min_amount() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let mut token1 = deploy_token_contract(&env, &Address::generate(&env));
    let mut token2 = deploy_token_contract(&env, &Address::generate(&env));

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
    let user = Address::generate(&env);

    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        Address::generate(&env),
        Address::generate(&env),
    );

    token1.mint(&user, &1_050_000);
    token2.mint(&user, &1_050_000);
    // 1:1 pool with large liquidity
    pool.provide_liquidity(
        &user,
        &Some(1_000_000),
        &Some(1_000_000),
        &Some(1_000_000),
        &Some(1_000_000),
        &None,
        &None::<u64>,
    );

    pool.swap(
        &user,
        &token1.address,
        &10,
        &Some(10),
        &None::<i64>,
        &None::<u64>,
        &None,
    );
    assert_eq!(token1.balance(&user), 49_990);
    assert_eq!(token2.balance(&user), 50_010);

    pool.swap(
        &user,
        &token2.address,
        &5_000i128,
        &Some(4_900i128),
        &Some(500i64),
        &None::<u64>,
        &None,
    );

    assert_eq!(token1.balance(&user), 54_966);
    assert_eq!(token2.balance(&user), 45_010);
}

#[test]
#[should_panic(expected = "Pool: do_swap: Return amount is smaller then expected minimum amount")]
fn test_should_fail_when_invalid_ask_asset_min_amount() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let mut token1 = deploy_token_contract(&env, &Address::generate(&env));
    let mut token2 = deploy_token_contract(&env, &Address::generate(&env));
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
    let user = Address::generate(&env);
    let stake_manager = Address::generate(&env);
    let stake_owner = Address::generate(&env);

    let swap_fees = 0i64;
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    token1.mint(&user, &1_001_000);
    token2.mint(&user, &1_001_000);
    pool.provide_liquidity(
        &user,
        &Some(1_000_000),
        &Some(1_000_000),
        &Some(1_000_000),
        &Some(1_000_000),
        &None,
        &None::<u64>,
    );

    let spread = 100i64; // 1% maximum spread allowed
    pool.swap(
        &user,
        &token1.address,
        &1,
        &Some(10),
        &Some(spread),
        &None::<u64>,
        &None,
    );
}

#[test]
fn simple_swap_with_deadline_success() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
    let stake_owner = Address::generate(&env);

    let swap_fees = 0i64;
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    token1.mint(&user1, &1_001_000);
    token2.mint(&user1, &1_001_000);

    env.ledger().with_mut(|li| li.timestamp = 49);
    pool.provide_liquidity(
        &user1,
        &Some(1_000_000),
        &Some(1_000_000),
        &Some(1_000_000),
        &Some(1_000_000),
        &None,
        &Some(50u64),
    );

    // selling just one token with 1% max spread allowed
    let spread = 100i64; // 1% maximum spread allowed
    env.ledger().with_mut(|li| li.timestamp = 99);
    // we set the deadline to be at latest 100 and we execute swap at 99
    pool.swap(
        &user1,
        // FIXM: Disable Referral struct
        // &None::<Referral>,
        &token1.address,
        &1,
        &None,
        &Some(spread),
        &Some(100u64),
        &None,
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
            stake_address: result.clone().stake_address,
        }
    );
    assert_eq!(token1.balance(&user1), 999); // -1 from the swap
    assert_eq!(token2.balance(&user1), 1001); // 1 from the swap

    env.ledger().with_mut(|li| li.timestamp = 149);
    let output_amount = pool.swap(
        &user1,
        // FIXM: Disable Referral struct
        // &None::<Referral>,
        &token2.address,
        &1_000,
        &None,
        &Some(spread),
        &Some(150),
        &None,
    );
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
            stake_address: result.clone().stake_address,
        }
    );
    assert_eq!(output_amount, 1000);
    assert_eq!(token1.balance(&user1), 1999); // 999 + 1_000 as a result of swap
    assert_eq!(token2.balance(&user1), 1001 - 1000); // user1 sold 1k of token B on second swap
}

#[test]
#[should_panic(expected = "Pool: Swap: Transaction executed after deadline!")]
fn simple_swap_with_should_fail_when_after_the_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
    let stake_owner = Address::generate(&env);

    let swap_fees = 0i64;
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
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
        &None::<u64>,
    );

    // selling just one token with 1% max spread allowed
    let spread = 100i64; // 1% maximum spread allowed
    env.ledger().with_mut(|li| li.timestamp = 100);
    // this will panic, because our deadline is before the current timestamp
    pool.swap(
        &user1,
        // FIXM: Disable Referral struct
        // &None::<Referral>,
        &token1.address,
        &1,
        &None,
        &Some(spread),
        &Some(99u64),
        &None,
    );
}

#[test]
fn simple_swap_with_biggest_possible_decimal_precision() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
    let stake_owner = Address::generate(&env);

    let swap_fees = 0i64;
    let pool = deploy_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        // allowing the maximum spread of %100
        Some(10_000),
        stake_manager,
        stake_owner,
    );

    token1.mint(&user1, &999_000_000_000_001_000);
    token2.mint(&user1, &999_000_000_000_001_000);
    pool.provide_liquidity(
        &user1,
        &Some(450_000_000_000_000_000),
        &Some(405_000_000_000_000_000),
        &Some(450_000_000_000_000_000),
        &Some(405_000_000_000_000_000),
        &None,
        &None::<u64>,
    );

    // 50% spread
    let spread = 5000i64;
    pool.swap(
        &user1,
        // FIXM: Disable Referral struct
        // &None::<Referral>,
        &token1.address,
        &200_000_000_000_000_000,
        &None,
        &Some(spread),
        &None::<u64>,
        &None,
    );

    let share_token_address = pool.query_share_token_address();
    let result = pool.query_pool_info();

    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 650_000_000_000_000_000i128,
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 311_538_461_538_461_538i128,
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: 450_000_000_000_000_000i128,
            },
            stake_address: result.clone().stake_address,
        }
    );

    assert_eq!(token1.balance(&user1), 349_000_000_000_001_000i128);
    assert_eq!(token2.balance(&user1), 687_461_538_461_539_462i128);
}
