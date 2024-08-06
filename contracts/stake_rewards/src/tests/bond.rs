use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    vec, Address, Env, IntoVal, Val, Vec,
};

use super::setup::{deploy_staking_rewards_contract, deploy_token_contract};
use crate::storage::{BondingInfo, Stake};

#[test]
fn initialize_staking_rewards_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let reward_token = deploy_token_contract(&env, &admin);
    let staking = Address::generate(&env);

    let staking_rewards =
        deploy_staking_rewards_contract(&env, &admin, &reward_token.address, &staking);

    assert_eq!(staking_rewards.query_admin(), admin);
    assert_eq!(
        staking_rewards.query_config().config.staking_contract,
        staking
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn calculate_bond_called_by_anyone() {
    let env = Env::default();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);
    let staking = Address::generate(&env);

    let staking_rewards =
        deploy_staking_rewards_contract(&env, &admin, &reward_token.address, &staking);

    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    assert_eq!(lp_token.balance(&user1), 10_000);

    // if staking rewards is not called by staking contract, authorization will fail
    staking_rewards.calculate_bond(
        &user1,
        &BondingInfo {
            stakes: vec![
                &env,
                Stake {
                    stake: 10_000,
                    stake_timestamp: 0,
                },
            ],
            reward_debt: 0,
            last_reward_time: 0,
            total_stake: 10_000,
        },
    );
}

#[test]
#[ignore = "Figure out how to assert two authentication (user and contract) in the same assertion..."]
fn calculate_bond_called_by_staking_contract() {
    let env = Env::default();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);
    let staking = Address::generate(&env);

    let staking_rewards =
        deploy_staking_rewards_contract(&env, &admin, &reward_token.address, &staking);

    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    assert_eq!(lp_token.balance(&user1), 10_000);

    let bonding_info = BondingInfo {
        stakes: vec![
            &env,
            Stake {
                stake: 10_000,
                stake_timestamp: 0,
            },
        ],
        reward_debt: 0,
        last_reward_time: 0,
        total_stake: 10_000,
    };

    let bond_fn_arg: Vec<Val> = (user1.clone(), bonding_info.clone()).into_val(&env);
    staking_rewards
        .mock_auths(&[MockAuth {
            address: &staking,
            invoke: &MockAuthInvoke {
                contract: &staking_rewards.address,
                fn_name: "calculate_bond",
                args: bond_fn_arg,
                sub_invokes: &[],
            },
        }])
        .calculate_bond(&user1, &bonding_info);
}
