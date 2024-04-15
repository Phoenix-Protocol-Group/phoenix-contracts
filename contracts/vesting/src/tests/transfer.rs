use curve::{Curve, SaturatingLinear};
use soroban_sdk::{
    testutils::{arbitrary::std::dbg, Address as _, Ledger},
    vec, Address, Env, String,
};

use crate::{
    storage::{MinterInfo, VestingBalance, VestingTokenInfo},
    tests::setup::instantiate_vesting_client,
};

use super::setup::deploy_token_contract;

#[test]
fn transfer_tokens() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let vester2 = Address::generate(&env);
    let whitelisted_account = Address::generate(&env);
    let token = deploy_token_contract(&env, &admin);

    token.mint(&vester1, &1_000);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let allowed_vesters = vec![&env, whitelisted_account.clone()];

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &Some(allowed_vesters),
        &10u32,
    );
    assert_eq!(token.balance(&vester2), 0);
    dbg!("before");
    vesting_client.transfer_token(&vester1, &vester2, &100);
    dbg!("after");
    assert_eq!(vesting_client.query_balance(&vester1), 900);
    assert_eq!(token.balance(&vester2), 100);
    assert_eq!(vesting_client.query_total_supply(), 200);
}

#[test]
#[should_panic(expected = "Vesting: Transfer token: Invalid transfer amount")]
fn transfer_tokens_should_fail_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let vester2 = Address::generate(&env);
    let whitelisted_account = Address::generate(&env);
    let token = deploy_token_contract(&env, &admin);

    token.mint(&vester1, &1_000);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let allowed_vesters = vec![&env, whitelisted_account.clone()];

    let vesting_client = instantiate_vesting_client(&env);
    env.ledger().with_mut(|li| li.timestamp = 1000);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &Some(allowed_vesters),
        &10u32,
    );

    assert_eq!(token.balance(&vester2), 0);
    vesting_client.transfer_token(&vester1, &vester2, &0);
    assert_eq!(vesting_client.query_balance(&vester1), 900);
    assert_eq!(token.balance(&vester2), 100);
}

#[test]
fn transfer_vesting_works() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let vester2 = Address::generate(&env);
    dbg!(&vester1, &vester2, &admin);
    let token = deploy_token_contract(&env, &admin);

    token.mint(&vester1, &1_000);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let allowed_vesters = vec![&env, vester1.clone()];

    let vesting_client = instantiate_vesting_client(&env);
    env.ledger().with_mut(|li| li.timestamp = 1000);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &Some(allowed_vesters),
        &10u32,
    );

    dbg!(vesting_client.query_vesting(&vester1));
    dbg!(vesting_client.query_vesting(&vester2));
    vesting_client.transfer_vesting(
        &vester1,
        &vester2,
        &200,
        &Curve::SaturatingLinear(SaturatingLinear {
            min_x: 15,
            min_y: 120,
            max_x: 60,
            max_y: 0,
        }),
    );

    dbg!("after", vesting_client.query_vesting(&vester2));
    assert_eq!(1, 1);
}
