use pretty_assertions::assert_eq;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env,
};

use super::setup::{deploy_staking_contract, deploy_token_contract};

use crate::{
    msg::ConfigResponse,
    storage::{Config, Stake},
};

#[test]
fn initializa_staking_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address, &manager, &owner);

    let response = staking.query_config();
    assert_eq!(
        response,
        ConfigResponse {
            config: Config {
                lp_token: lp_token.address,
                min_bond: 1_000i128,
                min_reward: 1_000i128,
                manager,
                owner,
            }
        }
    );

    let response = staking.query_admin();
    assert_eq!(response, admin);
}

#[test]
#[should_panic(expected = "Stake: Initialize: initializing contract twice is not allowed")]
fn test_deploying_stake_twice_should_fail() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    let first = deploy_staking_contract(&env, admin.clone(), &lp_token.address, &manager, &owner);

    first.initialize(
        &admin,
        &lp_token.address,
        &100i128,
        &50i128,
        &manager,
        &owner,
    );
}

#[test]
#[should_panic = "Stake: Bond: Trying to stake less than minimum required"]
fn bond_too_few() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address, &manager, &owner);

    lp_token.mint(&user, &999);

    staking.bond(&user, &999);
}

#[test]
fn bond_simple() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address, &manager, &owner);

    lp_token.mint(&user, &10_000);

    staking.bond(&user, &10_000);

    let bonds = staking.query_staked(&user).stakes;
    assert_eq!(
        bonds,
        vec![
            &env,
            Stake {
                stake: 10_000,
                stake_timestamp: 0,
            }
        ]
    );
    assert_eq!(staking.query_total_staked(), 10_000);

    assert_eq!(lp_token.balance(&user), 0);
    assert_eq!(lp_token.balance(&staking.address), 10_000);
}

#[test]
fn unbond_simple() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let user2 = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address, &manager, &owner);

    lp_token.mint(&user, &35_000);
    lp_token.mint(&user2, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp = 2000;
    });
    staking.bond(&user, &10_000);
    env.ledger().with_mut(|li| {
        li.timestamp = 4000;
    });
    staking.bond(&user, &10_000);
    staking.bond(&user2, &10_000);
    env.ledger().with_mut(|li| {
        li.timestamp = 4000;
    });
    staking.bond(&user, &15_000);

    assert_eq!(staking.query_staked(&user).stakes.len(), 3);
    assert_eq!(lp_token.balance(&user), 0);
    assert_eq!(lp_token.balance(&staking.address), 45_000);

    let stake_timestamp = 4000;
    staking.unbond(&user, &10_000, &stake_timestamp);

    let bonds = staking.query_staked(&user).stakes;
    assert_eq!(
        bonds,
        vec![
            &env,
            Stake {
                stake: 10_000,
                stake_timestamp: 2_000,
            },
            Stake {
                stake: 15_000,
                stake_timestamp: 4_000,
            }
        ]
    );
    assert_eq!(staking.query_total_staked(), 35_000);

    assert_eq!(lp_token.balance(&user), 10_000);
    assert_eq!(lp_token.balance(&user2), 0);
    assert_eq!(lp_token.balance(&staking.address), 35_000);
}

#[test]
fn initializing_contract_sets_total_staked_var() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address, &manager, &owner);

    assert_eq!(staking.query_total_staked(), 0);
}

#[test]
#[should_panic(expected = "Stake: Remove stake: Stake not found")]
fn unbond_wrong_user_stake_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let user2 = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address, &manager, &owner);

    lp_token.mint(&user, &35_000);
    lp_token.mint(&user2, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp = 2_000;
    });
    staking.bond(&user, &10_000);
    env.ledger().with_mut(|li| {
        li.timestamp = 4_000;
    });
    staking.bond(&user, &10_000);
    staking.bond(&user2, &10_000);

    assert_eq!(lp_token.balance(&user), 15_000);
    assert_eq!(lp_token.balance(&user2), 0);
    assert_eq!(lp_token.balance(&staking.address), 30_000);

    staking.unbond(&user2, &10_000, &2_000);
}

#[test]
fn pay_rewards_during_unbond() {
    const STAKED_AMOUNT: i128 = 1_000;
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);
    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address, &manager, &owner);

    lp_token.mint(&user, &10_000);
    reward_token.mint(&admin, &10_000);

    staking.create_distribution_flow(&manager, &reward_token.address);
    staking.fund_distribution(&admin, &0u64, &10_000u64, &reward_token.address, &10_000);

    staking.bond(&user, &STAKED_AMOUNT);

    env.ledger().with_mut(|li| {
        li.timestamp = 5_000;
    });
    staking.distribute_rewards();

    // user has bonded for 5_000 time, initial rewards are 10_000
    // so user should have 5_000 rewards
    // 5_000 rewards are still undistributed
    assert_eq!(
        staking.query_undistributed_rewards(&reward_token.address),
        5_000
    );
    assert_eq!(
        staking
            .query_withdrawable_rewards(&user)
            .rewards
            .iter()
            .map(|reward| reward.reward_amount)
            .sum::<u128>(),
        5_000
    );

    assert_eq!(reward_token.balance(&user), 0);
    staking.unbond(&user, &STAKED_AMOUNT, &0);
    assert_eq!(reward_token.balance(&user), 5_000);
}
