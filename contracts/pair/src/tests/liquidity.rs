extern crate std;
use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal, Symbol};

use super::setup::{deploy_liquidity_pool_contract, deploy_token_contract};
use crate::{
    storage::{Asset, PoolResponse},
    token_contract,
};

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
    let pool = deploy_liquidity_pool_contract(&env, &token1.address, &token2.address);

    let share_token_address = pool.query_share_token_address();
    let token_share = token_contract::Client::new(&env, &share_token_address);

    token1.mint(&user1, &1000);
    assert_eq!(token1.balance(&user1), 1000);

    token2.mint(&user1, &1000);
    assert_eq!(token2.balance(&user1), 1000);

    pool.provide_liquidity(&user1, &100, &100, &100, &100);
    assert_eq!(
        env.auths(),
        [
            (
                user1.clone(),
                pool.address.clone(),
                Symbol::new(&env, "provide_liquidity"),
                (&user1, 100_i128, 100_i128, 100_i128, 100_i128).into_val(&env)
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
