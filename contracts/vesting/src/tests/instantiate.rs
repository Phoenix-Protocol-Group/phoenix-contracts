use curve::{Curve, SaturatingLinear};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env, String,
};

use crate::{
    storage::{Config, MinterInfo, VestingBalance, VestingTokenInfo},
    tests::setup::instantiate_vesting_client,
};

#[test]
fn instantiate_contract_succesffuly() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let vester2 = Address::generate(&env);
    let whitelisted_account = Address::generate(&env);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
        total_supply: 480,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1,
            balance: 240,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
        VestingBalance {
            address: vester2,
            balance: 240,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 30,
                min_y: 240,
                max_x: 120,
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

    assert_eq!(
        vesting_client.query_config(),
        Config {
            admin,
            whitelist: vec![&env, whitelisted_account],
            token_info: vesting_token.clone(),
            max_vesting_complexity: 10,
        }
    );

    assert_eq!(vesting_client.query_token_info(), vesting_token);
}

#[test]
fn instantiate_contract_succesffuly_with_constant_curve_minter_info() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let whitelisted_account = Address::generate(&env);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
        total_supply: 240,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1,
            balance: 240,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let allowed_vesters = vec![&env, whitelisted_account.clone()];
    let minter_info = MinterInfo {
        address: Address::generate(&env),
        capacity: Curve::Constant(511223344),
    };

    let vesting_client = instantiate_vesting_client(&env);
    env.ledger().with_mut(|li| li.timestamp = 1000);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info),
        &Some(allowed_vesters),
        &10u32,
    );

    assert_eq!(
        vesting_client.query_config(),
        Config {
            admin,
            whitelist: vec![&env, whitelisted_account],
            token_info: vesting_token,
            max_vesting_complexity: 10,
        }
    );
}

#[test]
fn instantiate_contract_succesffuly_with_empty_list_of_whitelisted_accounts() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let vester2 = Address::generate(&env);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
        total_supply: 480,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1,
            balance: 240,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
        VestingBalance {
            address: vester2,
            balance: 240,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 30,
                min_y: 240,
                max_x: 120,
                max_y: 0,
            }),
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);
    env.ledger().with_mut(|li| li.timestamp = 1000);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &None,
        &10u32,
    );

    assert_eq!(
        vesting_client.query_config(),
        Config {
            admin: admin.clone(),
            whitelist: vec![&env, admin],
            token_info: vesting_token,
            max_vesting_complexity: 10,
        }
    );
}

#[should_panic(expected = "Vesting: Initialize: At least one balance must be provided.")]
#[test]
fn instantiate_contract_without_any_vesting_balances_should_fail() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let whitelisted_account = Address::generate(&env);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
        total_supply: 0,
    };
    let vesting_balances = vec![&env];

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
}

#[should_panic(expected = "Vesting: Initialize: total supply over the cap")]
#[test]
fn instantiate_contract_should_panic_when_supply_over_the_cap() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let vester2 = Address::generate(&env);
    let whitelisted_account = Address::generate(&env);

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
            address: vester1,
            balance: 1_000,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
        VestingBalance {
            address: vester2,
            balance: 1_000,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 30,
                min_y: 240,
                max_x: 120,
                max_y: 0,
            }),
        },
    ];
    let minter_info = MinterInfo {
        address: Address::generate(&env),
        capacity: Curve::SaturatingLinear(SaturatingLinear {
            min_x: 30,
            min_y: 2,
            max_x: 120,
            max_y: 240,
        }),
    };

    let allowed_vesters = vec![&env, whitelisted_account.clone()];

    let vesting_client = instantiate_vesting_client(&env);
    env.ledger().with_mut(|li| li.timestamp = 1000);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info),
        &Some(allowed_vesters),
        &10u32,
    );
}
