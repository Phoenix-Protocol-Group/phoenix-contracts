use curve::{Curve, SaturatingLinear};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env, String,
};

use crate::{
    error::ContractError,
    storage::{VestingBalance, VestingTokenInfo},
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
    soroban_sdk::testutils::arbitrary::std::dbg!("before");
    vesting_client.transfer_token(&vester1, &vester2, &100);
    soroban_sdk::testutils::arbitrary::std::dbg!("after");
    vesting_client.transfer_token(&vester1, &vester2, &100);
    assert_eq!(vesting_client.query_balance(&vester1), 800);
    assert_eq!(token.balance(&vester2), 200);
    assert_eq!(vesting_client.query_vesting_total_supply(), 200);
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

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
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

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &None,
        &10u32,
    );

    vesting_client.transfer_token(&vester1, &vester2, &0);
}

#[test]
fn transfer_vesting_works() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let vester2 = Address::generate(&env);
    let vester3 = Address::generate(&env);
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
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &Some(allowed_vesters),
        &10u32,
    );

    env.ledger().with_mut(|li| {
        li.timestamp = 50;
    });

    assert_eq!(vesting_client.query_balance(&vester1), 1000);
    assert_eq!(vesting_client.query_balance(&vester2), 0);

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

    assert_eq!(vesting_client.query_balance(&vester1), 800);
    assert_eq!(vesting_client.query_balance(&vester2), 200);

    // vester1 starts with this curve and it automatically transfers to vester2
    // since vester2 has no curve before hand
    // assert_eq!(
    //     vesting_client.query_vesting(&vester1),
    //     Curve::SaturatingLinear(SaturatingLinear {
    //         min_x: 15,
    //         min_y: 120,
    //         max_x: 60,
    //         max_y: 0,
    //     })
    // );
    // assert_eq!(
    //     vesting_client.query_vesting(&vester2),
    //     Curve::SaturatingLinear(SaturatingLinear {
    //         min_x: 15,
    //         min_y: 120,
    //         max_x: 60,
    //         max_y: 0,
    //     })
    // );

    vesting_client.transfer_token(&vester2, &vester3, &100);

    assert_eq!(vesting_client.query_balance(&vester2), 100);
    assert_eq!(vesting_client.query_balance(&vester3), 100);

    // vesting does not allows us to transfer any more tokens
    assert_eq!(
        vesting_client.try_transfer_token(&vester2, &vester3, &100),
        Err(Ok(ContractError::CantMoveVestingTokens))
    );

    // fast forward time and we should be able to transfer
    env.ledger().with_mut(|li| {
        li.timestamp = 3000;
    });

    vesting_client.transfer_token(&vester2, &vester3, &100);

    assert_eq!(vesting_client.query_balance(&vester2), 0);
    assert_eq!(vesting_client.query_balance(&vester3), 200);
}

#[test]
#[should_panic(expected = "Vesting: Transfer Vesting: Transfer amount must be positive")]
fn transfer_vesting_should_fail_when_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let vester2 = Address::generate(&env);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
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
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &Some(allowed_vesters),
        &10u32,
    );

    vesting_client.transfer_vesting(
        &vester1,
        &vester2,
        &0,
        &Curve::SaturatingLinear(SaturatingLinear {
            min_x: 15,
            min_y: 120,
            max_x: 60,
            max_y: 0,
        }),
    );
}

#[test]
#[should_panic(expected = "Vesting: Transfer Vesting: Not authorized to transfer vesting")]
fn transfer_vesting_should_fail_when_sender_not_in_auth_list() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let vester2 = Address::generate(&env);
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

    let allowed_vesters = vec![&env, Address::generate(&env)];

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &Some(allowed_vesters),
        &10u32,
    );

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
}

#[test]
#[should_panic(expected = "Vesting: Transfer Vesting: Cannot transfer when non-fully vested")]
fn transfer_vesting_should_fail_when_invalid_low_value() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let vester2 = Address::generate(&env);
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
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &Some(allowed_vesters),
        &10u32,
    );

    vesting_client.transfer_vesting(
        &vester1,
        &vester2,
        &500,
        &Curve::SaturatingLinear(SaturatingLinear {
            min_x: 15,
            min_y: 120,
            max_x: 60,
            max_y: 60,
        }),
    );
}

#[test]
#[should_panic(expected = "Vesting: Assert Schedule Vest Amount: Vesting amount more than sent")]
fn transfer_vesting_should_fail_when_vesting_more_than_sent_amount() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let vester2 = Address::generate(&env);
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
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &Some(allowed_vesters),
        &10u32,
    );

    vesting_client.transfer_vesting(
        &vester1,
        &vester2,
        &100,
        &Curve::SaturatingLinear(SaturatingLinear {
            min_x: 15,
            min_y: 120,
            max_x: 60,
            max_y: 0,
        }),
    );
}
