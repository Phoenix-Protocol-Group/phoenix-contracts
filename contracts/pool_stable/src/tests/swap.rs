extern crate std;

use soroban_sdk::testutils::{AuthorizedFunction, AuthorizedInvocation, Ledger};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, IntoVal};

use super::setup::{deploy_stable_liquidity_pool_contract, deploy_token_contract};
use crate::storage::{Asset, PoolResponse, SimulateReverseSwapResponse, SimulateSwapResponse};
use soroban_decimal::Decimal;

#[test]
fn simple_swap() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
    let user1 = Address::generate(&env);
    let swap_fees = 0i64;
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        manager,
        factory,
        None,
    );

    token1.mint(&user1, &1_001_000);
    token2.mint(&user1, &1_001_000);
    pool.provide_liquidity(
        &user1,
        &1_000_000,
        &1_000_000,
        &None,
        &None::<u64>,
        &None::<u128>,
    );

    // true means "selling A token"
    // selling just one token with 1% max spread allowed
    let spread = 100i64; // 1% maximum spread allowed
    pool.swap(
        &user1,
        &token1.address,
        &1,
        &None,
        &Some(spread),
        &None::<u64>,
        &Some(150),
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
                        token1.address.clone(),
                        1_i128,
                        None::<i64>,
                        spread,
                        None::<u64>,
                        Some(150i64),
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
                amount: 1999000i128,
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );
    assert_eq!(token1.balance(&user1), 999); // -1 from the swap
    assert_eq!(token2.balance(&user1), 1001); // 1 from the swap

    // false means selling B token
    // this time 100 units
    let output_amount = pool.swap(
        &user1,
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
                amount: 1999000i128, // this has not changed
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );
    assert_eq!(output_amount, 1000);
    assert_eq!(token1.balance(&user1), 1999); // 999 + 1_000 as a result of swap
    assert_eq!(token2.balance(&user1), 1001 - 1000); // user1 sold 1k of token B on second swap
}

#[test]
fn simple_swap_millions_liquidity_swapping_half_milion_no_fee() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
    let user1 = Address::generate(&env);
    let swap_fees = 0i64;
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        manager,
        factory,
        None,
    );

    // minting 100 million tokens to user1
    token1.mint(&user1, &1_000_000_000_000_000);
    token2.mint(&user1, &1_000_000_000_000_000);
    // providing 10 million tokens as liquidity from both token1 and token2
    pool.provide_liquidity(
        &user1,
        &100_000_000_000_000,
        &100_000_000_000_000,
        &None,
        &None::<u64>,
        &None::<u128>,
    );
    // at this point, the pool holds:
    // token1: 100_000_000_000_000
    // token2: 100_000_000_000_000
    // ttal LP shares issued: 200_000_000_000_000

    // selling 500_000 tokens with 10% max spread allowed
    let spread = 1_000i64; // 10% maximum spread allowed
    pool.swap(
        &user1,
        &token1.address,
        &500_000,
        &None,
        &Some(spread),
        &None::<u64>,
        &Some(150),
    );
    // after the swap:
    // token1 in the pool increases by 500_000: 100_000_000_000_000 + 500_000 = 100_000_000_500_000
    // token2 in the pool decreases by ~500_000 (depending on swap calculation): 100_000_000_000_000 - 500_000 = 99_999_999_500_000
    // total LP shares remain unchanged at 199_999_999_999_000 (no liquidity added/removed)

    let share_token_address = pool.query_share_token_address();
    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 100000000500000i128,
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 99999999500000i128,
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: 199999999999000i128,
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );

    // user's token balances after the first swap:
    // token1 decreases by 500_000: 1_000_000_000_000_000 - 500_000 = 899_999_999_500_000
    // token2 increases by ~500_000: 1_000_000_000_000_000 + 500_000 = 900_000_000_500_000
    assert_eq!(token1.balance(&user1), 899999999500000);
    assert_eq!(token2.balance(&user1), 900000000500000);

    // this time 100_000 tokens
    let output_amount = pool.swap(
        &user1,
        &token2.address,
        &100_000,
        &None,
        &Some(spread),
        &None::<u64>,
        &None,
    );

    // after the second swap:
    // token1 in the pool decreases by ~100_000: 100_000_000_500_000 - 100_000 = 100_000_000_400_000
    // token2 in the pool increases by 100_000: 99_999_999_500_000 + 100_000 = 99_999_999_600_000
    // total LP shares remain unchanged at 199_999_999_999_000 (no liquidity added/removed)
    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 100000000400000,
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 99999999600000,
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 199999999999000
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );

    // user's token balances after the second swap:
    // token1 increases by ~100_000: 899_999_999_500_000 + 100_000 = 899_999_999_600_000
    // token2 decreases by 100_000: 900_000_000_500_000 - 100_000 = 900_000_000_400_000
    assert_eq!(output_amount, 100_000);
    assert_eq!(token1.balance(&user1), 899999999600000);
    assert_eq!(token2.balance(&user1), 900000000400000);
}

#[test]
fn simple_swap_ten_thousand_tokens() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
    let user1 = Address::generate(&env);
    let swap_fees = 0i64;
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        manager,
        factory,
        None,
    );

    // minting 100 million tokens to user1
    token1.mint(&user1, &1_000_000_000_000_000);
    token2.mint(&user1, &1_000_000_000_000_000);

    // providing 10 million tokens as liquidity
    pool.provide_liquidity(
        &user1,
        &100_000_000_000_000,
        &100_000_000_000_000,
        &None,
        &None::<u64>,
        &None::<u128>,
    );

    // selling 10,000 tokens with 5% max spread allowed
    let spread = 500i64;
    pool.swap(
        &user1,
        &token1.address,
        &1_000_000_000, // 10_000 tokens with 7 decimal precision
        &None,
        &Some(spread),
        &None::<u64>,
        &Some(150),
    );

    let share_token_address = pool.query_share_token_address();
    let result = pool.query_pool_info();

    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 100_001_000_000_000i128,
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 99_999_000_001_428_i128,
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: 199_999_999_999_000i128,
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );

    assert_eq!(token1.balance(&user1), 899_999_000_000_000);
    assert_eq!(token2.balance(&user1), 900_000_999_998_572);
}

#[test]
fn simple_swap_millions_liquidity_swapping_half_milion_high_fee() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
    let user1 = Address::generate(&env);

    // we set a 10% swap fee (1000 basis points)
    let swap_fees = 1_000i64; // 10%
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        manager,
        factory,
        None,
    );

    // minting 1_000_000_000_000_000 "units"
    // equals 100_000_000 tokens
    token1.mint(&user1, &1_000_000_000_000_000);
    token2.mint(&user1, &1_000_000_000_000_000);

    // providing 100_000_000_000_000 "units" of liquidity
    // equals 10_000_000 tokens
    pool.provide_liquidity(
        &user1,
        &100_000_000_000_000,
        &100_000_000_000_000,
        &None,
        &None::<u64>,
        &None::<u128>,
    );

    // at this point, the pool holds 100_000_000_000_000 "units"
    // equals 10_000_000 tokens
    // and total LP shares = 200_000_000_000_000 "units" => 20_000_000 LP shares.
    //
    // user1 is left with (100-000_000 - 10_000_000) = 90_000_000 tokens (i.e. 900_000_000_000_000 "units").

    // user sells 10_000_000_000 "units" of token1
    // equals 1_000 tokens, with a 10% max spread allowed.
    // Because the pool charges a 10% fee, user gets ~90% of the ideal return in token2.
    let spread = 1_000i64;
    pool.swap(
        &user1,
        &token1.address,
        &10_000_000_000,
        &None,
        &Some(spread),
        &None::<u64>,
        &None,
    );
    // after this swap:
    // token1 in the pool increases by 1_000 tokens (10_000_000,000 "units")
    // token2 in the pool decreases by slightly less than 1_000 tokens
    // total LP shares remain the same (no liquidity added/removed)

    let share_token_address = pool.query_share_token_address();
    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 100_010_000_000_000_i128,
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 99_990_000_142_855_i128,
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: 199_999_999_999_000_i128,
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );

    // user's balances after the first swap:
    // token1 decreases by ~1_000 tokens.
    // token2 increases by ~900 tokens net (10% fee).
    assert_eq!(token1.balance(&user1), 899_990_000_000_000);
    assert_eq!(token2.balance(&user1), 900_008_999_871_431);

    // Now user sells 1_000_000_000 "units" of token2 => 100 tokens,
    let output_amount = pool.swap(
        &user1,
        &token2.address,
        &1_000_000_000,
        &None,
        &Some(spread),
        &None::<u64>,
        &None,
    );

    // after this second swap:
    // token1 in the pool decreases by ~90 tokens (10% fee)
    // token2 in the pool increases by ~100 tokens
    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 100_009_000_115_713,
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 99_991_000_142_855,
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 199_999_999_999_000
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );

    assert_eq!(output_amount, 899_895_859);

    // final balances after the second swap:
    // token1: 899_990_000_000_000 "units" + ~90 tokens in "units"
    // token2: 900_008_999_871_431 "units" - 100 tokens in "units"
    assert_eq!(token1.balance(&user1), 899_990_899_895_859);
    assert_eq!(token2.balance(&user1), 900_007_999_871_431);
}

#[test]
fn swap_with_high_fee() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
    let user1 = Address::generate(&env);

    let swap_fees = 1_000i64; // 10% bps
    let fee_recipient = Address::generate(&env);
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        fee_recipient.clone(),
        None,
        None,
        manager,
        factory,
        None,
    );

    let initial_liquidity = 1_000_000i128;

    token1.mint(&user1, &(initial_liquidity + 100_000));
    token2.mint(&user1, &initial_liquidity);
    pool.provide_liquidity(
        &user1,
        &initial_liquidity,
        &initial_liquidity,
        &None,
        &None::<u64>,
        &None::<u128>,
    );

    let spread = 1_000; // 10% maximum spread allowed

    // let's swap 100_000 units of Token 1 in 1:1 pool with 10% protocol fee
    pool.swap(
        &user1,
        &token1.address,
        &100_000,
        &None,
        &Some(spread),
        &None::<u64>,
        &None,
    );

    // This is Stable swap LP with constant product formula
    let output_amount = 98_582i128; // rounding
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
                amount: 1999000i128,
            },
            stake_address: pool.query_stake_contract_address(),
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
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
    let user1 = Address::generate(&env);

    let swap_fees = 1_000i64; // 10% bps
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        manager,
        factory,
        None,
    );

    let initial_liquidity = 1_000_000i128;
    token1.mint(&user1, &initial_liquidity);
    token2.mint(&user1, &initial_liquidity);
    pool.provide_liquidity(
        &user1,
        &initial_liquidity,
        &initial_liquidity,
        &None,
        &None::<u64>,
        &None::<u128>,
    );

    // let's simulate swap 100_000 units of Token 1 in 1:1 pool with 10% protocol fee
    let offer_amount = 100_000i128;
    let result = pool.simulate_swap(&token1.address, &offer_amount);

    // This is Stable Swap LP with constant product formula
    let output_amount = 98_582i128;
    let fees = Decimal::percent(10) * output_amount;
    assert_eq!(
        result,
        SimulateSwapResponse {
            ask_amount: output_amount - fees,
            // spread_amount: any difference between the offer and return amounts since it's 1:1
            spread_amount: offer_amount - output_amount,
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
            offer_amount: 100_000i128,
            // spread_amount: any difference between the offer and return amounts since it's 1:1
            spread_amount: offer_amount + fees - output_amount,
            // spread_amount: 11276,
            // commission_amount: fees,
            commission_amount: 9858,
        }
    );

    // false indicates selling the other asset - transaction goes the same
    let result = pool.simulate_swap(&token2.address, &offer_amount);
    assert_eq!(
        result,
        SimulateSwapResponse {
            ask_amount: output_amount - fees,
            spread_amount: offer_amount - output_amount,
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
            offer_amount: 100_000i128,
            spread_amount: offer_amount + fees - output_amount,
            // commission_amount: fees,
            commission_amount: fees,
        }
    );
}

#[test]
fn simple_swap_with_deadline_should_work() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
    let user1 = Address::generate(&env);
    let swap_fees = 0i64;
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        manager,
        factory,
        None,
    );

    token1.mint(&user1, &1_001_000);
    token2.mint(&user1, &1_001_000);
    env.ledger().with_mut(|li| li.timestamp = 49);
    pool.provide_liquidity(
        &user1,
        &1_000_000,
        &1_000_000,
        &None,
        &Some(50),
        &None::<u128>,
    );

    let spread = 100i64;
    // making the swap at the final moment
    env.ledger().with_mut(|li| li.timestamp = 99);
    pool.swap(
        &user1,
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
                amount: 1999000i128,
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );
    assert_eq!(token1.balance(&user1), 999); // -1 from the swap
    assert_eq!(token2.balance(&user1), 1001); // 1 from the swap

    // false means selling B token
    // this time 100 units
    env.ledger().with_mut(|li| li.timestamp = 149);
    let output_amount = pool.swap(
        &user1,
        &token2.address,
        &1_000,
        &None,
        &Some(spread),
        &Some(150u64),
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
                amount: 1999000i128, // this has not changed
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );
    assert_eq!(output_amount, 1000);
    assert_eq!(token1.balance(&user1), 1999); // 999 + 1_000 as a result of swap
    assert_eq!(token2.balance(&user1), 1001 - 1000); // user1 sold 1k of token B on second swap
}

#[test]
#[should_panic(expected = "Pool Stable: Swap: Transaction executed after deadline!")]
fn simple_swap_should_panic_after_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
    let user1 = Address::generate(&env);
    let swap_fees = 0i64;
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        manager,
        factory,
        None,
    );

    token1.mint(&user1, &1_001_000);
    token2.mint(&user1, &1_001_000);
    pool.provide_liquidity(
        &user1,
        &1_000_000,
        &1_000_000,
        &None,
        &None::<u64>,
        &None::<u128>,
    );

    // true means "selling A token"
    // selling just one token with 1% max spread allowed
    let spread = 100i64; // 1% maximum spread allowed
                         // making the swap after the deadline
    env.ledger().with_mut(|li| li.timestamp = 100);
    pool.swap(
        &user1,
        &token1.address,
        &1,
        &None,
        &Some(spread),
        &Some(99u64),
        &None,
    );
}

#[test]
#[should_panic(expected = "Pool: do_swap: User agrees to swap at a lower percentage.")]
fn simple_swap_with_low_user_fee_should_panic() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
    let user1 = Address::generate(&env);
    let swap_fees = 100; //swap fee is %1
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        swap_fees,
        None,
        None,
        None,
        manager,
        factory,
        None,
    );

    token1.mint(&user1, &1_001_000);
    token2.mint(&user1, &1_001_000);
    pool.provide_liquidity(&user1, &1_000_000, &1_000_000, &None, &None::<u64>, &None);

    let spread = 100i64; // 1% maximum spread allowed
    pool.swap(
        &user1,
        &token1.address,
        &1,
        &None,
        &Some(spread),
        &None::<u64>,
        &Some(50), // user wants to swap for %.5
    );
}
