use crate::{
    storage::{VestingInfo, VestingSchedule, VestingTokenInfo},
    tests::setup::instantiate_vesting_client,
};
use curve::{Curve, PiecewiseLinear, SaturatingLinear, Step};

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env, String,
};

use super::setup::deploy_token_contract;

#[test]
fn claim_tokens_when_fully_vested() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

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
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 0,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
        VestingSchedule {
            recipient: Address::generate(&env),
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 200,
                max_x: 60,
                max_y: 0,
            }),
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
    vesting_client.claim(&vester1, &0);

    // vester1 has 120 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 120);

    // there must be 200 vesting tokens left in the contract - remaining for the 2nd vester
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 200);
}

#[test]
fn transfer_tokens_when_half_vested() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

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
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 0,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
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
    vesting_client.claim(&vester1, &0);

    // vester1 has 60 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 60);

    // there must be 60 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 60);
}

#[test]
fn claim_tokens_once_then_claim_again() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

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
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 0,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

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
    vesting_client.claim(&vester1, &0);

    // vester1 has 60 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 60);

    // there must be 60 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 60);

    // we move time to the end of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 60);

    // user collects the remaining vested tokens and transfers them to himself
    vesting_client.claim(&vester1, &0);

    // vester1 has 120 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 120);

    // there must be 0 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 0);
}

#[test]
fn user_can_claim_tokens_way_after_the_testing_period() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

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
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 0,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
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
    vesting_client.claim(&vester1, &0);

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
    env.cost_estimate().budget().reset_unlimited();

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
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: START_TIMESTAMP,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(&admin, &vesting_token, &10u32);
    vesting_client.create_vesting_schedules(&vesting_schedules);

    // we set the timestamp at a time earlier than the vesting period start
    env.ledger()
        .with_mut(|li| li.timestamp = START_TIMESTAMP - 10);

    // we try to claim the tokens before the vesting period has started
    vesting_client.claim(&vester1, &0);
}

#[test]
#[should_panic(expected = "Vesting: Claim: No tokens available to claim")]
fn claim_after_all_tokens_have_been_claimed() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

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
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 0,
                min_y: 1_000,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(&admin, &vesting_token, &10u32);
    vesting_client.create_vesting_schedules(&vesting_schedules);

    env.ledger().with_mut(|li| li.timestamp = 61);

    // we claim tokens once
    vesting_client.claim(&vester1, &0);
    assert_eq!(vesting_client.query_balance(&vester1), 1_000);
    // and second one fails
    vesting_client.claim(&vester1, &0);
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
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 0,
                min_y: 300,
                max_x: 1_000,
                max_y: 0,
            }),
        },
        VestingSchedule {
            recipient: vester2.clone(),
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 0,
                min_y: 200,
                max_x: 500,
                max_y: 0,
            }),
        },
        VestingSchedule {
            recipient: vester3.clone(),
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 125,
                min_y: 250,
                max_x: 750,
                max_y: 0,
            }),
        },
        VestingSchedule {
            recipient: vester4.clone(),
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 250,
                min_y: 250,
                max_x: 1_500,
                max_y: 0,
            }),
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
    vesting_client.claim(&vester1, &0);
    assert_eq!(vesting_client.query_balance(&vester1), 150);

    // vester2 can withdraw all tokens
    assert_eq!(vesting_client.query_balance(&vester2), 0);
    vesting_client.claim(&vester2, &0);
    assert_eq!(vesting_client.query_balance(&vester2), 200);

    // vester3 can withdraw 150 tokens out of 250 tokens
    assert_eq!(vesting_client.query_balance(&vester3), 0);
    vesting_client.claim(&vester3, &0);
    assert_eq!(vesting_client.query_balance(&vester3), 150);

    // vester4 can withdraw 50 tokens out of 250 tokens
    assert_eq!(vesting_client.query_balance(&vester4), 0);
    vesting_client.claim(&vester4, &0);
    assert_eq!(vesting_client.query_balance(&vester4), 50);

    // users have withdrawn a total of 550 tokens
    // total remaining in the contract is 450 tokens
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 450);

    // we now move the timestamp to 1_000
    env.ledger().with_mut(|li| li.timestamp = 1_000);

    // vester1 can withdraw the remaining 150 tokens
    assert_eq!(vesting_client.query_balance(&vester1), 150);
    vesting_client.claim(&vester1, &0);
    assert_eq!(vesting_client.query_balance(&vester1), 300);

    // vester2 has nothing to withdraw
    // vester3 can withdraw the remaining 100 tokens
    assert_eq!(vesting_client.query_balance(&vester3), 150);
    vesting_client.claim(&vester3, &0);
    assert_eq!(vesting_client.query_balance(&vester3), 250);

    // vester4 can withdraw 100 - maximum for the period
    assert_eq!(vesting_client.query_balance(&vester4), 50);
    vesting_client.claim(&vester4, &0);
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
    vesting_client.claim(&vester4, &0);
    assert_eq!(vesting_client.query_balance(&vester4), 250);

    assert_eq!(vesting_client.query_balance(&vesting_client.address), 0);
}

#[test]
fn claim_works() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

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
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 0,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
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
    assert_eq!(vesting_client.query_available_to_claim(&vester1, &0), 0);

    // we move time to half of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 30);

    // vester1 claims all available for claiming tokens
    vesting_client.claim(&vester1, &0);

    // vester1 has 60 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 60);
    // vester1 has 0 tokens available for claiming after claiming the vested amount
    assert_eq!(vesting_client.query_available_to_claim(&vester1, &0), 0);

    // there must be 60 vesting tokens left in the contract - remaining for the 2nd vester
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 60);

    // we move time to the end of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 60);

    // vester1 claims the remaining tokens
    vesting_client.claim(&vester1, &0);

    // vester1 has 120 tokens after claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 120);
    // vester1 has 0 tokens available for claiming after claiming the vested amount
    assert_eq!(vesting_client.query_available_to_claim(&vester1, &0), 0);

    // there must be 0 vesting tokens left in the contract
    assert_eq!(vesting_client.query_balance(&vesting_client.address), 0);
}

#[test]
fn claim_tokens_from_two_distributions() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let token_client = deploy_token_contract(&env, &admin);

    token_client.mint(&admin, &2_000);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token_client.address.clone(),
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(&admin, &vesting_token, &10u32);

    let vesting_schedules = vec![
        &env,
        VestingSchedule {
            recipient: vester1.clone(),
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 0,
                min_y: 1_500,
                max_x: 100,
                max_y: 0,
            }),
        },
    ];
    vesting_client.create_vesting_schedules(&vesting_schedules);
    assert_eq!(token_client.balance(&vesting_client.address), 1_500);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time to the half of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 50);

    // user collects the vested tokens and transfers them to himself
    vesting_client.claim(&vester1, &0);

    assert_eq!(vesting_client.query_balance(&vester1), 750);
    assert_eq!(token_client.balance(&vesting_client.address), 750);

    // create a vesting schedule which starts in the middle of the previous one
    let vesting_schedules = vec![
        &env,
        VestingSchedule {
            recipient: vester1.clone(),
            curve: Curve::PiecewiseLinear(PiecewiseLinear {
                steps: vec![
                    &env,
                    Step {
                        time: 50,
                        value: 500,
                    },
                    Step {
                        time: 100,
                        value: 250,
                    },
                    Step {
                        time: 150,
                        value: 0,
                    },
                ],
            }),
        },
    ];
    vesting_client.create_vesting_schedules(&vesting_schedules);

    assert_eq!(
        vesting_client.query_all_vesting_info(&vester1),
        vec![
            &env,
            VestingInfo {
                recipient: vester1.clone(),
                balance: 750, // balance is deducted because it was already once claimed
                schedule: Curve::SaturatingLinear(SaturatingLinear {
                    min_x: 0,
                    min_y: 1_500,
                    max_x: 100,
                    max_y: 0,
                })
            },
            VestingInfo {
                recipient: vester1.clone(),
                balance: 500,
                schedule: Curve::PiecewiseLinear(PiecewiseLinear {
                    steps: vec![
                        &env,
                        Step {
                            time: 50,
                            value: 500,
                        },
                        Step {
                            time: 100,
                            value: 250,
                        },
                        Step {
                            time: 150,
                            value: 0,
                        },
                    ],
                })
            }
        ]
    );
    // we move time to the half of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 100);

    vesting_client.claim(&vester1, &0);
    assert_eq!(vesting_client.query_balance(&vester1), 1_500);
    assert_eq!(token_client.balance(&vesting_client.address), 500);

    vesting_client.claim(&vester1, &1);
    assert_eq!(vesting_client.query_balance(&vester1), 1_750);
    assert_eq!(token_client.balance(&vesting_client.address), 250);

    env.ledger().with_mut(|li| li.timestamp = 150);
    vesting_client.claim(&vester1, &1);
    assert_eq!(vesting_client.query_balance(&vester1), 2_000);
    assert_eq!(token_client.balance(&vesting_client.address), 0);
}

#[test]
fn first_mainnet_simulation() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let token_client = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 7,
        address: token_client.address.clone(),
    };
    token_client.mint(&admin, &100_000_000);

    let vesting_schedules = vec![
        &env,
        VestingSchedule {
            recipient: vester1.clone(),
            curve: Curve::SaturatingLinear(SaturatingLinear {
                // 1h schedule between 13:40 and 14:40
                min_x: 1716817200,
                min_y: 100000000,
                max_x: 1716820800,
                max_y: 0,
            }),
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(&admin, &vesting_token, &10u32);

    // we move time to the beginning of the vesting schedule (100s before)
    env.ledger().with_mut(|li| li.timestamp = 1716817100);
    vesting_client.create_vesting_schedules(&vesting_schedules);

    // after initialization the admin has 0 vesting tokens
    // contract has 120 vesting tokens
    assert_eq!(token_client.balance(&admin), 0);
    assert_eq!(token_client.balance(&vesting_client.address), 100_000_000);

    // vester1 has 0 tokens before claiming the vested amount
    assert_eq!(vesting_client.query_balance(&vester1), 0);

    // we move time 20 minutes into the future, 1/3 of the schedule
    env.ledger().with_mut(|li| li.timestamp = 1716818400);

    // user collects 1/3 of the vested tokens and transfers them to himself
    vesting_client.claim(&vester1, &0);

    assert_eq!(
        vesting_client.query_vesting_info(&vester1, &0),
        VestingInfo {
            recipient: vester1.clone(),
            balance: 66_666_667,
            schedule: Curve::SaturatingLinear(SaturatingLinear {
                // 1h schedule between 13:40 and 14:40
                min_x: 1716817200,
                min_y: 100000000,
                max_x: 1716820800,
                max_y: 0,
            }),
        },
    );

    assert_eq!(vesting_client.query_balance(&vester1), 33_333_333);
    assert_eq!(token_client.balance(&vesting_client.address), 66_666_667);
    assert_eq!(
        vesting_client.query_balance(&vesting_client.address),
        66_666_667
    );

    // we move time to the end of the vesting period
    env.ledger().with_mut(|li| li.timestamp = 1716820801);

    // user collects the remaining vested tokens and transfers them to himself
    vesting_client.claim(&vester1, &0);

    assert_eq!(vesting_client.query_balance(&vester1), 100_000_000);

    assert_eq!(vesting_client.query_balance(&vesting_client.address), 0);
}
