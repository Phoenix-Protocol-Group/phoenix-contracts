use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env, String,
};

use crate::{
    storage::{DistributionInfo, MinterInfo, VestingBalance, VestingTokenInfo},
    tests::setup::instantiate_vesting_client,
};

#[test]
fn instantiate_contract_succesffully() {
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
            address: vester1.clone(),
            balance: 240,
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
        VestingBalance {
            address: vester2,
            balance: 240,
            distribution_info: DistributionInfo {
                start_timestamp: 30,
                end_timestamp: 120,
                amount: 240,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);
    env.ledger().with_mut(|li| li.timestamp = 1000);
    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    assert_eq!(vesting_client.query_token_info(), vesting_token);
    assert_eq!(
        vesting_client.query_distribution_info(&vester1),
        vesting_balances.get(0).unwrap().distribution_info
    );
}

#[test]
fn instantiate_contract_succesffully_with_constant_curve_minter_info() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

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
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let minter_info = MinterInfo {
        address: Address::generate(&env),
        mint_cap: 511223344,
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info),
        &10u32,
    );

    assert_eq!(vesting_client.query_token_info(), vesting_token);
}

#[should_panic(expected = "Vesting: Initialize: At least one vesting schedule must be provided.")]
#[test]
fn instantiate_contract_without_any_vesting_balances_should_fail() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
        total_supply: 0,
    };
    let vesting_balances = vec![&env];

    let vesting_client = instantiate_vesting_client(&env);
    env.ledger().with_mut(|li| li.timestamp = 1000);
    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);
}

#[should_panic(expected = "Vesting: Initialize: total supply over the cap")]
#[test]
fn instantiate_contract_should_panic_when_supply_over_the_cap() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
        total_supply: 1_000,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1,
            balance: 1_000,
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let minter_info = MinterInfo {
        address: Address::generate(&env),
        mint_cap: 500,
    };

    let vesting_client = instantiate_vesting_client(&env);
    env.ledger().with_mut(|li| li.timestamp = 1000);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info),
        &10u32,
    );
}
