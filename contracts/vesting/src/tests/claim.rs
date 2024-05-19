use crate::{
    storage::{DistributionInfo, VestingSchedule, VestingTokenInfo},
    tests::setup::instantiate_vesting_client,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env, String,
};

use super::setup::deploy_token_contract;

#[test]
fn claim_tokens_when_fully_vested() {
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

    let vesting_schedules = vec![
        &env,
        VestingSchedule {
            recipient: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 0,
                end_timestamp: 60,
                amount: 120,
            },
        },
        VestingSchedule {
            recipient: Address::generate(&env),
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

    vesting_client.initialize(&admin, &vesting_token, &10u32);
    vesting_client.create_vesting_schedules(&vesting_schedules);

    // after initialization the admin has 0 vesting tokens
    // contract has 320 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 320);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time to the end of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 60);

    // user collects the vested tokens and transfers them to himself
    vesting_client.claim(&vester1);

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

    let vesting_schedules = vec![
        &env,
        VestingSchedule {
            recipient: vester1.clone(),
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

    vesting_client.initialize(&admin, &vesting_token, &10u32);
    vesting_client.create_vesting_schedules(&vesting_schedules);

    // after initialization the admin has 0 vesting tokens
    // contract has 120 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 120);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time to the middle of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 30);

    // user collects the vested tokens and transfers them to himself
    vesting_client.claim(&vester1);

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

    let vesting_schedules = vec![
        &env,
        VestingSchedule {
            recipient: vester1.clone(),
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

    vesting_client.initialize(&admin, &vesting_token, &10u32);
    vesting_client.create_vesting_schedules(&vesting_schedules);

    // after initialization the admin has 0 vesting tokens
    // contract has 120 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 120);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time to the middle of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 30);

    // user collects 1/2 of the vested tokens and transfers them to himself
    vesting_client.claim(&vester1);

    // vester1 has 60 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 60);

    // there must be 60 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 60);

    // we move time to the end of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 60);

    // user collects the remaining vested tokens and transfers them to himself
    vesting_client.claim(&vester1);

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

    let vesting_schedules = vec![
        &env,
        VestingSchedule {
            recipient: vester1.clone(),
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

    vesting_client.initialize(&admin, &vesting_token, &10u32);
    vesting_client.create_vesting_schedules(&vesting_schedules);

    // after initialization the admin has 0 vesting tokens
    // contract has 120 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 120);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time ahead in time
    env.ledger().with_mut(|li| li.timestamp = 61);

    // user collects everything
    vesting_client.claim(&vester1);

    // vester1 has 120 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 120);

    // there must be 0 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 0);
}

#[test]
#[should_panic(expected = "Vesting: Claim: No tokens available to claim")]
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

    let vesting_schedules = vec![
        &env,
        VestingSchedule {
            recipient: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: START_TIMESTAMP,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(&admin, &vesting_token, &10u32);
    vesting_client.create_vesting_schedules(&vesting_schedules);

    // we set the timestamp at a time earlier than the vesting period start
    env.ledger()
        .with_mut(|li| li.timestamp = START_TIMESTAMP - 10);

    // we try to claim the tokens before the vesting period has started
    vesting_client.claim(&vester1);
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

    let vesting_schedules = vec![
        &env,
        VestingSchedule {
            recipient: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 0,
                end_timestamp: 1_000,
                amount: 300,
            },
        },
        VestingSchedule {
            recipient: vester2.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 0,
                end_timestamp: 500,
                amount: 200,
            },
        },
        VestingSchedule {
            recipient: vester3.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 125,
                end_timestamp: 750,
                amount: 250,
            },
        },
        VestingSchedule {
            recipient: vester4.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 250,
                end_timestamp: 1_500,
                amount: 250,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(&admin, &vesting_token, &10u32);
    vesting_client.create_vesting_schedules(&vesting_schedules);

    // vesting period for our 4 vesters is between 0 and 1_500
    // we will move timestamp 3 times by 500 units and on each withdrawal we will transfer the vested amount

    env.ledger().with_mut(|li| li.timestamp = 500);

    // vester1 can withdraw 150 tokens out of 300 tokens
    assert_eq!(vesting_client.query_balance(&vester1), 0);
    vesting_client.claim(&vester1);
    assert_eq!(vesting_client.query_balance(&vester1), 150);

    // vester2 can withdraw all tokens
    assert_eq!(vesting_client.query_balance(&vester2), 0);
    vesting_client.claim(&vester2);
    assert_eq!(vesting_client.query_balance(&vester2), 200);

    // vester3 can withdraw 150 tokens out of 250 tokens
    assert_eq!(vesting_client.query_balance(&vester3), 0);
    vesting_client.claim(&vester3);
    assert_eq!(vesting_client.query_balance(&vester3), 150);

    // vester4 can withdraw 50 tokens out of 250 tokens
    assert_eq!(vesting_client.query_balance(&vester4), 0);
    vesting_client.claim(&vester4);
    assert_eq!(vesting_client.query_balance(&vester4), 50);

    // users have withdrawn a total of 550 tokens
    // total remaining in the contract is 450 tokens
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 450);

    // we now move the timestamp to 1_000
    env.ledger().with_mut(|li| li.timestamp = 1_000);

    // vester1 can withdraw the remaining 150 tokens
    assert_eq!(vesting_client.query_balance(&vester1), 150);
    vesting_client.claim(&vester1);
    assert_eq!(vesting_client.query_balance(&vester1), 300);

    // vester2 has nothing to withdraw
    // vester3 can withdraw the remaining 100 tokens
    assert_eq!(vesting_client.query_balance(&vester3), 150);
    vesting_client.claim(&vester3);
    assert_eq!(vesting_client.query_balance(&vester3), 250);

    // vester4 can withdraw 100 - maximum for the period
    assert_eq!(vesting_client.query_balance(&vester4), 50);
    vesting_client.claim(&vester4);
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
    vesting_client.claim(&vester4);
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

    let vesting_schedules = vec![
        &env,
        VestingSchedule {
            recipient: vester1.clone(),
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

    vesting_client.initialize(&admin, &vesting_token, &10u32);
    vesting_client.create_vesting_schedules(&vesting_schedules);

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
