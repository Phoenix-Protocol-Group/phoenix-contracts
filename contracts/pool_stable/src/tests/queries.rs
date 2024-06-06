extern crate std;

use pretty_assertions::assert_eq;

use soroban_sdk::{testutils::Address as _, Address, Env};

use super::setup::{deploy_stable_liquidity_pool_contract, deploy_token_contract};
use crate::{
    storage::{Asset, PoolResponse},
    token_contract,
};

#[test]
fn query_share_valid_liquidity() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let stake_manager = Address::generate(&env);
    let stake_owner = Address::generate(&env);

    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address.clone());

    token1.mint(&user1, &150);
    token1.mint(&user2, &250);
    token1.mint(&user3, &350);

    token2.mint(&user1, &200);
    token2.mint(&user2, &300);
    token2.mint(&user3, &400);

    // all users provide liquidity in a 3:4 ratio
    pool.provide_liquidity(&user1, &150, &200, &None);
    // pool.provide_liquidity(&user2, &150, &200, &None);
    // pool.provide_liquidity(&user3, &150, &200, &None);

    // user1 assertions
    let lp_share_balance_user1 = dbg!(token_share.balance(&user1));
    let query_share_result_user1 = pool.query_share(&lp_share_balance_user1);
    assert_eq!(
        query_share_result_user1,
        (
            Asset {
                address: token1.address.clone(),
                amount: 150
            },
            Asset {
                address: token2.address.clone(),
                amount: 201
            }
        )
    );

    // let pool_info_before_withdrawal = pool.query_pool_info();
    // assert_eq!(
    //     pool_info_before_withdrawal,
    //     PoolResponse {
    //         asset_a: Asset {
    //             address: token1.address.clone(),
    //             amount: 450
    //         },
    //         asset_b: Asset {
    //             address: token2.address.clone(),
    //             amount: 600
    //         },
    //         asset_lp_share: Asset {
    //             address: share_token_address.clone(),
    //             amount: token_share.balance(&user1) + token_share.balance(&user2) + token_share.balance(&user3)
    //         },
    //         stake_address: pool_info_before_withdrawal.clone().stake_address,
    //     }
    // );

    // pool.withdraw_liquidity(&user1, &lp_share_balance_user1, &100i128, &100i128);
    // let pool_info_after_withdrawal = pool.query_pool_info();
    // assert_eq!(
    //     pool_info_after_withdrawal,
    //     PoolResponse {
    //         asset_a: Asset {
    //             address: token1.address.clone(),
    //             amount: 301
    //         },
    //         asset_b: Asset {
    //             address: token2.address.clone(),
    //             amount: 401
    //         },
    //         asset_lp_share: Asset {
    //             address: share_token_address.clone(),
    //             amount: 346
    //         },
    //         stake_address: pool_info_after_withdrawal.clone().stake_address,
    //     }
    // );

    // let lp_share_balance_after_withdraw_user1: i128 = token_share.balance(&user1);
    // assert_eq!(lp_share_balance_after_withdraw_user1, 0);

    // // user2 assertions
    // let lp_share_balance_user2 = token_share.balance(&user2);
    // let query_share_result_user2 = pool.query_share(&lp_share_balance_user2);
    // assert_eq!(
    //     query_share_result_user2,
    //     (
    //         Asset {
    //             address: token1.address.clone(),
    //             amount: 150
    //         },
    //         Asset {
    //             address: token2.address.clone(),
    //             amount: 200
    //         }
    //     )
    // );

    // pool.withdraw_liquidity(&user2, &lp_share_balance_user2, &150i128, &200i128);
    // let pool_info_after_withdrawal = pool.query_pool_info();
    // assert_eq!(
    //     pool_info_after_withdrawal,
    //     PoolResponse {
    //         asset_a: Asset {
    //             address: token1.address.clone(),
    //             amount: 151
    //         },
    //         asset_b: Asset {
    //             address: token2.address.clone(),
    //             amount: 201
    //         },
    //         asset_lp_share: Asset {
    //             address: share_token_address.clone(),
    //             amount: 173
    //         },
    //         stake_address: pool_info_after_withdrawal.clone().stake_address,
    //     }
    // );

    // let lp_share_balance_after_withdraw_user2: i128 = token_share.balance(&user2);
    // assert_eq!(lp_share_balance_after_withdraw_user2, 0);

    // // user3 assertions
    // let lp_share_balance_user3 = token_share.balance(&user3);
    // let query_share_result_user3 = pool.query_share(&lp_share_balance_user3);
    // assert_eq!(
    //     query_share_result_user3,
    //     (
    //         Asset {
    //             address: token1.address.clone(),
    //             amount: 151
    //         },
    //         Asset {
    //             address: token2.address.clone(),
    //             amount: 201
    //         }
    //     )
    // );

    // // user3 has 173 shares, we are withdrawing 73
    // pool.withdraw_liquidity(&user3, &73, &1i128, &1i128);
    // let pool_info_after_withdrawal = pool.query_pool_info();
    // assert_eq!(
    //     pool_info_after_withdrawal,
    //     PoolResponse {
    //         asset_a: Asset {
    //             address: token1.address.clone(),
    //             amount: 88
    //         },
    //         asset_b: Asset {
    //             address: token2.address.clone(),
    //             amount: 117
    //         },
    //         asset_lp_share: Asset {
    //             address: share_token_address.clone(),
    //             amount: 100
    //         },
    //         stake_address: pool_info_after_withdrawal.clone().stake_address,
    //     }
    // );

    // let lp_share_balance_after_withdraw_user3: i128 = token_share.balance(&user3);
    // assert_eq!(lp_share_balance_after_withdraw_user3, 100);

    // let query_share_result_user3 = pool.query_share(&lp_share_balance_after_withdraw_user3);
    // assert_eq!(
    //     query_share_result_user3,
    //     (
    //         Asset {
    //             address: token1.address.clone(),
    //             amount: 88
    //         },
    //         Asset {
    //             address: token2.address.clone(),
    //             amount: 117
    //         }
    //     )
    // );
}

#[test]
fn query_share_empty_pool() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }
    let user1 = Address::generate(&env);
    let stake_manager = Address::generate(&env);
    let stake_owner = Address::generate(&env);

    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    let lp_share_balance = token_share.balance(&user1);
    let query_share_result = pool.query_share(&lp_share_balance);
    assert_eq!(
        query_share_result,
        (
            Asset {
                address: token1.address,
                amount: 0
            },
            Asset {
                address: token2.address,
                amount: 0
            }
        )
    );
}
