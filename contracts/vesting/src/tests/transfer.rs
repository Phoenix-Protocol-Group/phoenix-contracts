use crate::{
    storage::{DistributionInfo, VestingBalance, VestingTokenInfo},
    tests::setup::instantiate_vesting_client,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env, String,
};

use super::setup::deploy_token_contract;

#[test]
fn transfer_tokens_when_fully_vested() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
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
            distribution_info: DistributionInfo {
                start_timestamp: 0,
                end_timestamp: 60,
                amount: 120,
            },
        },
        VestingBalance {
            rcpt_address: Address::generate(&env),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 200,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    // admin has 1_000 vesting tokens prior to initializing the contract
    assert_eq!(token_client.balance(&admin), 1_000);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    // after initialization the admin has 0 vesting tokens
    // contract has 1_000 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 1_000);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time to the end of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 60);

    // user collects the vested tokens and transfers them to himself
    vesting_client.transfer_token(&vester1, &vester1, &120);

    // vester1 has 120 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 120);

    // there must be 880 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 880);
}

#[test]
fn transfer_tokens_when_half_vested() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
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
            distribution_info: DistributionInfo {
                start_timestamp: 0,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    // admin has 1_000 vesting tokens prior to initializing the contract
    assert_eq!(token_client.balance(&admin), 1_000);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    // after initialization the admin has 0 vesting tokens
    // contract has 1_000 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 1_000);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time to the middle of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 30);

    // user collects the vested tokens and transfers them to himself
    vesting_client.transfer_token(&vester1, &vester1, &60);

    // vester1 has 60 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 60);

    // there must be 940 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 940);
}

#[test]
fn test_claim_tokens_once_then_claim_again() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
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
            distribution_info: DistributionInfo {
                start_timestamp: 0,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    // admin has 1_000 vesting tokens prior to initializing the contract
    assert_eq!(token_client.balance(&admin), 1_000);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    // after initialization the admin has 0 vesting tokens
    // contract has 1_000 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 1_000);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time to the middle of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 30);

    // user collects 1/2 of the vested tokens and transfers them to himself
    vesting_client.transfer_token(&vester1, &vester1, &60);

    // vester1 has 60 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 60);

    // there must be 940 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 940);

    // we move time to the end of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 60);

    // user collects the remaining vested tokens and transfers them to himself
    vesting_client.transfer_token(&vester1, &vester1, &60);

    // vester1 has 120 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 120);

    // there must be 880 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 880);
}

#[test]
fn test_user_can_claim_tokens_way_after_the_testing_period() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
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
            distribution_info: DistributionInfo {
                start_timestamp: 0,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    // admin has 1_000 vesting tokens prior to initializing the contract
    assert_eq!(token_client.balance(&admin), 1_000);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    // after initialization the admin has 0 vesting tokens
    // contract has 1_000 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 1_000);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time way ahead in time
    env.ledger().with_mut(|li| li.timestamp = 1000);

    // user collects everything
    vesting_client.transfer_token(&vester1, &vester1, &120);

    // vester1 has 120 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 120);

    // there must be 880 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 880);
}

#[test]
fn user_claims_only_a_part_of_the_allowed_vested_amount_then_claims_the_remaining_afterwards() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
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
            distribution_info: DistributionInfo {
                start_timestamp: 0,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    // admin has 1_000 vesting tokens prior to initializing the contract
    assert_eq!(token_client.balance(&admin), 1_000);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    // after initialization the admin has 0 vesting tokens
    // contract has 1_000 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 1_000);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time to the middle of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 30);

    // user can collect 30 tokens, but he only collects 15
    vesting_client.transfer_token(&vester1, &vester1, &15);

    // vester1 has 15 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 15);

    // there must be 985 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 985);

    // we move the time to the end of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 60);

    // user collects 15 more tokens
    vesting_client.transfer_token(&vester1, &vester1, &15);

    // vester1 has 30 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 30);

    // there must be 970 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 970);

    // we move time way ahead in time
    env.ledger().with_mut(|li| li.timestamp = 1000);

    // user decides it's times to become milionaire and collects the remaining 90 tokens
    vesting_client.transfer_token(&vester1, &vester1, &90);

    // vester1 has 120 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 120);

    // there must be 880 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 880);
}

#[test]
#[should_panic(
    expected = "Vesting: Verify Vesting Update Balances: Remaining amount must be at least equal to vested amount"
)]
fn transfer_vesting_token_before_vesting_period_starts_should_fail() {
    const START_TIMESTAMP: u64 = 15;
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
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
            distribution_info: DistributionInfo {
                start_timestamp: START_TIMESTAMP,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    // we set the timestamp at a time earlier than the vesting period start
    env.ledger()
        .with_mut(|li| li.timestamp = START_TIMESTAMP - 10);

    // we try to transfer the tokens before the vesting period has started
    vesting_client.transfer_token(&vester1, &vester1, &120);
}

#[test]
#[should_panic(expected = "Vesting: Transfer token: Invalid transfer amount")]
fn transfer_tokens_should_fail_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);
    token_client.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token_client.address,
        total_supply: 120,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    vesting_client.transfer_token(&vester1, &vester1, &0);
}
