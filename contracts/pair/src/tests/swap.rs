extern crate std;
use soroban_sdk::{testutils::Address as _, Address, Env};

use super::setup::{deploy_liquidity_pool_contract, deploy_token_contract};
use crate::storage::{Asset, PoolResponse};

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
                amount: 1_000_001 - 990, // previous balance minus 990
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
    assert_eq!(token1.balance(&user1), 1989); // 999 + 990 as a result of swap
    assert_eq!(token2.balance(&user1), 1001 - 1000); // user1 sold 1k of token B on second swap
}
