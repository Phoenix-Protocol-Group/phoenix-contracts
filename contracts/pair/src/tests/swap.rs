extern crate std;
use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal, Symbol};

use super::setup::{deploy_token_contract, deploy_liquidity_pool_contract};
use crate::{
    token_contract,
};

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
    let pool = deploy_liquidity_pool_contract(&env, &token1.address, &token2.address);

    let token_share = token_contract::Client::new(&env, &pool.query_share_token_address());

    token1.mint(&user1, &1000);
    token2.mint(&user1, &1000);
    pool.provide_liquidity(&user1, &100, &100, &100, &100);

    // true means "selling A token"
    // selling just one token with 10% max spread allowed
    pool.swap(&user1, &true, &1, &10);
    assert_eq!(
        env.auths(),
        [
            (
                user1.clone(),
                pool.address.clone(),
                Symbol::short("swap"),
                (&user1, true, 1_i128, 100_i128).into_val(&env)
            ),
            (
                user1.clone(),
                token1.address.clone(),
                Symbol::short("transfer"),
                (&user1, &pool.address, 1_i128).into_val(&env)
            )
        ]
    );

    assert_eq!(token1.balance(&user1), 899);
    assert_eq!(token1.balance(&pool.address), 101);
    assert_eq!(token2.balance(&user1), 1001);
    assert_eq!(token2.balance(&pool.address), 99);

}
