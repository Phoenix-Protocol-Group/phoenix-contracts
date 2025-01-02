extern crate std;

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
    env.cost_estimate().budget().reset_unlimited();

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
        None,
    );

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address.clone());

    token1.mint(&user1, &1500);
    token1.mint(&user2, &2500);
    token1.mint(&user3, &3500);

    token2.mint(&user1, &2000);
    token2.mint(&user2, &3000);
    token2.mint(&user3, &4000);

    // all users provide liquidity in a 3:4 ratio
    pool.provide_liquidity(&user1, &1500, &2000, &None, &None::<u64>, &None::<u128>);
    pool.provide_liquidity(&user2, &1500, &2000, &None, &None::<u64>, &None::<u128>);
    pool.provide_liquidity(&user3, &1500, &2000, &None, &None::<u64>, &None::<u128>);

    // user1 assertions
    let lp_share_balance_user1 = token_share.balance(&user1);
    let query_share_result_user1 = pool.query_share(&lp_share_balance_user1);
    // rounding errors, again - 1 token is 0.0000001
    assert_eq!(
        query_share_result_user1,
        (
            Asset {
                address: token1.address.clone(),
                amount: 1499
            },
            Asset {
                address: token2.address.clone(),
                amount: 1999
            }
        )
    );

    let pool_info_before_withdrawal = pool.query_pool_info();
    assert_eq!(
        pool_info_before_withdrawal,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 4500
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 6000
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: token_share.balance(&user1)
                    + token_share.balance(&user2)
                    + token_share.balance(&user3)
            },
            stake_address: pool_info_before_withdrawal.clone().stake_address,
        }
    );

    // user1 withdraws
    pool.withdraw_liquidity(
        &user1,
        &lp_share_balance_user1,
        &1000i128,
        &1000i128,
        &None::<u64>,
    );
    let pool_info_after_withdrawal = pool.query_pool_info();
    assert_eq!(
        pool_info_after_withdrawal,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 3001
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 4001
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: token_share.balance(&user2) + token_share.balance(&user3)
            },
            stake_address: pool_info_after_withdrawal.clone().stake_address,
        }
    );

    let lp_share_balance_after_withdraw_user1: i128 = token_share.balance(&user1);
    assert_eq!(lp_share_balance_after_withdraw_user1, 0);

    // user2 assertions
    let lp_share_balance_user2 = token_share.balance(&user2);
    let query_share_result_user2 = pool.query_share(&lp_share_balance_user2);
    assert_eq!(
        query_share_result_user2,
        (
            Asset {
                address: token1.address.clone(),
                amount: 1500
            },
            Asset {
                address: token2.address.clone(),
                amount: 2000
            }
        )
    );

    // user2 withdraws his liquidity
    pool.withdraw_liquidity(
        &user2,
        &lp_share_balance_user2,
        &1500i128,
        &2000i128,
        &None::<u64>,
    );
    let pool_info_after_withdrawal = pool.query_pool_info();
    assert_eq!(
        pool_info_after_withdrawal,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 1501
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 2001
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: token_share.balance(&user3)
            },
            stake_address: pool_info_after_withdrawal.clone().stake_address,
        }
    );

    let lp_share_balance_after_withdraw_user2: i128 = token_share.balance(&user2);
    assert_eq!(lp_share_balance_after_withdraw_user2, 0);

    // user3 assertions
    let lp_share_balance_user3 = token_share.balance(&user3);
    let query_share_result_user3 = pool.query_share(&lp_share_balance_user3);
    assert_eq!(
        query_share_result_user3,
        (
            Asset {
                address: token1.address.clone(),
                amount: 1501
            },
            Asset {
                address: token2.address.clone(),
                amount: 2001
            }
        )
    );

    // user3 has 2499 shares, we are withdrawing 1499
    pool.withdraw_liquidity(&user3, &1499, &1i128, &1i128, &None::<u64>);
    let pool_info_after_withdrawal = pool.query_pool_info();
    assert_eq!(
        pool_info_after_withdrawal,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 601
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 801
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: 1000
            },
            stake_address: pool_info_after_withdrawal.clone().stake_address,
        }
    );

    let lp_share_balance_after_withdraw_user3: i128 = token_share.balance(&user3);
    assert_eq!(lp_share_balance_after_withdraw_user3, 1000);

    let query_share_result_user3 = pool.query_share(&lp_share_balance_after_withdraw_user3);
    assert_eq!(
        query_share_result_user3,
        (
            Asset {
                address: token1.address.clone(),
                amount: 601
            },
            Asset {
                address: token2.address.clone(),
                amount: 801
            }
        )
    );
}

#[test]
fn query_share_empty_pool() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

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
        None,
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
