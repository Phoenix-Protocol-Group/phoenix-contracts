extern crate std;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Ledger},
    vec, Address, BytesN, Env, IntoVal, String, Symbol,
};

use super::setup::{deploy_staking_contract, deploy_token_contract};
use pretty_assertions::assert_eq;

use crate::{
    msg::{
        AnnualizedReward, AnnualizedRewardsResponse, WithdrawableReward,
        WithdrawableRewardsResponse,
    },
    tests::setup::{ONE_DAY, ONE_WEEK, SIXTY_DAYS},
};

#[test]
fn add_distribution_and_distribute_reward() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &admin,
        &50u32,
    );

    staking.create_distribution_flow(
        &admin,
        &reward_token.address,
        &BytesN::from_array(&env, &[1; 32]),
        &10,
        &100,
        &1,
    );

    assert_eq!(
        env.auths(),
        [(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    staking.address.clone(),
                    Symbol::new(&env, "create_distribution_flow"),
                    (
                        &admin.clone(),
                        reward_token.address.clone(),
                        BytesN::from_array(&env, &[1; 32]),
                        10u32,
                        100i128,
                        1i128
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![],
            }
        ),]
    );

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    // bond tokens for user to enable distribution for him
    lp_token.mint(&user, &1000);
    staking.bond(&user, &1000);

    // simulate moving forward 60 days for the full APR multiplier
    env.ledger().with_mut(|li| {
        li.timestamp = SIXTY_DAYS;
    });

    let staking_rewards = staking.query_distribution(&reward_token.address).unwrap();

    let reward_duration = 600;
    staking.fund_distribution(
        &SIXTY_DAYS,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );

    assert_eq!(
        env.auths(),
        [(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    staking.address.clone(),
                    Symbol::new(&env, "fund_distribution"),
                    (
                        SIXTY_DAYS,
                        reward_duration,
                        reward_token.address.clone(),
                        reward_amount as i128
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        staking_rewards.clone(), // Repeat the fund_distribution call
                        Symbol::new(&env, "fund_distribution"),
                        (SIXTY_DAYS, reward_duration, reward_amount as i128).into_val(&env),
                    )),
                    sub_invocations: std::vec![AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            reward_token.address.clone(),
                            symbol_short!("transfer"),
                            (&admin, &staking_rewards.clone(), reward_amount as i128)
                                .into_val(&env)
                        )),
                        sub_invocations: std::vec![],
                    },],
                },],
            }
        ),]
    );

    staking.distribute_rewards();
    assert_eq!(
        staking.query_undistributed_rewards(&reward_token.address),
        reward_amount
    );

    env.ledger().with_mut(|li| {
        li.timestamp = SIXTY_DAYS + reward_duration;
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
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let manager = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);
    let reward_token_2 = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &admin,
        &50u32,
    );

    staking.create_distribution_flow(
        &admin,
        &reward_token.address,
        &BytesN::from_array(&env, &[1; 32]),
        &10,
        &100,
        &1,
    );
    staking.create_distribution_flow(
        &admin,
        &reward_token_2.address,
        &BytesN::from_array(&env, &[2; 32]),
        &10,
        &100,
        &1,
    );

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));
    reward_token_2.mint(&admin, &((reward_amount * 2) as i128));

    // bond tokens for user to enable distribution for him
    lp_token.mint(&user, &1000);
    staking.bond(&user, &1000);
    // simulate moving forward 60 days for the full APR multiplier
    env.ledger().with_mut(|li| li.timestamp = SIXTY_DAYS);

    let reward_duration = 600;
    staking.fund_distribution(
        &SIXTY_DAYS,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );
    staking.fund_distribution(
        &SIXTY_DAYS,
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
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let user4 = Address::generate(&env);
    let manager = Address::generate(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &admin,
        &50u32,
    );

    staking.create_distribution_flow(
        &admin,
        &reward_token.address,
        &BytesN::from_array(&env, &[1; 32]),
        &10,
        &100,
        &1,
    );

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

    // simulate moving forward 60 days for the full APR multiplier
    env.ledger().with_mut(|li| {
        li.timestamp = SIXTY_DAYS;
    });

    let reward_duration = 600;
    staking.fund_distribution(
        &SIXTY_DAYS,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );

    env.ledger().with_mut(|li| {
        li.timestamp += 600;
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
    env.budget().reset_unlimited();

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
        &admin,
        &50u32,
    );

    staking.create_distribution_flow(
        &admin,
        &reward_token.address,
        &BytesN::from_array(&env, &[1; 32]),
        &10,
        &100,
        &1,
    );

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    // first user bonds before distribution started
    lp_token.mint(&user, &1000);
    staking.bond(&user, &1000);
    // simulate moving forward 60 days for the full APR multiplier
    env.ledger().with_mut(|li| li.timestamp = SIXTY_DAYS);

    let reward_duration = 600;
    staking.fund_distribution(
        &SIXTY_DAYS,
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

    // Second user bonded later; we again simulate moving his stakes up to 60 days
    env.ledger().with_mut(|li| {
        li.timestamp += SIXTY_DAYS;
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
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let user2 = Address::generate(&env);
    let manager = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &admin,
        &50u32,
    );

    staking.create_distribution_flow(
        &admin,
        &reward_token.address,
        &BytesN::from_array(&env, &[1; 32]),
        &10,
        &100,
        &1,
    );

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    let reward_duration = SIXTY_DAYS * 2;
    staking.fund_distribution(
        &0,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );

    lp_token.mint(&user, &1000);
    staking.bond(&user, &1000);
    env.ledger().with_mut(|li| {
        li.timestamp = SIXTY_DAYS;
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
    // we move time to the end of the distribution
    env.ledger().with_mut(|li| {
        li.timestamp = SIXTY_DAYS * 2;
    });

    staking.distribute_rewards();
    // user 1 was the only who bonded for the first half time
    // and then he had 50% for the second half
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
#[should_panic(expected = "Stake: Fund distribution: No distribution for this reward token exists")]
fn fund_rewards_without_establishing_distribution() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &admin,
        &50u32,
    );

    reward_token.mint(&admin, &1000);

    staking.fund_distribution(&2_000, &600, &reward_token.address, &1000);
}

#[test]
fn try_to_withdraw_rewards_without_bonding() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &admin,
        &50u32,
    );

    staking.create_distribution_flow(
        &admin,
        &reward_token.address,
        &BytesN::from_array(&env, &[1; 32]),
        &10,
        &100,
        &1,
    );

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    env.ledger().with_mut(|li| {
        li.timestamp = 2_000;
    });

    let reward_duration = 600;
    staking.fund_distribution(
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
// Error #9 at stake_rewards: InvalidTime = 9
#[should_panic(expected = "Error(Contract, #9)")]
fn fund_distribution_starting_before_current_timestamp() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &admin,
        &50u32,
    );

    staking.create_distribution_flow(
        &admin,
        &reward_token.address,
        &BytesN::from_array(&env, &[1; 32]),
        &10,
        &100,
        &1,
    );

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    env.ledger().with_mut(|li| {
        li.timestamp = 2_000;
    });

    let reward_duration = 600;
    staking.fund_distribution(
        &1_999,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    )
}

#[test]
// Error #6 at stake_rewards: MinRewardNotEnough = 6
#[should_panic(expected = "Error(Contract, #6)")]
fn fund_distribution_with_reward_below_required_minimum() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &admin,
        &50u32,
    );

    staking.create_distribution_flow(
        &admin,
        &reward_token.address,
        &BytesN::from_array(&env, &[1; 32]),
        &10,
        &100,
        &1,
    );

    reward_token.mint(&admin, &10);

    env.ledger().with_mut(|li| {
        li.timestamp = 2_000;
    });

    let reward_duration = 600;
    staking.fund_distribution(&2_000, &reward_duration, &reward_token.address, &10);
}

#[test]
fn calculate_apr() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &admin,
        &50u32,
    );

    staking.create_distribution_flow(
        &admin,
        &reward_token.address,
        &BytesN::from_array(&env, &[1; 32]),
        &10,
        &100,
        &1,
    );

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    // whole year of distribution
    let reward_duration = 60 * 60 * 24 * 365;
    staking.fund_distribution(
        &SIXTY_DAYS,
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
    // simulate moving forward 60 days for the full APR multiplier
    env.ledger().with_mut(|li| {
        li.timestamp = SIXTY_DAYS;
    });

    // 100k rewards distributed for the 10 months gives ~120% APR
    assert_eq!(
        staking.query_annualized_rewards(),
        AnnualizedRewardsResponse {
            rewards: vec![
                &env,
                AnnualizedReward {
                    asset: reward_token.address.clone(),
                    amount: String::from_str(&env, "119672.131147540983606557")
                }
            ]
        }
    );

    let reward_amount: u128 = 50_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    staking.fund_distribution(
        &(2 * &SIXTY_DAYS),
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
                    amount: String::from_str(&env, "150000")
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

    staking.create_distribution_flow(
        &Address::generate(&env),
        &reward_token.address,
        &BytesN::from_array(&env, &[1; 32]),
        &10,
        &100,
        &1,
    );
}

#[test]
fn test_v_phx_vul_010_unbond_breakes_reward_distribution() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let user_1 = Address::generate(&env);
    let user_2 = Address::generate(&env);
    let manager = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &admin,
        &50u32,
    );

    staking.create_distribution_flow(
        &admin,
        &reward_token.address,
        &BytesN::from_array(&env, &[1; 32]),
        &10,
        &100,
        &1,
    );

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    // bond tokens for user to enable distribution for him
    lp_token.mint(&user_1, &1_000);
    lp_token.mint(&user_2, &1_000);

    staking.bond(&user_1, &1_000);
    staking.bond(&user_2, &1_000);

    // simulate moving forward 60 days for the full APR multiplier
    env.ledger().with_mut(|li| li.timestamp = SIXTY_DAYS);

    let reward_duration = 10_000;
    staking.fund_distribution(
        &SIXTY_DAYS,
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
    staking.unbond(&user_1, &1_000, &0);
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
    env.budget().reset_unlimited();

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
        &admin,
        &50u32,
    );

    staking.create_distribution_flow(
        &admin,
        &reward_token.address,
        &BytesN::from_array(&env, &[1; 32]),
        &10,
        &100,
        &1,
    );

    let reward_amount: u128 = 100_000;
    reward_token.mint(&admin, &(reward_amount as i128));

    lp_token.mint(&user, &1_000);
    staking.bond(&user, &1_000);

    // simulate moving forward 60 days for the full APR multiplier
    env.ledger().with_mut(|li| {
        li.timestamp = SIXTY_DAYS;
    });

    let reward_duration = 10_000;

    staking.fund_distribution(
        &SIXTY_DAYS,
        &reward_duration,
        &reward_token.address,
        &(reward_amount as i128),
    );

    env.ledger().with_mut(|li| {
        li.timestamp += reward_duration;
    });

    staking.distribute_rewards();

    staking.unbond(&user, &1_000, &0);

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

#[should_panic(
    expected = "Stake: Create distribution flow: Distribution for this reward token exists!"
)]
#[test]
fn panic_when_adding_same_distribution_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &admin,
        &50u32,
    );

    staking.create_distribution_flow(
        &admin,
        &reward_token.address,
        &BytesN::from_array(&env, &[1; 32]),
        &10,
        &100,
        &1,
    );
    staking.create_distribution_flow(
        &admin,
        &reward_token.address,
        &BytesN::from_array(&env, &[1; 32]),
        &10,
        &100,
        &1,
    );
}

// Error #12 at stake_rewards: InvalidMaxComplexity = 12
#[should_panic(expected = "Error(Contract, #12)")]
#[test]
fn panic_when_funding_distribution_with_curve_too_complex() {
    const DISTRIBUTION_MAX_COMPLEXITY: u32 = 3;
    const FIVE_MINUTES: u64 = 300;
    const TEN_MINUTES: u64 = 600;
    const ONE_WEEK: u64 = 604_800;

    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &admin,
        &DISTRIBUTION_MAX_COMPLEXITY,
    );

    staking.create_distribution_flow(
        &admin,
        &reward_token.address,
        &BytesN::from_array(&env, &[1; 32]),
        &10,
        &100,
        &1,
    );

    reward_token.mint(&admin, &10000);

    staking.fund_distribution(&0, &FIVE_MINUTES, &reward_token.address, &1000);
    staking.fund_distribution(&FIVE_MINUTES, &TEN_MINUTES, &reward_token.address, &1000);
    staking.fund_distribution(&TEN_MINUTES, &ONE_WEEK, &reward_token.address, &1000);
    staking.fund_distribution(
        &(ONE_WEEK + 1),
        &(ONE_WEEK + 3),
        &reward_token.address,
        &1000,
    );
    staking.fund_distribution(
        &(ONE_WEEK + 3),
        &(ONE_WEEK + 5),
        &reward_token.address,
        &1000,
    );
    staking.fund_distribution(
        &(ONE_WEEK + 6),
        &(ONE_WEEK + 7),
        &reward_token.address,
        &1000,
    );
    staking.fund_distribution(
        &(ONE_WEEK + 8),
        &(ONE_WEEK + 9),
        &reward_token.address,
        &1000,
    );
}
