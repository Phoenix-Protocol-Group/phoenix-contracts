use pretty_assertions::assert_eq;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env,
};

use super::setup::{deploy_staking_contract, deploy_token_contract};

use crate::error::ContractError::{StakeLessThenMinBond, StakeNotFound};
use crate::{
    msg::{WithdrawableReward, WithdrawableRewardsResponse},
    storage::{Config, Stake},
};

#[test]
fn add_distribution_and_distribute_reward() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user = Address::random(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    staking.create_distribution_flow(&admin, &admin, &reward_token.address);

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    // bond tokens for user to enable distribution for him
    lp_token.mint(&user, &1000);
    staking.bond(&user, &1000);

    env.ledger().with_mut(|li| {
        li.timestamp = 2_000;
    });

    let reward_duration = 600;
    staking.fund_distribution(
        &admin,
        &2_000,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );

    staking.distribute_rewards();
    assert_eq!(
        staking.query_undistributed_rewards(&reward_token.address),
        reward_amount
    );

    env.ledger().with_mut(|li| {
        li.timestamp = 2_600;
    });
    staking.distribute_rewards();
    assert_eq!(
        staking.query_undistributed_rewards(&reward_token.address),
        0
    );
    assert_eq!(
        staking.query_distributed_rewards(&reward_token.address),
        reward_amount
    );

    assert_eq!(
        staking.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount
                }
            ]
        }
    );

    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), reward_amount as i128);
}

#[test]
fn two_distributions() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user = Address::random(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);
    let reward_token_2 = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    staking.create_distribution_flow(&admin, &admin, &reward_token.address);
    staking.create_distribution_flow(&admin, &admin, &reward_token_2.address);

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));
    reward_token_2.mint(&admin, &((reward_amount * 2) as i128));

    // bond tokens for user to enable distribution for him
    lp_token.mint(&user, &1000);
    staking.bond(&user, &1000);

    env.ledger().with_mut(|li| {
        li.timestamp = 2_000;
    });

    let reward_duration = 600;
    staking.fund_distribution(
        &admin,
        &2_000,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );
    staking.fund_distribution(
        &admin,
        &2_000,
        &reward_duration,
        &reward_token_2.address,
        &((reward_amount * 2) as i128),
    );

    // distribute rewards during half time
    env.ledger().with_mut(|li| {
        li.timestamp = 2_300;
    });
    staking.distribute_rewards();
    assert_eq!(
        staking.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: reward_amount / 2
                },
                WithdrawableReward {
                    reward_address: reward_token_2.address.clone(),
                    reward_amount: reward_amount
                }
            ]
        }
    );
    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), (reward_amount / 2) as i128);
    assert_eq!(reward_token_2.balance(&user), reward_amount as i128);

    env.ledger().with_mut(|li| {
        li.timestamp = 2_600;
    });
    staking.distribute_rewards();
    // first reward token
    assert_eq!(
        staking.query_undistributed_rewards(&reward_token.address),
        0
    );
    assert_eq!(
        staking.query_distributed_rewards(&reward_token.address),
        reward_amount
    );
    // second reward token
    assert_eq!(
        staking.query_undistributed_rewards(&reward_token_2.address),
        0
    );
    assert_eq!(
        staking.query_distributed_rewards(&reward_token_2.address),
        reward_amount * 2
    );

    // since half of rewards were already distributed, after full distirubtion
    // round another half is ready
    assert_eq!(
        staking.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: reward_amount / 2
                },
                WithdrawableReward {
                    reward_address: reward_token_2.address.clone(),
                    reward_amount: reward_amount
                }
            ]
        }
    );

    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), reward_amount as i128);
    assert_eq!(reward_token_2.balance(&user), (reward_amount * 2) as i128);
}

#[test]
fn four_users_with_different_stakes() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user = Address::random(&env);
    let user2 = Address::random(&env);
    let user3 = Address::random(&env);
    let user4 = Address::random(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    staking.create_distribution_flow(&admin, &admin, &reward_token.address);

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    // bond tokens for users; each user has a different amount staked
    lp_token.mint(&user, &1000);
    staking.bond(&user, &1000);
    lp_token.mint(&user2, &2000);
    staking.bond(&user2, &2000);
    lp_token.mint(&user3, &3000);
    staking.bond(&user3, &3000);
    lp_token.mint(&user4, &4000);
    staking.bond(&user4, &4000);

    env.ledger().with_mut(|li| {
        li.timestamp = 2_000;
    });

    let reward_duration = 600;
    staking.fund_distribution(
        &admin,
        &2_000,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );

    env.ledger().with_mut(|li| {
        li.timestamp = 2_600;
    });
    staking.distribute_rewards();

    // total staked amount is 10_000
    // user1 should have 10% of the rewards, user2 20%, user3 30%, user4 40%
    assert_eq!(
        staking.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 10_000
                }
            ]
        }
    );
    assert_eq!(
        staking.query_withdrawable_rewards(&user2),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 20_000
                }
            ]
        }
    );
    assert_eq!(
        staking.query_withdrawable_rewards(&user3),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 30_000
                }
            ]
        }
    );
    assert_eq!(
        staking.query_withdrawable_rewards(&user4),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 40_000
                }
            ]
        }
    );

    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), 10_000);
    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 20_000);
    staking.withdraw_rewards(&user3);
    assert_eq!(reward_token.balance(&user3), 30_000);
    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 40_000);
}
