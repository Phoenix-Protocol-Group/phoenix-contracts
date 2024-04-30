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

    token_client.mint(&admin, &320);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token_client.address.clone(),
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

    // admin has 320 vesting tokens prior to initializing the contract
    assert_eq!(token_client.balance(&admin), 320);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    // after initialization the admin has 0 vesting tokens
    // contract has 320 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 320);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time to the end of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 60);

    // user collects the vested tokens and transfers them to himself
    vesting_client.transfer_token(&vester1, &vester1, &120);

    // vester1 has 120 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 120);

    // there must be 200 vesting tokens left in the contract - remaining for the 2nd vester
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 200);
}

#[test]
fn transfer_tokens_when_half_vested() {
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
        address: token_client.address.clone(),
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

    // admin has 120 vesting tokens prior to initializing the contract
    assert_eq!(token_client.balance(&admin), 120);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    // after initialization the admin has 0 vesting tokens
    // contract has 120 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 120);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time to the middle of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 30);

    // user is greedy and tries to transfer more than he can
    let result = vesting_client.try_transfer_token(&vester1, &vester1, &61);
    assert!(result.is_err());

    // user collects the vested tokens and transfers them to himself
    vesting_client.transfer_token(&vester1, &vester1, &60);

    // vester1 has 60 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 60);

    // there must be 60 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 60);
}

#[test]
fn test_claim_tokens_once_then_claim_again() {
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
        address: token_client.address.clone(),
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

    // admin has 120 vesting tokens prior to initializing the contract
    assert_eq!(token_client.balance(&admin), 120);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    // after initialization the admin has 0 vesting tokens
    // contract has 120 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 120);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time to the middle of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 30);

    // user collects 1/2 of the vested tokens and transfers them to himself
    vesting_client.transfer_token(&vester1, &vester1, &60);

    // vester1 has 60 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 60);

    // there must be 60 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 60);

    // we move time to the end of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 60);

    // user collects the remaining vested tokens and transfers them to himself
    vesting_client.transfer_token(&vester1, &vester1, &60);

    // vester1 has 120 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 120);

    // there must be 0 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 0);
}

#[test]
fn test_user_can_claim_tokens_way_after_the_testing_period() {
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
        address: token_client.address.clone(),
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

    // admin has 120 vesting tokens prior to initializing the contract
    assert_eq!(token_client.balance(&admin), 120);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    // after initialization the admin has 0 vesting tokens
    // contract has 120 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 120);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time ahead in time
    env.ledger().with_mut(|li| li.timestamp = 61);

    // user collects everything
    vesting_client.transfer_token(&vester1, &vester1, &120);

    // vester1 has 120 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 120);

    // there must be 0 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 0);
}

#[test]
fn user_claims_only_a_part_of_the_allowed_vested_amount_then_claims_the_remaining_afterwards() {
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
        address: token_client.address.clone(),
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

    // admin has 120 vesting tokens prior to initializing the contract
    assert_eq!(token_client.balance(&admin), 120);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    // after initialization the admin has 0 vesting tokens
    // contract has 120 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 120);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time to the middle of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 30);

    // user can collect 30 tokens, but he only collects 15
    vesting_client.transfer_token(&vester1, &vester1, &15);

    // vester1 has 15 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 15);

    // there must be 105 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 105);

    // we move the time to the end of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 60);

    // user collects 15 more tokens
    vesting_client.transfer_token(&vester1, &vester1, &15);

    // vester1 has 30 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 30);

    // there must be 90 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 90);

    // we move time way ahead in time
    env.ledger().with_mut(|li| li.timestamp = 1000);

    // user decides it's times to become milionaire and collects the remaining 90 tokens
    vesting_client.transfer_token(&vester1, &vester1, &90);

    // vester1 has 120 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 120);

    // there must be 0 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 0);
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

#[test]
fn transfer_works_with_multiple_users_and_distributions() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let vester2 = Address::generate(&env);
    let vester3 = Address::generate(&env);
    let vester4 = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);
    token_client.mint(&admin, &1_000);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token_client.address.clone(),
    };

    let vesting_balances = vec![
        &env,
        VestingBalance {
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 0,
                end_timestamp: 1_000,
                amount: 300,
            },
        },
        VestingBalance {
            rcpt_address: vester2.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 0,
                end_timestamp: 500,
                amount: 200,
            },
        },
        VestingBalance {
            rcpt_address: vester3.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 125,
                end_timestamp: 750,
                amount: 250,
            },
        },
        VestingBalance {
            rcpt_address: vester4.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 250,
                end_timestamp: 1_500,
                amount: 250,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    // vesting period for our 4 vesters is between 0 and 1_500
    // we will move timestamp 3 times by 500 units and on each withdrawal we will transfer the vested amount

    env.ledger().with_mut(|li| li.timestamp = 500);

    // vester1 can withdraw 150 tokens out of 300 tokens
    assert_eq!(vesting_client.query_balance(&vester1), 0);
    vesting_client.transfer_token(&vester1, &vester1, &150);
    assert_eq!(vesting_client.query_balance(&vester1), 150);

    // vester2 can withdraw all tokens
    assert_eq!(vesting_client.query_balance(&vester2), 0);
    vesting_client.transfer_token(&vester2, &vester2, &200);
    assert_eq!(vesting_client.query_balance(&vester2), 200);

    // vester3 can withdraw 150 tokens out of 250 tokens
    assert_eq!(vesting_client.query_balance(&vester3), 0);
    vesting_client.transfer_token(&vester3, &vester3, &150);
    assert_eq!(vesting_client.query_balance(&vester3), 150);

    // vester4 can withdraw 50 tokens out of 250 tokens
    assert_eq!(vesting_client.query_balance(&vester4), 0);
    vesting_client.transfer_token(&vester4, &vester4, &50);
    assert_eq!(vesting_client.query_balance(&vester4), 50);

    // users have withdrawn a total of 550 tokens
    // total remaining in the contract is 450 tokens
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 450);

    // we now move the timestamp to 1_000
    env.ledger().with_mut(|li| li.timestamp = 1_000);

    // vester1 can withdraw the remaining 150 tokens
    assert_eq!(vesting_client.query_balance(&vester1), 150);
    vesting_client.transfer_token(&vester1, &vester1, &150);
    assert_eq!(vesting_client.query_balance(&vester1), 300);

    // vester2 has nothing to withdraw
    // vester3 can withdraw the remaining 100 tokens
    assert_eq!(vesting_client.query_balance(&vester3), 150);
    vesting_client.transfer_token(&vester3, &vester3, &100);
    assert_eq!(vesting_client.query_balance(&vester3), 250);

    // vester4 can withdraw 100 - maximum for the period
    assert_eq!(vesting_client.query_balance(&vester4), 50);
    vesting_client.transfer_token(&vester4, &vester4, &100);
    assert_eq!(vesting_client.query_balance(&vester4), 150);

    // in the 2nd round users have withdrawn 350 tokens
    // total remaining in the contract is 100 tokens

    // move the timestamp to 1_500
    env.ledger().with_mut(|li| li.timestamp = 1_500);

    // vester1 has nothing to withdraw
    // vester2 has nothing to withdraw
    // vester3 has nothing to withdraw
    // vester4 can withdraw the remaining 100 tokens
    assert_eq!(vesting_client.query_balance(&vester4), 150);
    vesting_client.transfer_token(&vester4, &vester4, &100);
    assert_eq!(vesting_client.query_balance(&vester4), 250);

    assert_eq!(vesting_client.query_balance(&vesting_client.address), 0);
}

#[test]
fn claim_works() {
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
        address: token_client.address.clone(),
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

    // admin has 120 vesting tokens prior to initializing the contract
    assert_eq!(token_client.balance(&admin), 120);

    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    // after initialization the admin has 0 vesting tokens
    // contract has 120 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 120);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);
    // vester1 has 0 tokens available for claiming before the vesting period starts
    assert_eq!(vesting_client.query_available_to_claim(&vester1), 0);

    // we move time to half of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 30);

    // vester1 claims all available for claiming tokens
    vesting_client.claim(&vester1);

    // vester1 has 60 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 60);
    // vester1 has 0 tokens available for claiming after claiming the vested amount
    assert_eq!(vesting_client.query_available_to_claim(&vester1), 0);

    // there must be 60 vesting tokens left in the contract - remaining for the 2nd vester
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 60);

    // we move time to the end of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 60);

    // vester1 claims the remaining tokens
    vesting_client.claim(&vester1);

    // vester1 has 120 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 120);
    // vester1 has 0 tokens available for claiming after claiming the vested amount
    assert_eq!(vesting_client.query_available_to_claim(&vester1), 0);

    // there must be 0 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 0);
}
