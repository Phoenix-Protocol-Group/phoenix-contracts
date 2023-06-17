extern crate std;

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, IntoVal, Symbol};

use crate::{
    contract::{LiquidityPool, LiquidityPoolClient},
    token_contract,
};

fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(env, &env.register_stellar_asset_contract(admin.clone()))
}

fn install_token_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../token/target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
    );
    env.install_contract_wasm(WASM)
}

fn deploy_liquidity_pool_contract<'a>(
    env: &Env,
    token_a: &Address,
    token_b: &Address,
) -> LiquidityPoolClient<'a> {
    let pool = LiquidityPoolClient::new(env, &env.register_contract(None, LiquidityPool {}));
    let token_wasm_hash = install_token_wasm(env);
    let share_token_decimals = 7u32;
    pool.initialize(&token_wasm_hash, token_a, token_b, &share_token_decimals);
    pool
}

#[test]
fn test() {
    let env = Env::default();
    env.mock_all_auths();

    let mut admin1 = Address::random(&env);
    let mut admin2 = Address::random(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if &token2.address < &token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let user1 = Address::random(&env);
    let pool = deploy_liquidity_pool_contract(&env, &token1.address, &token2.address);

    let token_share = token_contract::Client::new(&env, &pool.query_share_token_address());

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
                (&user1, 100_u128, 100_u128, 100_u128, 100_u128).into_val(&env)
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
}
