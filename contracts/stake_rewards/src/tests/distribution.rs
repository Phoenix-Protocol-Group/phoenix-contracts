use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env, String,
};

use super::setup::{deploy_staking_rewards_contract, deploy_token_contract};

use crate::msg::{AnnualizedRewardResponse, WithdrawableRewardResponse};

#[test]
fn two_users_one_starts_after_distribution_begins() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);

    // we simulate the full APR after 60 days of staking
    let sixty_days = 3600 * 24 * 60;

    // first user bonds before distribution started
    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    staking.bond(&user1, &10_000);

    reward_token.mint(&admin, &1_000_000);
    // therefore the distribution must take at least 60 days in this test case
    let reward_duration = sixty_days * 3;
    // distribution starts at time 0
    staking_rewards.fund_distribution(&0, &reward_duration, &1_000_000);

    env.ledger().with_mut(|li| {
        li.timestamp = sixty_days; // distribution already goes for 1/3 of the time
    });

    staking_rewards.distribute_rewards();

    // at this points, since 1/3 of the time has passed and only one user is staking, he should have 33% of the rewards
    assert_eq!(
        staking_rewards.query_withdrawable_reward(&user1),
        WithdrawableRewardResponse {
            reward_address: reward_token.address.clone(),
            reward_amount: 333_332
        }
    );

    // second user bonds and we are waiting another 60 days for the full APR
    let user2 = Address::generate(&env);
    lp_token.mint(&user2, &10_000);
    staking.bond(&user2, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp = sixty_days * 2; // distribution already goes for 2/3 of the time
    });

    staking_rewards.distribute_rewards();

    // Now we need to split the previous reward equivalent into a two users
    assert_eq!(
        staking_rewards.query_withdrawable_reward(&user1),
        WithdrawableRewardResponse {
            reward_address: reward_token.address.clone(),
            reward_amount: 333_332 + 166_667,
        }
    );
    assert_eq!(
        staking_rewards.query_withdrawable_reward(&user2),
        WithdrawableRewardResponse {
            reward_address: reward_token.address.clone(),
            reward_amount: 166_666
        }
    );

    staking_rewards.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 499_999);
    staking_rewards.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 166_666);
    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        333_334
    );
}

#[test]
fn two_users_both_bonds_after_distribution_starts() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);

    // we simulate the full APR after 60 days of staking
    let sixty_days = 3600 * 24 * 60;

    reward_token.mint(&admin, &1_000_000);
    // therefore the distribution must take at least 60 days in this test case
    let reward_duration = sixty_days * 3;
    // distribution starts at time 0
    staking_rewards.fund_distribution(&0, &reward_duration, &1_000_000);

    // first user bonds after distribution started
    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    staking.bond(&user1, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp = sixty_days; // distribution already goes for 1/3 of the time
    });

    staking_rewards.distribute_rewards();

    // at this points, since 1/3 of the time has passed and only one user is staking, he should have 33% of the rewards
    assert_eq!(
        staking_rewards.query_withdrawable_reward(&user1),
        WithdrawableRewardResponse {
            reward_address: reward_token.address.clone(),
            reward_amount: 333_332
        }
    );

    // second user bonds and we are waiting another 60 days for the full APR
    let user2 = Address::generate(&env);
    lp_token.mint(&user2, &10_000);
    staking.bond(&user2, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp = sixty_days * 2; // distribution already goes for 2/3 of the time
    });

    staking_rewards.distribute_rewards();

    // Now we need to split the previous reward equivalent into a two users
    assert_eq!(
        staking_rewards.query_withdrawable_reward(&user1),
        WithdrawableRewardResponse {
            reward_address: reward_token.address.clone(),
            reward_amount: 333_332 + 166_667,
        }
    );
    assert_eq!(
        staking_rewards.query_withdrawable_reward(&user2),
        WithdrawableRewardResponse {
            reward_address: reward_token.address.clone(),
            reward_amount: 166_666
        }
    );

    staking_rewards.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 499_999);
    staking_rewards.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 166_666);
    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        333_334
    );
}

#[test]
fn try_to_withdraw_rewards_without_bonding() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (_staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);

    let start_timestamp = 100;
    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp;
    });

    reward_token.mint(&admin, &1_000_000);
    let reward_duration = 600;
    staking_rewards.fund_distribution(&start_timestamp, &reward_duration, &1_000_000);

    env.ledger().with_mut(|li| {
        li.timestamp = 2_600;
    });
    staking_rewards.distribute_rewards();
    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        1_000_000
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        0
    );

    let user = Address::generate(&env);
    assert_eq!(
        staking_rewards.query_withdrawable_reward(&reward_token.address),
        WithdrawableRewardResponse {
            reward_address: reward_token.address.clone(),
            reward_amount: 0
        }
    );

    staking_rewards.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), 0);
}

#[test]
#[should_panic(
    expected = "Stake rewards: Fund distribution: Fund distribution start time is too early"
)]
fn fund_distribution_starting_before_current_timestamp() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (_staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);

    let start_timestamp = 100;
    env.ledger().with_mut(|li| {
        li.timestamp = 150;
    });

    reward_token.mint(&admin, &1_000_000);
    let reward_duration = 600;
    staking_rewards.fund_distribution(&start_timestamp, &reward_duration, &1_000_000);
}

#[test]
#[should_panic(expected = "Stake rewards: Fund distribution: minimum reward amount not reached")]
fn fund_distribution_with_reward_below_required_minimum() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (_staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);

    let start_timestamp = 100;
    reward_token.mint(&admin, &100);
    let reward_duration = 600;
    // Min reward is defined in setup as 1_000 tokens
    staking_rewards.fund_distribution(&start_timestamp, &reward_duration, &999);
}

#[test]
fn calculate_apr() {
    let day_in_seconds = 3600 * 24;

    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);
    assert_eq!(staking.query_total_staked(), 0);

    let start_timestamp = day_in_seconds;
    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp;
    });

    reward_token.mint(&admin, &1_000_000);
    // whole year of distribution
    let reward_duration = 60 * 60 * 24 * 365;
    staking_rewards.fund_distribution(&start_timestamp, &reward_duration, &1_000_000);

    // nothing bonded, no rewards
    assert_eq!(
        staking_rewards.query_annualized_reward(),
        AnnualizedRewardResponse {
            asset: reward_token.address.clone(),
            amount: String::from_str(&env, "0")
        }
    );

    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    staking.bond(&user1, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp += day_in_seconds;
    });

    // 100k rewards distributed for the whole year gives 100% APR
    assert_eq!(
        staking_rewards.query_annualized_reward(),
        AnnualizedRewardResponse {
            asset: reward_token.address.clone(),
            amount: String::from_str(&env, "100000.072802197802197802")
        }
    );

    let reward_amount: i128 = 500_000;
    reward_token.mint(&admin, &reward_amount);

    staking_rewards.fund_distribution(&(2 * start_timestamp), &reward_duration, &reward_amount);

    // having another 50k in rewards increases APR
    assert_eq!(
        staking_rewards.query_annualized_reward(),
        AnnualizedRewardResponse {
            asset: reward_token.address.clone(),
            amount: String::from_str(&env, "149726.1")
        }
    );
}

#[test]
fn test_v_phx_vul_010_unbond_breakes_reward_distribution() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);

    // bond tokens for user to enable distribution for him
    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &1_000);
    staking.bond(&user1, &1_000);

    let user2 = Address::generate(&env);
    lp_token.mint(&user2, &1_000);
    staking.bond(&user2, &1_000);

    // we simulate full stake time
    let start_timestamp = 60 * 3600 * 24;
    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp;
    });

    let reward_duration = 10_000;
    let reward_amount = 100_000;
    reward_token.mint(&admin, &reward_amount);
    staking_rewards.fund_distribution(
        &start_timestamp, // start distirbution
        &reward_duration,
        &reward_amount,
    );

    env.ledger().with_mut(|li| {
        li.timestamp += 2_000;
    });

    staking_rewards.distribute_rewards();
    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        80_000 // 100k total rewards, we have 2000 seconds passed, so we have 80k undistributed rewards
    );

    // at the 1/2 of the distribution time, user_1 unbonds
    env.ledger().with_mut(|li| {
        li.timestamp += 3_000;
    });
    staking_rewards.distribute_rewards();
    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        50_000
    );

    // user1 unbonds, which automatically withdraws the rewards
    assert_eq!(
        staking_rewards.query_withdrawable_reward(&user1),
        WithdrawableRewardResponse {
            reward_address: reward_token.address.clone(),
            reward_amount: 25_000
        }
    );
    staking.unbond(&user1, &1_000, &0); // when he bonded
    assert_eq!(
        staking_rewards.query_withdrawable_reward(&user1),
        WithdrawableRewardResponse {
            reward_address: reward_token.address.clone(),
            reward_amount: 0
        }
    );

    env.ledger().with_mut(|li| {
        li.timestamp += 10_000;
    });

    staking_rewards.distribute_rewards();
    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        0
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        reward_amount as u128
    );

    assert_eq!(
        staking_rewards.query_withdrawable_reward(&user2),
        WithdrawableRewardResponse {
            reward_address: reward_token.address.clone(),
            reward_amount: 75_000
        }
    );

    staking_rewards.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 25_000i128);
}

#[should_panic(expected = "Stake rewards: Fund distribution: Curve complexity validation failed")]
#[test]
fn panic_when_funding_distribution_with_curve_too_complex() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (_staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);

    reward_token.mint(&admin, &10_000);

    // Default max complexity in setup.rs is 10
    staking_rewards.fund_distribution(&17, &300, &1000);
    staking_rewards.fund_distribution(&15, &280, &1000);
    staking_rewards.fund_distribution(&30, &154, &1000);
    staking_rewards.fund_distribution(&532, &754, &1000);
    staking_rewards.fund_distribution(&210, &423154, &1000);
    staking_rewards.fund_distribution(&640, &53254, &1000);
}

#[test]
fn add_multiple_users() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);
    assert_eq!(staking.query_total_staked(), 0);

    // first user bonds before distribution started
    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    staking.bond(&user1, &10_000);
    // do not calculate bond first

    // second user bonds after distribution started
    let user2 = Address::generate(&env);
    lp_token.mint(&user2, &10_000);
    staking.bond(&user2, &10_000);

    // we simulate full stake time
    let start_timestamp = 60 * 3600 * 24;
    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp;
    });

    reward_token.mint(&admin, &1_000_000);
    let reward_duration = 600;
    staking_rewards.fund_distribution(&start_timestamp, &reward_duration, &1_000_000);

    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp + 200; // distribution already goes for 1/3 of the time
    });

    staking_rewards.add_multiple_users(&vec![&env, user1.clone(), user2.clone()]);

    staking_rewards.distribute_rewards();

    // at this points, since 1/3 of the time has passed and two users are staking, he should have 33% /2 of the rewards
    assert_eq!(
        staking_rewards.query_withdrawable_reward(&user1),
        WithdrawableRewardResponse {
            reward_address: reward_token.address.clone(),
            reward_amount: 166_666
        }
    );

    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp + 400; // distribution already goes for 2/3 of the time
    });

    staking_rewards.distribute_rewards();

    // Now we need to split the previous reward equivalent into a two users
    assert_eq!(
        staking_rewards.query_withdrawable_reward(&user1),
        WithdrawableRewardResponse {
            reward_address: reward_token.address.clone(),
            reward_amount: 166_666 + 166_666,
        }
    );
    assert_eq!(
        staking_rewards.query_withdrawable_reward(&user2),
        WithdrawableRewardResponse {
            reward_address: reward_token.address.clone(),
            reward_amount: 333_332
        }
    );

    staking_rewards.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 333_332);
    staking_rewards.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 333_332);
    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        333_334
    );
}
