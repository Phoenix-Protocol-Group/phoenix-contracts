use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env, String,
};

use super::setup::{deploy_staking_contract, deploy_token_contract};

use crate::msg::{
    AnnualizedReward, AnnualizedRewardsResponse, WithdrawableReward, WithdrawableRewardsResponse,
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
                    reward_amount
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
                    reward_amount
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

#[test]
fn two_users_one_starts_after_distribution_begins() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user = Address::random(&env);
    let user2 = Address::random(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    staking.create_distribution_flow(&admin, &admin, &reward_token.address);

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    // first user bonds before distribution started
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

    env.ledger().with_mut(|li| {
        li.timestamp = 2_300;
    });
    staking.distribute_rewards();

    // at this points, since half of the time has passed and only one user is staking, he should have 50% of the rewards
    assert_eq!(
        staking.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 50_000
                }
            ]
        }
    );

    // user2 starts staking after the distribution has begun
    lp_token.mint(&user2, &1000);
    staking.bond(&user2, &1000);

    env.ledger().with_mut(|li| {
        li.timestamp = 2_600;
    });
    staking.distribute_rewards();

    // first user should get 75_000, second user 25_000 since he joined at the half time
    assert_eq!(
        staking.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 75_000
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
                    reward_amount: 25_000
                }
            ]
        }
    );

    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), 75_000);
    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 25_000);
}

#[test]
fn two_users_both_bonds_after_distribution_starts() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user = Address::random(&env);
    let user2 = Address::random(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    staking.create_distribution_flow(&admin, &admin, &reward_token.address);

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

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
        li.timestamp = 2_200;
    });
    lp_token.mint(&user, &1000);
    staking.bond(&user, &1000);

    staking.distribute_rewards();

    // at this points, since half of the time has passed and only one user is staking, he should have 50% of the rewards
    assert_eq!(
        staking.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 33_332
                }
            ]
        }
    );

    // user2 starts staking after the distribution has begun
    env.ledger().with_mut(|li| {
        li.timestamp = 2_400;
    });
    lp_token.mint(&user2, &1000);
    staking.bond(&user2, &1000);

    staking.distribute_rewards();
    assert_eq!(
        staking.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 49_999
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
                    reward_amount: 16_666
                }
            ]
        }
    );

    env.ledger().with_mut(|li| {
        li.timestamp = 2_600;
    });
    staking.distribute_rewards();

    // first user should get 75_000, second user 25_000 since he joined at the half time
    assert_eq!(
        staking.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 66_666
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
                    reward_amount: 33_333
                }
            ]
        }
    );

    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), 66_666);
    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 33_333);
}

#[test]
#[should_panic(expected = "Stake: Fund distribution: Not reward curve exists")]
fn fund_rewards_without_establishing_distribution() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    reward_token.mint(&admin, &1000);

    staking.fund_distribution(&admin, &2_000, &600, &reward_token.address, &1000);
}

#[test]
fn try_to_withdraw_rewards_without_bonding() {
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
    assert_eq!(
        staking.query_undistributed_rewards(&reward_token.address),
        reward_amount
    );
    assert_eq!(staking.query_distributed_rewards(&reward_token.address), 0);

    assert_eq!(
        staking.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 0
                }
            ]
        }
    );

    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), 0);
}

#[test]
#[should_panic(expected = "Stake: Fund distribution: Fund distribution start time is too early")]
fn fund_distribution_starting_before_current_timestamp() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    staking.create_distribution_flow(&admin, &admin, &reward_token.address);

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    env.ledger().with_mut(|li| {
        li.timestamp = 2_000;
    });

    let reward_duration = 600;
    staking.fund_distribution(
        &admin,
        &1_999,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    )
}

#[test]
#[should_panic(expected = "Stake: Fund distribution: minimum reward amount not reached")]
fn fund_distribution_with_reward_below_required_minimum() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    staking.create_distribution_flow(&admin, &admin, &reward_token.address);

    reward_token.mint(&admin, &10);

    env.ledger().with_mut(|li| {
        li.timestamp = 2_000;
    });

    let reward_duration = 600;
    staking.fund_distribution(&admin, &2_000, &reward_duration, &reward_token.address, &10);
}

#[test]
fn calculate_apr() {
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

    env.ledger().with_mut(|li| {
        li.timestamp = 0;
    });

    // whole year of distribution
    let reward_duration = 60 * 60 * 24 * 365;
    staking.fund_distribution(
        &admin,
        &2_000,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );

    // nothing bonded, no rewards
    assert_eq!(
        staking.query_annualized_rewards(),
        AnnualizedRewardsResponse {
            rewards: vec![
                &env,
                AnnualizedReward {
                    asset: reward_token.address.clone(),
                    amount: String::from_slice(&env, "0")
                }
            ]
        }
    );

    // bond tokens for user to enable distribution for him
    lp_token.mint(&user, &1000);
    staking.bond(&user, &1000);

    // 100k rewards distributed for the whole year gives 100% APR
    assert_eq!(
        staking.query_annualized_rewards(),
        AnnualizedRewardsResponse {
            rewards: vec![
                &env,
                AnnualizedReward {
                    asset: reward_token.address.clone(),
                    amount: String::from_slice(&env, "100")
                }
            ]
        }
    );

    let reward_amount: u128 = 50_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    staking.fund_distribution(
        &admin,
        &2_000,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );

    // having another 50k in rewards increases APR
    assert_eq!(
        staking.query_annualized_rewards(),
        AnnualizedRewardsResponse {
            rewards: vec![
                &env,
                AnnualizedReward {
                    asset: reward_token.address.clone(),
                    amount: String::from_slice(&env, "150")
                }
            ]
        }
    );
}
