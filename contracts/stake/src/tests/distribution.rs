use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env, String,
};

use super::setup::{deploy_staking_contract, deploy_token_contract};
use pretty_assertions::assert_eq;

use crate::{
    msg::{
        AnnualizedReward, AnnualizedRewardsResponse, WithdrawableReward,
        WithdrawableRewardsResponse,
    },
    tests::setup::{ONE_DAY, ONE_WEEK},
};

#[test]
fn add_distribution_and_distribute_reward() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &50u32,
    );

    staking.create_distribution_flow(&manager, &reward_token.address);

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    // bond tokens for user to enable distribution for him
    lp_token.mint(&user, &1000);
    env.ledger().with_mut(|li| {
        li.timestamp = ONE_DAY;
    });

    staking.bond(&user, &1000);

    let reward_duration = 600;
    staking.fund_distribution(
        &admin,
        &ONE_DAY,
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
        li.timestamp = ONE_DAY + reward_duration;
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

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);
    let reward_token_2 = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &50u32,
    );

    staking.create_distribution_flow(&manager, &reward_token.address);
    staking.create_distribution_flow(&manager, &reward_token_2.address);

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));
    reward_token_2.mint(&admin, &((reward_amount * 2) as i128));

    // bond tokens for user to enable distribution for him
    lp_token.mint(&user, &1000);
    env.ledger().with_mut(|li| li.timestamp = ONE_DAY);
    staking.bond(&user, &1000);

    let reward_duration = 600;
    staking.fund_distribution(
        &admin,
        &ONE_DAY,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );
    staking.fund_distribution(
        &admin,
        &ONE_DAY,
        &reward_duration,
        &reward_token_2.address,
        &((reward_amount * 2) as i128),
    );

    // distribute rewards during half time
    env.ledger().with_mut(|li| {
        li.timestamp += 300;
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
        li.timestamp += 600;
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

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let user4 = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &50u32,
    );

    staking.create_distribution_flow(&manager, &reward_token.address);

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    // bond tokens for users; each user has a different amount staked
    env.ledger().with_mut(|li| {
        li.timestamp = ONE_WEEK;
    });

    lp_token.mint(&user, &1000);
    staking.bond(&user, &1000);
    lp_token.mint(&user2, &2000);
    staking.bond(&user2, &2000);
    lp_token.mint(&user3, &3000);
    staking.bond(&user3, &3000);
    lp_token.mint(&user4, &4000);
    staking.bond(&user4, &4000);

    let eight_days = ONE_WEEK + ONE_DAY;
    env.ledger().with_mut(|li| li.timestamp = eight_days);

    let reward_duration = 600;
    staking.fund_distribution(
        &admin,
        &eight_days,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );

    env.ledger().with_mut(|li| {
        li.timestamp = eight_days + 600;
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

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let user2 = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &50u32,
    );

    staking.create_distribution_flow(&manager, &reward_token.address);

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    // first user bonds before distribution started
    lp_token.mint(&user, &1000);
    env.ledger().with_mut(|li| li.timestamp = ONE_DAY);
    staking.bond(&user, &1000);

    let reward_duration = 600;
    staking.fund_distribution(
        &admin,
        &ONE_DAY,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );

    env.ledger().with_mut(|li| {
        li.timestamp += 300;
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
        li.timestamp += 300;
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

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let user2 = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &50u32,
    );

    staking.create_distribution_flow(&manager, &reward_token.address);

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    env.ledger().with_mut(|li| li.timestamp = ONE_DAY);

    let reward_duration = 600;
    staking.fund_distribution(
        &admin,
        &ONE_DAY,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );

    env.ledger().with_mut(|li| {
        li.timestamp += 200;
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
                    reward_amount: 33_333
                }
            ]
        }
    );

    // user2 starts staking after the distribution has begun
    env.ledger().with_mut(|li| {
        li.timestamp += 200;
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
        li.timestamp += 200;
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

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &50u32,
    );

    reward_token.mint(&admin, &1000);

    staking.fund_distribution(&admin, &2_000, &600, &reward_token.address, &1000);
}

#[test]
fn try_to_withdraw_rewards_without_bonding() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &50u32,
    );

    staking.create_distribution_flow(&manager, &reward_token.address);

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

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &50u32,
    );

    staking.create_distribution_flow(&manager, &reward_token.address);

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

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &50u32,
    );

    staking.create_distribution_flow(&manager, &reward_token.address);

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

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &50u32,
    );

    staking.create_distribution_flow(&manager, &reward_token.address);

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    env.ledger().with_mut(|li| {
        li.timestamp = ONE_DAY;
    });

    // whole year of distribution
    let reward_duration = 60 * 60 * 24 * 365;
    staking.fund_distribution(
        &admin,
        &ONE_DAY,
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
                    amount: String::from_str(&env, "0")
                }
            ]
        }
    );

    // bond tokens for user to enable distribution for him
    lp_token.mint(&user, &1000);
    env.ledger().with_mut(|li| {
        li.timestamp += ONE_DAY;
    });
    staking.bond(&user, &1000);

    // 100k rewards distributed for the whole year gives 100% APR
    assert_eq!(
        staking.query_annualized_rewards(),
        AnnualizedRewardsResponse {
            rewards: vec![
                &env,
                AnnualizedReward {
                    asset: reward_token.address.clone(),
                    amount: String::from_str(&env, "100000.975274725274725274")
                }
            ]
        }
    );

    let reward_amount: u128 = 50_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    staking.fund_distribution(
        &admin,
        &(2 * &ONE_DAY),
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
                    amount: String::from_str(&env, "149727")
                }
            ]
        }
    );
}

#[test]
#[should_panic(expected = "Stake: create distribution: Non-authorized creation!")]
fn add_distribution_should_fail_when_not_authorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &50u32,
    );

    staking.create_distribution_flow(&Address::generate(&env), &reward_token.address);
}

#[test]
fn test_v_phx_vul_010_unbond_breakes_reward_distribution() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user_1 = Address::generate(&env);
    let user_2 = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &50u32,
    );

    staking.create_distribution_flow(&manager, &reward_token.address);

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    // bond tokens for user to enable distribution for him
    lp_token.mint(&user_1, &1_000);
    lp_token.mint(&user_2, &1_000);

    env.ledger().with_mut(|li| li.timestamp = ONE_DAY);
    staking.bond(&user_1, &1_000);
    staking.bond(&user_2, &1_000);
    let reward_duration = 10_000;
    staking.fund_distribution(
        &admin,
        &ONE_DAY,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );

    env.ledger().with_mut(|li| {
        li.timestamp += 2_000;
    });

    staking.distribute_rewards();
    assert_eq!(
        staking.query_undistributed_rewards(&reward_token.address),
        80_000 // 100k total rewards, we have 2000 seconds passed, so we have 80k undistributed rewards
    );

    // at the 1/2 of the distribution time, user_1 unbonds
    env.ledger().with_mut(|li| {
        li.timestamp += 3_000;
    });
    staking.distribute_rewards();
    assert_eq!(
        staking.query_undistributed_rewards(&reward_token.address),
        50_000
    );

    // user1 unbonds, which automatically withdraws the rewards
    assert_eq!(
        staking.query_withdrawable_rewards(&user_1),
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
    staking.unbond(&user_1, &1_000, &ONE_DAY);
    assert_eq!(
        staking.query_withdrawable_rewards(&user_1),
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

    env.ledger().with_mut(|li| {
        li.timestamp += 10_000;
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
        staking.query_withdrawable_rewards(&user_2),
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

    staking.withdraw_rewards(&user_1);
    assert_eq!(reward_token.balance(&user_1), 25_000i128);
}

#[test]
fn test_bond_withdraw_unbond() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &50u32,
    );

    staking.create_distribution_flow(&manager, &reward_token.address);

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    lp_token.mint(&user, &1_000);
    env.ledger().with_mut(|li| li.timestamp = ONE_DAY);
    staking.bond(&user, &1_000);

    let reward_duration = 10_000;

    staking.fund_distribution(
        &admin,
        &ONE_DAY,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );

    env.ledger().with_mut(|li| {
        li.timestamp = ONE_DAY + reward_duration;
    });

    staking.distribute_rewards();

    staking.unbond(&user, &1_000, &ONE_DAY);

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
    // one more time to make sure that calculations during unbond aren't off
    staking.withdraw_rewards(&user);
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
}

#[should_panic(expected = "Stake: Add distribution: Distribution already added")]
#[test]
fn panic_when_adding_same_distribution_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &50u32,
    );

    staking.create_distribution_flow(&manager, &reward_token.address);
    staking.create_distribution_flow(&manager, &reward_token.address);
}

#[should_panic(expected = "Stake: Fund distribution: Curve complexity validation failed")]
#[test]
fn panic_when_funding_distribution_with_curve_too_complex() {
    const DISTRIBUTION_MAX_COMPLEXITY: u32 = 3;
    const FIVE_MINUTES: u64 = 300;
    const TEN_MINUTES: u64 = 600;
    const ONE_WEEK: u64 = 604_800;

    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &DISTRIBUTION_MAX_COMPLEXITY,
    );

    staking.create_distribution_flow(&manager, &reward_token.address);

    reward_token.mint(&admin, &3000);

    staking.fund_distribution(&admin, &0, &FIVE_MINUTES, &reward_token.address, &1000);
    staking.fund_distribution(
        &admin,
        &FIVE_MINUTES,
        &TEN_MINUTES,
        &reward_token.address,
        &1000,
    );

    // assert just to prove that we have 2 successful fund distributions
    assert_eq!(
        staking.query_undistributed_rewards(&reward_token.address),
        2000
    );

    // uh-oh fail
    staking.fund_distribution(
        &admin,
        &TEN_MINUTES,
        &ONE_WEEK,
        &reward_token.address,
        &1000,
    );
}

#[test]
fn one_user_bond_twice_in_a_day_bond_one_more_time_after_a_week_get_rewards() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let manager = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &Address::generate(&env),
        &50u32,
    );

    staking.create_distribution_flow(&manager, &reward_token.address);

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    // bond tokens for user to enable distribution for him
    lp_token.mint(&user, &3000);
    env.ledger().with_mut(|li| {
        li.timestamp = ONE_DAY;
    });

    staking.fund_distribution(
        &admin,
        &ONE_DAY,
        &(2 * ONE_WEEK),
        &reward_token.address,
        &(reward_amount as i128),
    );

    // first bond for the day
    staking.bond(&user, &1000);

    staking.distribute_rewards();

    // it's the start of the rewards distribution, so we should have 0 distributed rewards
    assert_eq!(
        staking.query_undistributed_rewards(&reward_token.address),
        reward_amount
    );

    // user bonds for the second time in a day (12 hours later minus one second)
    env.ledger()
        .with_mut(|li| li.timestamp += (ONE_DAY / 2) - 1);
    staking.bond(&user, &1000);

    assert_eq!(staking.query_staked(&user).stakes.len(), 1);

    env.ledger().with_mut(|li| {
        li.timestamp += ONE_DAY / 2;
    });

    staking.distribute_rewards();
    // distribuion rewards duration is 2 weeks, so after 1 days after starting we should have 1/14 of the rewards
    // 1/14 out of 100_000 is 7142
    assert_eq!(
        staking.query_undistributed_rewards(&reward_token.address),
        92858
    );
    assert_eq!(
        staking.query_distributed_rewards(&reward_token.address),
        7142
    );

    // user bonds for a third time in the middle of the distribution period
    env.ledger().with_mut(|li| li.timestamp = ONE_WEEK);
    staking.bond(&user, &1_000);

    env.ledger()
        .with_mut(|li| li.timestamp = ONE_WEEK + ONE_DAY);

    staking.distribute_rewards();

    // after a week and a day we should have %50 of the rewards
    assert_eq!(
        staking.query_undistributed_rewards(&reward_token.address),
        50_000
    );
    assert_eq!(
        staking.query_distributed_rewards(&reward_token.address),
        50_000
    );

    // reward period is over
    env.ledger()
        .with_mut(|li| li.timestamp = 2 * ONE_WEEK + ONE_DAY);
    staking.distribute_rewards();

    assert_eq!(
        staking.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 99999
                }
            ]
        }
    );

    staking.withdraw_rewards(&user);
    // it's not actually one whole token that's left, but a fraction of that. I guess we owe this to the
    // rewards distribution timestamp I'm working with
    assert_eq!(reward_token.balance(&user), 99999);
    assert_eq!(reward_token.balance(&staking.address), 1);
}
