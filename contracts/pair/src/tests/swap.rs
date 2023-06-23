extern crate std;
use soroban_sdk::{testutils::Address as _, Address, Env};

use super::setup::{deploy_liquidity_pool_contract, deploy_token_contract};
use crate::storage::{Asset, PoolResponse};
use decimal::Decimal;

#[test]
fn simple_swap() {
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
    let swap_fees = 0i32;
    let pool =
        deploy_liquidity_pool_contract(&env, &token1.address, &token2.address, swap_fees, None);

    token1.mint(&user1, &1_001_000);
    token2.mint(&user1, &1_001_000);
    pool.provide_liquidity(&user1, &1_000_000, &1_000_000, &1_000_000, &1_000_000);

    // true means "selling A token"
    // selling just one token with 1% max spread allowed
    let spread = 1; // 1% maximum spread allowed
    pool.swap(&user1, &true, &1, &None, &spread);
    // FIXME: Can't assert Auths because Option shows up as some Null object - how to assign it?
    // assert_eq!(
    //     env.auths(),
    //     [
    //         (
    //             user1.clone(),
    //             pool.address.clone(),
    //             Symbol::short("swap"),
    //             (&user1, true, 1_i128, 100_i128).into_val(&env)
    //         ),
    //         (
    //             user1.clone(),
    //             token1.address.clone(),
    //             Symbol::short("transfer"),
    //             (&user1, &pool.address, 1_i128).into_val(&env)
    //         )
    //     ]
    // );

    let share_token_address = pool.query_share_token_address();
    let result = pool.query_pool_info();
    assert_eq!(
        result,
        PoolResponse {
            asset_a: Asset {
                address: token1.address.clone(),
                amount: 1_000_001i128
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: 999_999i128
            },
            asset_lp_share: Asset {
                address: share_token_address.clone(),
                amount: 1_000_000i128
            }
        }
    );
    assert_eq!(token1.balance(&user1), 999); // -1 from the swap
    assert_eq!(token2.balance(&user1), 1001); // 1 from the swap

    // false means selling B token
    // this time 100 units
    pool.swap(&user1, &false, &1_000, &None, &spread);
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
                amount: 999_999 + 1000
            },
            asset_lp_share: Asset {
                address: share_token_address,
                amount: 1_000_000i128 // this has not changed
            }
        }
    );
    assert_eq!(token1.balance(&user1), 1999); // 999 + 1_000 as a result of swap
    assert_eq!(token2.balance(&user1), 1001 - 1000); // user1 sold 1k of token B on second swap
}

#[test]
fn swap_with_high_fee() {
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

    let swap_fees = 1_000i32; // 10% bps
    let fee_recipient = Address::random(&env);
    let pool = deploy_liquidity_pool_contract(
        &env,
        &token1.address,
        &token2.address,
        swap_fees,
        fee_recipient.clone(),
    );

    let initial_liquidity = 1_000_000i128;

    token1.mint(&user1, &(initial_liquidity + 100_000));
    token2.mint(&user1, &initial_liquidity);
    pool.provide_liquidity(
        &user1,
        &initial_liquidity,
        &initial_liquidity,
        &initial_liquidity,
        &initial_liquidity,
    );

    let spread = 10; // 10% maximum spread allowed

    // let's swap 100_000 units of Token 1 in 1:1 pool with 10% protocol fee
    pool.swap(&user1, &true, &100_000, &None, &spread);

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
                amount: initial_liquidity + 100_000i128
            },
            asset_b: Asset {
                address: token2.address.clone(),
                amount: initial_liquidity - output_amount,
            },
            asset_lp_share: Asset {
                address: pool.query_share_token_address(),
                amount: 1_000_000i128
            }
        }
    );
    // 10% fees are deducted from the swap result and sent to fee recipient address
    let fees = Decimal::percent(10) * output_amount;
    assert_eq!(token2.balance(&user1), output_amount - fees);
    assert_eq!(token2.balance(&fee_recipient), fees);
}
