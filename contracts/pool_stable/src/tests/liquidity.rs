extern crate std;

use phoenix::utils::AutoUnstakeInfo;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Ledger},
    vec, Address, Env, IntoVal, Symbol,
};

use super::setup::{deploy_stable_liquidity_pool_contract, deploy_token_contract};
use crate::{
    stake_contract::{
        self, Stake, StakedResponse, WithdrawableReward, WithdrawableRewardsResponse,
    },
    storage::{Asset, PoolResponse},
    token_contract,
};

#[test]
fn provide_liqudity() {
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

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    token1.mint(&user1, &1000);
    assert_eq!(token1.balance(&user1), 1000);

    token2.mint(&user1, &1000);
    assert_eq!(token2.balance(&user1), 1000);

    // tokens 1 & 2 have 7 decimal digits, meaning those values are 0.0001 of token
    pool.provide_liquidity(
        &user1,
        &1000,
        &1000,
        &None,
        &None::<u64>,
        &None::<u128>,
        &false,
    );

    assert_eq!(
        env.auths(),
        [(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    pool.address.clone(),
                    Symbol::new(&env, "provide_liquidity"),
                    (
                        &user1,
                        1000i128,
                        1000i128,
                        None::<i64>,
                        None::<u64>,
                        None::<u128>,
                        false
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            token1.address.clone(),
                            symbol_short!("transfer"),
                            (&user1, &pool.address, 1000_i128).into_val(&env)
                        )),
                        sub_invocations: std::vec![],
                    },
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            token2.address.clone(),
                            symbol_short!("transfer"),
                            (&user1, &pool.address, 1000_i128).into_val(&env)
                        )),
                        sub_invocations: std::vec![],
                    },
                ],
            }
        ),]
    );

    assert_eq!(token_share.balance(&user1), 1000);
    assert_eq!(token_share.balance(&pool.address), 0);
    assert_eq!(token1.balance(&user1), 0);
    assert_eq!(token1.balance(&pool.address), 1000);
    assert_eq!(token2.balance(&user1), 0);
    assert_eq!(token2.balance(&pool.address), 1000);

    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address,
                amount: 1000i128
            },
            asset_b: Asset {
                address: token2.address,
                amount: 1000i128
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 1000i128
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );

    assert_eq!(pool.query_total_issued_lp(), 1000);
}

#[test]
fn provide_liqudity_big_numbers() {
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

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    // minting 1_000_000 tokens to the user
    token1.mint(&user1, &10_000_000_000_000);
    assert_eq!(token1.balance(&user1), 10_000_000_000_000);

    token2.mint(&user1, &10_000_000_000_000);
    assert_eq!(token2.balance(&user1), 10_000_000_000_000);

    // user1 provides 100_000 tokens
    pool.provide_liquidity(
        &user1,
        &1_000_000_000_000,
        &1_000_000_000_000,
        &None,
        &None::<u64>,
        &None::<u128>,
        &false,
    );

    assert_eq!(
        env.auths(),
        [(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    pool.address.clone(),
                    Symbol::new(&env, "provide_liquidity"),
                    (
                        &user1,
                        1_000_000_000_000_i128,
                        1_000_000_000_000_i128,
                        None::<i64>,
                        None::<u64>,
                        None::<u128>,
                        false
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            token1.address.clone(),
                            symbol_short!("transfer"),
                            (&user1, &pool.address, 1_000_000_000_000_i128).into_val(&env)
                        )),
                        sub_invocations: std::vec![],
                    },
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            token2.address.clone(),
                            symbol_short!("transfer"),
                            (&user1, &pool.address, 1_000_000_000_000_i128).into_val(&env)
                        )),
                        sub_invocations: std::vec![],
                    },
                ],
            }
        ),]
    );

    assert_eq!(token_share.balance(&user1), 1999999999000);
    assert_eq!(token_share.balance(&pool.address), 0);
    assert_eq!(token1.balance(&user1), 9000000000000);
    assert_eq!(token1.balance(&pool.address), 1000000000000);
    assert_eq!(token2.balance(&user1), 9000000000000);
    assert_eq!(token2.balance(&pool.address), 1000000000000);

    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address,
                amount: 1000000000000i128
            },
            asset_b: Asset {
                address: token2.address,
                amount: 1000000000000i128
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 1999999999000i128
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );

    assert_eq!(pool.query_total_issued_lp(), 1999999999000);
}

#[test]
fn withdraw_liquidity() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
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

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    token1.mint(&user1, &1000);
    token2.mint(&user1, &1000);
    // tokens 1 & 2 have 7 decimal digits, meaning those values are 0.0001 of token
    pool.provide_liquidity(
        &user1,
        &1000,
        &1000,
        &None,
        &None::<u64>,
        &None::<u128>,
        &false,
    );

    assert_eq!(token_share.balance(&user1), 1000);
    assert_eq!(token_share.balance(&pool.address), 0);
    assert_eq!(token1.balance(&user1), 0);
    assert_eq!(token1.balance(&pool.address), 1000);
    assert_eq!(token2.balance(&user1), 0);
    assert_eq!(token2.balance(&pool.address), 1000);

    let share_amount = 500; // half of the shares
    let min_a = 500;
    let min_b = 500;
    pool.withdraw_liquidity(
        &user1,
        &share_amount,
        &min_a,
        &min_b,
        &None::<u64>,
        &None::<AutoUnstakeInfo>,
    );
    assert_eq!(
        env.auths(),
        [(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    pool.address.clone(),
                    Symbol::new(&env, "withdraw_liquidity"),
                    (
                        &user1,
                        500i128,
                        500i128,
                        500i128,
                        None::<u64>,
                        None::<AutoUnstakeInfo>
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        share_token_address.clone(),
                        symbol_short!("transfer"),
                        (&user1, &pool.address, 500_i128).into_val(&env)
                    )),
                    sub_invocations: std::vec![],
                },],
            }
        ),]
    );

    assert_eq!(token_share.balance(&user1), 500);
    assert_eq!(token_share.balance(&pool.address), 0); // sanity check
    assert_eq!(token1.balance(&user1), 500);
    assert_eq!(token1.balance(&pool.address), 500);
    assert_eq!(token2.balance(&user1), 500);
    assert_eq!(token2.balance(&pool.address), 500);

    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 500i128,
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 500i128,
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 500i128,
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );

    // clear the pool
    pool.withdraw_liquidity(
        &user1,
        &500,
        &500,
        &500,
        &None::<u64>,
        &None::<AutoUnstakeInfo>,
    );
    assert_eq!(token_share.balance(&user1), 0);
    assert_eq!(token_share.balance(&pool.address), 0); // sanity check
    assert_eq!(token1.balance(&user1), 1000);
    assert_eq!(token1.balance(&pool.address), 0);
    assert_eq!(token2.balance(&user1), 1000);
    assert_eq!(token2.balance(&pool.address), 0);
}

// Single asset liquidity providing is now disabled

// #[test]
// #[should_panic = "Pool: split_deposit_based_on_pool_ratio: Both pools and deposit must be a positive!"]
// fn provide_liqudity_single_asset_on_empty_pool() {
//     let env = Env::default();
//     env.mock_all_auths();
//
//     let mut admin1 = Address::generate(&env);
//     let mut admin2 = Address::generate(&env);
//     let user1 = Address::generate(&env);
//
//     let mut token1 = deploy_token_contract(&env, &admin1);
//     let mut token2 = deploy_token_contract(&env, &admin2);
//     if token2.address < token1.address {
//         std::mem::swap(&mut token1, &mut token2);
//         std::mem::swap(&mut admin1, &mut admin2);
//     }
//     let swap_fees = 0i64;
//     let pool = deploy_stable_liquidity_pool_contract(
//         &env,
//         None,
//         (&token1.address, &token2.address),
//         swap_fees,
//         None,
//         None,
//         None,
//     );
//
//     token1.mint(&user1, &1_000_000);
//
//     // providing liquidity with single asset is not allowed on an empty pool
//     pool.provide_liquidity(&user1, &1_000_000, &0i128, &None);
// }
//
// #[test]
// fn provide_liqudity_single_asset_equal() {
//     let env = Env::default();
//     env.mock_all_auths();
//     env.budget().reset_unlimited();
//
//     let mut admin1 = Address::generate(&env);
//     let mut admin2 = Address::generate(&env);
//     let user1 = Address::generate(&env);
//
//     let mut token1 = deploy_token_contract(&env, &admin1);
//     let mut token2 = deploy_token_contract(&env, &admin2);
//     if token2.address < token1.address {
//         std::mem::swap(&mut token1, &mut token2);
//         std::mem::swap(&mut admin1, &mut admin2);
//     }
//     let swap_fees = 0i64;
//     let pool = deploy_stable_liquidity_pool_contract(
//         &env,
//         None,
//         (&token1.address, &token2.address),
//         swap_fees,
//         None,
//         None,
//         None,
//     );
//
//     token1.mint(&user1, &10_000_000);
//     token2.mint(&user1, &10_000_000);
//
//     // providing liquidity with single asset is not allowed on an empty pool
//     pool.provide_liquidity(&user1, &10_000_000, &10_000_000, &None);
//     assert_eq!(token1.balance(&pool.address), 10_000_000);
//     assert_eq!(token2.balance(&pool.address), 10_000_000);
//
//     token1.mint(&user1, &100_000);
//
//     // Providing 100k of token1 to 1:1 pool will perform swap which will create imbalance
//     pool.provide_liquidity(&user1, &100_000, &0i128, &None);
//     // before swap : A(10_000_000), B(10_000_000)
//     // since pool is equal divides 50/50 sum for swap
//     // swap 50k A for B = 49752
//     // after swap : A(10_050_000), B(9_950_248)
//     // after providing liquidity
//     // A(1_100_000), B(1_000_000)
//
//     assert_eq!(token1.balance(&pool.address), 10_100_000);
//     // because of lack of fees, first swap took from pool b exact amount
//     // that was provided to the pool in the next step
//     assert_eq!(token2.balance(&pool.address), 10_000_000);
//     assert_eq!(token1.balance(&user1), 0);
//     assert_eq!(token2.balance(&user1), 0);
// }
//
// #[test]
// fn provide_liqudity_single_asset_equal_with_fees() {
//     let env = Env::default();
//     env.mock_all_auths();
//     env.budget().reset_unlimited();
//
//     let mut admin1 = Address::generate(&env);
//     let mut admin2 = Address::generate(&env);
//     let user1 = Address::generate(&env);
//
//     let mut token1 = deploy_token_contract(&env, &admin1);
//     let mut token2 = deploy_token_contract(&env, &admin2);
//     if token2.address < token1.address {
//         std::mem::swap(&mut token1, &mut token2);
//         std::mem::swap(&mut admin1, &mut admin2);
//     }
//     let swap_fees = 1_000i64; // 10% bps
//     let pool = deploy_stable_liquidity_pool_contract(
//         &env,
//         None,
//         (&token1.address, &token2.address),
//         swap_fees,
//         None,
//         None,
//         None,
//     );
//
//     let initial_pool_liquidity = 10_000_000;
//     token1.mint(&user1, &initial_pool_liquidity);
//     token2.mint(&user1, &initial_pool_liquidity);
//
//     // providing liquidity with single asset is not allowed on an empty pool
//     pool.provide_liquidity(
//         &user1,
//         &initial_pool_liquidity,
//         &initial_pool_liquidity,
//         &None,
//     );
//     assert_eq!(token1.balance(&pool.address), initial_pool_liquidity);
//     assert_eq!(token2.balance(&pool.address), initial_pool_liquidity);
//
//     let token_a_amount = 100_000;
//     token1.mint(&user1, &token_a_amount);
//     // Providing 100k of token1 to 1:1 pool will perform swap which will create imbalance
//     pool.provide_liquidity(&user1, &token_a_amount, &0i128, &None);
//     // before swap : A(10_000_000), B(10_000_000)
//     // algorithm splits 100k in such way, so that after swapping (with 10% fee)
//     // it will provide liquidity maintining 1:1 ratio
//     // split is 47_266 token A and 47213 token B (52_734 of token A was swapped to B)
//     // after swap : A(10_052_734), B(9_947_542)
//     // after providing liquidity
//     // A(1_100_000), B(9_994_755)
//
//     // return_amount: i128 = ask_pool - (cp / (offer_pool + offer_amount))
//     let return_amount = 52_458; // that's how many tokens B would be received from 52_734 tokens A
//     let fees = Decimal::percent(10);
//     assert_eq!(
//         token1.balance(&pool.address),
//         initial_pool_liquidity + token_a_amount
//     );
//     assert_eq!(
//         token2.balance(&pool.address),
//         initial_pool_liquidity - return_amount * fees
//     );
//     assert_eq!(token1.balance(&user1), 0);
//     assert_eq!(token2.balance(&user1), 0);
// }

#[test]
#[should_panic(expected = "The value 10001 is out of range. Must be between 0 and 10000 bps.")]
fn provide_liqudity_too_high_fees() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
    let swap_fees = 10_001i64;
    deploy_stable_liquidity_pool_contract(
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
}

#[test]
#[should_panic(expected = "value cannot be less than or equal zero")]
fn swap_with_no_amounts() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);
    let user1 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
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
    // providing all amounts as None
    pool.provide_liquidity(
        &user1,
        &0i128,
        &0i128,
        &None,
        &None::<u64>,
        &None::<u128>,
        &false,
    );
}

#[test]
#[should_panic(
    expected = "Pool Stable: WithdrawLiquidity: Minimum amount of token_a or token_b is not satisfied!"
)]
fn withdraw_liqudity_below_min() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);
    let user1 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
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

    token1.mint(&user1, &1000);
    token2.mint(&user1, &1000);
    pool.provide_liquidity(
        &user1,
        &1000,
        &1000,
        &None,
        &None::<u64>,
        &None::<u128>,
        &false,
    );

    let share_amount = 500;
    // Expecting min_a and/or min_b as huge bigger then available
    pool.withdraw_liquidity(
        &user1,
        &share_amount,
        &3000,
        &3000,
        &None::<u64>,
        &None::<AutoUnstakeInfo>,
    );
}

#[test]
fn provide_liqudity_with_deadline_works() {
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

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    token1.mint(&user1, &1000);
    assert_eq!(token1.balance(&user1), 1000);

    token2.mint(&user1, &1000);
    assert_eq!(token2.balance(&user1), 1000);

    env.ledger().with_mut(|li| li.timestamp = 99);
    pool.provide_liquidity(
        &user1,
        &1000,
        &1000,
        &None,
        &Some(100),
        &None::<u128>,
        &false,
    );

    assert_eq!(token_share.balance(&user1), 1000);
    assert_eq!(token_share.balance(&pool.address), 0);
    assert_eq!(token1.balance(&user1), 0);
    assert_eq!(token1.balance(&pool.address), 1000);
    assert_eq!(token2.balance(&user1), 0);
    assert_eq!(token2.balance(&pool.address), 1000);

    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address,
                amount: 1000i128
            },
            asset_b: Asset {
                address: token2.address,
                amount: 1000i128
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 1000i128
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );

    assert_eq!(pool.query_total_issued_lp(), 1000);
}

#[test]
#[should_panic(expected = "Pool Stable: Provide Liquidity: Transaction executed after deadline!")]
fn provide_liqudity_past_deadline_should_panic() {
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

    token1.mint(&user1, &1000);
    assert_eq!(token1.balance(&user1), 1000);

    token2.mint(&user1, &1000);
    assert_eq!(token2.balance(&user1), 1000);

    env.ledger().with_mut(|li| li.timestamp = 100);
    pool.provide_liquidity(
        &user1,
        &1000,
        &1000,
        &None,
        &Some(99),
        &None::<u128>,
        &false,
    );
}

#[test]
fn withdraw_liquidity_with_deadline_should_work() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
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

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    token1.mint(&user1, &1000);
    token2.mint(&user1, &1000);

    pool.provide_liquidity(
        &user1,
        &1000,
        &1000,
        &None,
        &None::<u64>,
        &None::<u128>,
        &false,
    );

    assert_eq!(token_share.balance(&user1), 1000);
    assert_eq!(token_share.balance(&pool.address), 0);
    assert_eq!(token1.balance(&user1), 0);
    assert_eq!(token1.balance(&pool.address), 1000);
    assert_eq!(token2.balance(&user1), 0);
    assert_eq!(token2.balance(&pool.address), 1000);

    let share_amount = 500;
    let min_a = 500;
    let min_b = 500;
    env.ledger().with_mut(|li| li.timestamp = 49);
    pool.withdraw_liquidity(
        &user1,
        &share_amount,
        &min_a,
        &min_b,
        &Some(50),
        &None::<AutoUnstakeInfo>,
    );

    assert_eq!(token_share.balance(&user1), 500);
    assert_eq!(token_share.balance(&pool.address), 0);
    assert_eq!(token1.balance(&user1), 500);
    assert_eq!(token1.balance(&pool.address), 500);
    assert_eq!(token2.balance(&user1), 500);
    assert_eq!(token2.balance(&pool.address), 500);

    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 500i128,
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 500i128,
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 500i128,
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );

    // clear the pool
    env.ledger().with_mut(|li| li.timestamp = 99);
    pool.withdraw_liquidity(
        &user1,
        &500,
        &500,
        &500,
        &Some(100),
        &None::<AutoUnstakeInfo>,
    );
    assert_eq!(token_share.balance(&user1), 0);
    assert_eq!(token_share.balance(&pool.address), 0);
    assert_eq!(token1.balance(&user1), 1000);
    assert_eq!(token1.balance(&pool.address), 0);
    assert_eq!(token2.balance(&user1), 1000);
    assert_eq!(token2.balance(&pool.address), 0);
}

#[test]
#[should_panic(expected = "Pool Stable: Withdraw Liquidity: Transaction executed after deadline!")]
fn withdraw_liquidity_past_deadline_should_panic() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
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

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    token1.mint(&user1, &1000);
    token2.mint(&user1, &1000);

    pool.provide_liquidity(
        &user1,
        &1000,
        &1000,
        &None,
        &None::<u64>,
        &None::<u128>,
        &false,
    );

    assert_eq!(token_share.balance(&user1), 1000);
    assert_eq!(token_share.balance(&pool.address), 0);
    assert_eq!(token1.balance(&user1), 0);
    assert_eq!(token1.balance(&pool.address), 1000);
    assert_eq!(token2.balance(&user1), 0);
    assert_eq!(token2.balance(&pool.address), 1000);

    let share_amount = 500;
    let min_a = 500;
    let min_b = 500;
    env.ledger().with_mut(|li| li.timestamp = 50);
    pool.withdraw_liquidity(
        &user1,
        &share_amount,
        &min_a,
        &min_b,
        &Some(49),
        &None::<AutoUnstakeInfo>,
    );
}

#[ignore = "maybe not the right logic when checking if min_shares is above shares"]
#[test]
#[should_panic(expected = "Pool Stable: Provide Liquidity: Slippage tolerance exceeded")]
fn provide_liqudity_should_panic_when_shares_to_be_minted_below_minimum_shares() {
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
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        Some(10_000),
        None,
        manager,
        factory,
        None,
    );

    token1.mint(&user1, &2000);
    token2.mint(&user1, &2000);

    pool.provide_liquidity(
        &user1,
        &1000,
        &1000,
        &None,
        &None::<u64>,
        &None::<u128>,
        &false,
    );
    pool.provide_liquidity(
        &user1,
        &1000,
        &1000,
        &Some(1),
        &None::<u64>,
        &None::<u128>,
        &false,
    );
}

#[test]
fn provide_liqudity_with_user_specified_minimum_lp_shares() {
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

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    token1.mint(&user1, &1000);
    assert_eq!(token1.balance(&user1), 1000);

    token2.mint(&user1, &1000);
    assert_eq!(token2.balance(&user1), 1000);

    pool.provide_liquidity(
        &user1,
        &1000,
        &1000,
        &None,
        &None::<u64>,
        &Some(1_000),
        &false,
    );

    assert_eq!(token_share.balance(&user1), 1000);
    assert_eq!(token_share.balance(&pool.address), 0);
    assert_eq!(token1.balance(&user1), 0);
    assert_eq!(token1.balance(&pool.address), 1000);
    assert_eq!(token2.balance(&user1), 0);
    assert_eq!(token2.balance(&pool.address), 1000);

    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address,
                amount: 1000i128
            },
            asset_b: Asset {
                address: token2.address,
                amount: 1000i128
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 1000i128
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );

    assert_eq!(pool.query_total_issued_lp(), 1000);
}

#[test]
#[should_panic(
    expected = "Pool Stable: Provide Liquidity: Issued shares are less than the user requsted"
)]
fn provide_liqudity_with_user_specified_minimum_lp_shares_should_panic_when_user_expected_is_more_than_issued(
) {
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

    token1.mint(&user1, &1000);
    assert_eq!(token1.balance(&user1), 1000);

    token2.mint(&user1, &1000);
    assert_eq!(token2.balance(&user1), 1000);

    // Here the user provides `1_000` of each token and expects more than the pool allocation
    pool.provide_liquidity(
        &user1,
        &1000,
        &1000,
        &None,
        &None::<u64>,
        &Some(1_001),
        &false,
    );
}

#[test]
fn provide_liqudity_with_auto_stake() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &manager);

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
        manager.clone(),
        factory,
        None,
    );

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);
    let result = pool.query_pool_info();

    let stake = stake_contract::Client::new(&env, &result.stake_address);

    stake.create_distribution_flow(&manager, &reward_token.address);

    let reward_amount = 1_000_000_000_000_000;
    //100_000_000
    reward_token.mint(&manager, &reward_amount);
    let distribution_duration = 10_000;
    stake.fund_distribution(
        &manager,
        &0,
        &distribution_duration,
        &reward_token.address,
        &reward_amount,
    );

    token1.mint(&user1, &1000);
    assert_eq!(token1.balance(&user1), 1000);

    token2.mint(&user1, &1000);
    assert_eq!(token2.balance(&user1), 1000);

    // tokens 1 & 2 have 7 decimal digits, meaning those values are 0.0001 of token
    let stake_timestamp = env.ledger().timestamp();
    pool.provide_liquidity(
        &user1,
        &1000,
        &1000,
        &None,
        &None::<u64>,
        &None::<u128>,
        &true,
    );

    assert_eq!(token_share.balance(&user1), 0);
    assert_eq!(token_share.balance(&pool.address), 0);
    assert_eq!(token1.balance(&user1), 0);
    assert_eq!(token1.balance(&pool.address), 1000);
    assert_eq!(token2.balance(&user1), 0);
    assert_eq!(token2.balance(&pool.address), 1000);

    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address,
                amount: 1000i128
            },
            asset_b: Asset {
                address: token2.address,
                amount: 1000i128
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 1000i128
            },
            stake_address: pool.query_stake_contract_address(),
        }
    );

    assert_eq!(
        stake.query_withdrawable_rewards(&user1),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 0
                }
            ]
        }
    );

    assert_eq!(
        stake.query_staked(&user1),
        StakedResponse {
            stakes: vec![
                &env,
                Stake {
                    stake: 1_000,
                    stake_timestamp
                }
            ]
        }
    );

    assert_eq!(pool.query_total_issued_lp(), 1000);

    env.ledger().set_timestamp(distribution_duration / 2);
    stake.distribute_rewards();

    assert_eq!(
        stake.query_withdrawable_rewards(&user1),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address,
                    reward_amount: (reward_amount / 2) as u128
                }
            ]
        }
    );
}

#[test]
fn withdraw_liquidity_with_auto_unstake() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let user = Address::generate(&env);
    let stake_manager = Address::generate(&env);

    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        stake_manager.clone(),
        Address::generate(&env),
        None,
    );

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    let result = pool.query_pool_info();
    let stake = stake_contract::Client::new(&env, &result.stake_address);

    stake.create_distribution_flow(&stake_manager, &reward_token.address);
    let reward_amount = 1_000_000_000_000_000;
    reward_token.mint(&stake_manager, &reward_amount);
    let distribution_duration = 10_000;
    stake.fund_distribution(
        &stake_manager,
        &0,
        &distribution_duration,
        &reward_token.address,
        &reward_amount,
    );

    token1.mint(&user, &1_000_000_000_000_000);
    token2.mint(&user, &1_000_000_000_000_000);
    let stake_timestamp = env.ledger().timestamp();
    pool.provide_liquidity(
        &user,
        &1_000_000_000_000,
        &1_000_000_000_000,
        &None,
        &None::<u64>,
        &None::<u128>,
        &true,
    );

    soroban_sdk::testutils::arbitrary::std::dbg!();
    let initial_stake = stake.query_staked(&user);
    assert_eq!(
        initial_stake,
        StakedResponse {
            stakes: vec![
                &env,
                Stake {
                    stake: initial_stake.stakes.get(0).unwrap().stake,
                    stake_timestamp
                }
            ]
        }
    );
    assert_eq!(
        initial_stake.stakes.get(0).unwrap().stake,
        1_999_999_999_000
    );
    assert_eq!(
        initial_stake.stakes.get(0).unwrap().stake_timestamp,
        stake_timestamp
    );

    soroban_sdk::testutils::arbitrary::std::dbg!();
    let share_amount = 1_999_999_999_000;
    let min_a = 500_000_000_000;
    let min_b = 500_000_000_000;
    let auto_unstake_info = AutoUnstakeInfo {
        stake_amount: initial_stake.stakes.get(0).unwrap().stake,
        stake_timestamp,
    };

    soroban_sdk::testutils::arbitrary::std::dbg!();
    env.ledger().with_mut(|li| li.timestamp += 1000);
    stake.distribute_rewards();
    assert_eq!(
        stake.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 99_999_999_999_999
                }
            ]
        }
    );

    assert_eq!(reward_token.balance(&user), 0); // should be 0 as the user has no rewards so far
    pool.withdraw_liquidity(
        &user,
        &share_amount,
        &min_a,
        &min_b,
        &None::<u64>,
        &Some(auto_unstake_info),
    );

    assert_eq!(reward_token.balance(&user), 99_999_999_999_999); // all rewards withdrawn

    let remaining_stake = stake.query_staked(&user);
    assert!(remaining_stake.stakes.is_empty());
    assert!(remaining_stake.stakes.get(0).is_none(),);

    assert_eq!(token1.balance(&user), 1000000000000000);
    assert_eq!(token2.balance(&user), 1000000000000000);
    assert_eq!(token_share.balance(&user), 0); // user withdraw everything
}

#[test]
#[should_panic(expected = "Error(Contract, #509)")]
fn withdraw_liquidity_with_auto_unstake_should_fail_when_unexisting_stake() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let user = Address::generate(&env);
    let stake_manager = Address::generate(&env);

    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        stake_manager.clone(),
        Address::generate(&env),
        None,
    );

    let share_amount = 500_000_000_000_000;
    let min_a = 500_000_000_000_000;
    let min_b = 500_000_000_000_000;
    let auto_unstake_info = AutoUnstakeInfo {
        stake_amount: 999_999_999_999_000,
        stake_timestamp: 0,
    };

    pool.withdraw_liquidity(
        &user,
        &share_amount,
        &min_a,
        &min_b,
        &None::<u64>,
        &Some(auto_unstake_info),
    );
}
