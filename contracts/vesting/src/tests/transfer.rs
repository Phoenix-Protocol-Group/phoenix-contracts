use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

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
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                min_value_at_start: 120,
                end_timestamp: 60,
                max_value_at_end: 0,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);
    assert_eq!(token.balance(&vester2), 0);
    vesting_client.transfer_token(&vester1, &vester2, &100);
    assert_eq!(vesting_client.query_balance(&vester1), 900);
    assert_eq!(token.balance(&vester2), 100);
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
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                min_value_at_start: 120,
                end_timestamp: 60,
                max_value_at_end: 0,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    vesting_client.transfer_token(&vester1, &vester2, &0);
}
