use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env, String,
};

use crate::{
    storage::{DistributionInfo, VestingBalance, VestingTokenInfo},
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
    let token_client = deploy_token_contract(&env, &admin);

    token_client.mint(&admin, &1_000);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token_client.address.clone(),
        total_supply: 1_000,
    };

    let vesting_balances = vec![
        &env,
        VestingBalance {
            rcpt_address: vester1.clone(),
            balance: 1_000,
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    soroban_sdk::testutils::arbitrary::std::dbg!(env.ledger().timestamp());
    vesting_client.collect_vesting(&vester1, &vester2, &100);
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
            rcpt_address: vester1.clone(),
            balance: 200,
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    vesting_client.collect_vesting(&vester1, &vester2, &0);
}

#[test]
fn verify_vesting_works() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let rcpt1 = Address::generate(&env);
    let rcpt2 = Address::generate(&env);
    let rcpt3 = Address::generate(&env);
    let _rcpt4 = Address::generate(&env);
    let token = deploy_token_contract(&env, &admin);

    token.mint(&admin, &1_000);

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
            rcpt_address: rcpt1.clone(),
            balance: 200,
            distribution_info: DistributionInfo {
                start_timestamp: 15, //TODO start from 0; make a 2nd test starting from 15 to validate that no user can withdraw earlier than the starting period
                end_timestamp: 60,
                amount: 200,
            },
        },
        VestingBalance {
            rcpt_address: rcpt3.clone(),
            balance: 400,
            distribution_info: DistributionInfo {
                start_timestamp: 30,
                end_timestamp: 120,
                amount: 400,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);
    // we set the timestamp at beginning of time
    env.ledger().with_mut(|li| li.timestamp = 60);
    soroban_sdk::testutils::arbitrary::std::dbg!(env.ledger().timestamp());

    // we try to transfer the tokens before the vesting period has started
    let vest1_before = vesting_client.query_balance(&rcpt1);
    soroban_sdk::testutils::arbitrary::std::dbg!(vest1_before);
    vesting_client.collect_vesting(&rcpt1, &rcpt2, &200);
    let reslt = vesting_client.query_balance(&rcpt2);
    soroban_sdk::testutils::arbitrary::std::dbg!(reslt);
}
