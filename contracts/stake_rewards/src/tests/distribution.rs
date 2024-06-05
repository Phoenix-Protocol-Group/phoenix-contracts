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
    assert_eq!(staking.query_total_staked(), 0);

    let start_timestamp = 100;
    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp;
    });

    reward_token.mint(&admin, &1_000_000);
    let reward_duration = 600;
    staking_rewards.fund_distribution(&admin, &start_timestamp, &reward_duration, &1_000_000);

    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp + 200; // distribution already goes for 1/3 of the time
    });

    // first user bonds after distribution started
    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    staking.bond(&user1, &10_000);
    staking_rewards.calculate_bond(&user1);

    staking_rewards.distribute_rewards();

    // at this points, since 1/3 of the time has passed and only one user is staking, he should have 33% of the rewards
    assert_eq!(
        staking_rewards.query_withdrawable_reward(&user1),
        WithdrawableRewardResponse {
            reward_address: reward_token.address.clone(),
            reward_amount: 333_332
        }
    );

    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp + 400; // distribution already goes for 2/3 of the time
    });

    // first user bonds before distribution started
    let user2 = Address::generate(&env);
    lp_token.mint(&user2, &10_000);
    staking.bond(&user2, &10_000);
    staking_rewards.calculate_bond(&user2);

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

// #[test]
// fn two_users_both_bonds_after_distribution_starts() {
//     let env = Env::default();
//     env.mock_all_auths();
//
//     let admin = Address::generate(&env);
//     let user = Address::generate(&env);
//     let user2 = Address::generate(&env);
//     let manager = Address::generate(&env);
//     let owner = Address::generate(&env);
//     let lp_token = deploy_token_contract(&env, &admin);
//     let reward_token = deploy_token_contract(&env, &admin);
//
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &owner,
//         &50u32,
//     );
//
//     staking.create_distribution_flow(&manager, &reward_token.address);
//
//     let reward_amount: u128 = 100_000;
//     reward_token.mint(&admin, &(reward_amount as i128));
//
//     env.ledger().with_mut(|li| li.timestamp = ONE_DAY);
//
//     let reward_duration = 600;
//     staking.fund_distribution(
//         &admin,
//         &ONE_DAY,
//         &reward_duration,
//         &reward_token.address,
//         &(reward_amount as i128),
//     );
//
//     env.ledger().with_mut(|li| {
//         li.timestamp += 200;
//     });
//     lp_token.mint(&user, &1000);
//     staking.bond(&user, &1000);
//
//     staking.distribute_rewards();
//
//     // at this points, since half of the time has passed and only one user is staking, he should have 50% of the rewards
//     assert_eq!(
//         staking.query_withdrawable_rewards(&user),
//         WithdrawableRewardsResponse {
//             rewards: vec![
//                 &env,
//                 WithdrawableReward {
//                     reward_address: reward_token.address.clone(),
//                     reward_amount: 33_333
//                 }
//             ]
//         }
//     );
//
//     // user2 starts staking after the distribution has begun
//     env.ledger().with_mut(|li| {
//         li.timestamp += 200;
//     });
//     lp_token.mint(&user2, &1000);
//     staking.bond(&user2, &1000);
//
//     staking.distribute_rewards();
//     assert_eq!(
//         staking.query_withdrawable_rewards(&user),
//         WithdrawableRewardsResponse {
//             rewards: vec![
//                 &env,
//                 WithdrawableReward {
//                     reward_address: reward_token.address.clone(),
//                     reward_amount: 49_999
//                 }
//             ]
//         }
//     );
//     assert_eq!(
//         staking.query_withdrawable_rewards(&user2),
//         WithdrawableRewardsResponse {
//             rewards: vec![
//                 &env,
//                 WithdrawableReward {
//                     reward_address: reward_token.address.clone(),
//                     reward_amount: 16_666
//                 }
//             ]
//         }
//     );
//
//     env.ledger().with_mut(|li| {
//         li.timestamp += 200;
//     });
//     staking.distribute_rewards();
//
//     // first user should get 75_000, second user 25_000 since he joined at the half time
//     assert_eq!(
//         staking.query_withdrawable_rewards(&user),
//         WithdrawableRewardsResponse {
//             rewards: vec![
//                 &env,
//                 WithdrawableReward {
//                     reward_address: reward_token.address.clone(),
//                     reward_amount: 66_666
//                 }
//             ]
//         }
//     );
//     assert_eq!(
//         staking.query_withdrawable_rewards(&user2),
//         WithdrawableRewardsResponse {
//             rewards: vec![
//                 &env,
//                 WithdrawableReward {
//                     reward_address: reward_token.address.clone(),
//                     reward_amount: 33_333
//                 }
//             ]
//         }
//     );
//
//     staking.withdraw_rewards(&user);
//     assert_eq!(reward_token.balance(&user), 66_666);
//     staking.withdraw_rewards(&user2);
//     assert_eq!(reward_token.balance(&user2), 33_333);
// }
//
// #[test]
// fn try_to_withdraw_rewards_without_bonding() {
//     let env = Env::default();
//     env.mock_all_auths();
//
//     let admin = Address::generate(&env);
//     let user = Address::generate(&env);
//     let manager = Address::generate(&env);
//     let owner = Address::generate(&env);
//     let lp_token = deploy_token_contract(&env, &admin);
//     let reward_token = deploy_token_contract(&env, &admin);
//
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &owner,
//         &50u32,
//     );
//
//     staking.create_distribution_flow(&manager, &reward_token.address);
//
//     let reward_amount: u128 = 100_000;
//     reward_token.mint(&admin, &(reward_amount as i128));
//
//     env.ledger().with_mut(|li| {
//         li.timestamp = 2_000;
//     });
//
//     let reward_duration = 600;
//     staking.fund_distribution(
//         &admin,
//         &2_000,
//         &reward_duration,
//         &reward_token.address,
//         &(reward_amount as i128),
//     );
//
//     env.ledger().with_mut(|li| {
//         li.timestamp = 2_600;
//     });
//     staking.distribute_rewards();
//     assert_eq!(
//         staking.query_undistributed_rewards(&reward_token.address),
//         reward_amount
//     );
//     assert_eq!(staking.query_distributed_rewards(&reward_token.address), 0);
//
//     assert_eq!(
//         staking.query_withdrawable_rewards(&user),
//         WithdrawableRewardsResponse {
//             rewards: vec![
//                 &env,
//                 WithdrawableReward {
//                     reward_address: reward_token.address.clone(),
//                     reward_amount: 0
//                 }
//             ]
//         }
//     );
//
//     staking.withdraw_rewards(&user);
//     assert_eq!(reward_token.balance(&user), 0);
// }
//
// #[test]
// #[should_panic(expected = "Stake: Fund distribution: Fund distribution start time is too early")]
// fn fund_distribution_starting_before_current_timestamp() {
//     let env = Env::default();
//     env.mock_all_auths();
//
//     let admin = Address::generate(&env);
//     let manager = Address::generate(&env);
//     let owner = Address::generate(&env);
//     let lp_token = deploy_token_contract(&env, &admin);
//     let reward_token = deploy_token_contract(&env, &admin);
//
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &owner,
//         &50u32,
//     );
//
//     staking.create_distribution_flow(&manager, &reward_token.address);
//
//     let reward_amount: u128 = 100_000;
//     reward_token.mint(&admin, &(reward_amount as i128));
//
//     env.ledger().with_mut(|li| {
//         li.timestamp = 2_000;
//     });
//
//     let reward_duration = 600;
//     staking.fund_distribution(
//         &admin,
//         &1_999,
//         &reward_duration,
//         &reward_token.address,
//         &(reward_amount as i128),
//     )
// }
//
// #[test]
// #[should_panic(expected = "Stake: Fund distribution: minimum reward amount not reached")]
// fn fund_distribution_with_reward_below_required_minimum() {
//     let env = Env::default();
//     env.mock_all_auths();
//
//     let admin = Address::generate(&env);
//     let manager = Address::generate(&env);
//     let owner = Address::generate(&env);
//
//     let lp_token = deploy_token_contract(&env, &admin);
//     let reward_token = deploy_token_contract(&env, &admin);
//
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &owner,
//         &50u32,
//     );
//
//     staking.create_distribution_flow(&manager, &reward_token.address);
//
//     reward_token.mint(&admin, &10);
//
//     env.ledger().with_mut(|li| {
//         li.timestamp = 2_000;
//     });
//
//     let reward_duration = 600;
//     staking.fund_distribution(&admin, &2_000, &reward_duration, &reward_token.address, &10);
// }
//
// #[test]
// fn calculate_apr() {
//     let env = Env::default();
//     env.mock_all_auths();
//
//     let admin = Address::generate(&env);
//     let user = Address::generate(&env);
//     let manager = Address::generate(&env);
//     let owner = Address::generate(&env);
//
//     let lp_token = deploy_token_contract(&env, &admin);
//     let reward_token = deploy_token_contract(&env, &admin);
//
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &owner,
//         &50u32,
//     );
//
//     staking.create_distribution_flow(&manager, &reward_token.address);
//
//     let reward_amount: u128 = 100_000;
//     reward_token.mint(&admin, &(reward_amount as i128));
//
//     env.ledger().with_mut(|li| {
//         li.timestamp = ONE_DAY;
//     });
//
//     // whole year of distribution
//     let reward_duration = 60 * 60 * 24 * 365;
//     staking.fund_distribution(
//         &admin,
//         &ONE_DAY,
//         &reward_duration,
//         &reward_token.address,
//         &(reward_amount as i128),
//     );
//
//     // nothing bonded, no rewards
//     assert_eq!(
//         staking.query_annualized_rewards(),
//         AnnualizedRewardsResponse {
//             rewards: vec![
//                 &env,
//                 AnnualizedReward {
//                     asset: reward_token.address.clone(),
//                     amount: String::from_str(&env, "0")
//                 }
//             ]
//         }
//     );
//
//     // bond tokens for user to enable distribution for him
//     lp_token.mint(&user, &1000);
//     env.ledger().with_mut(|li| {
//         li.timestamp += ONE_DAY;
//     });
//     staking.bond(&user, &1000);
//
//     // 100k rewards distributed for the whole year gives 100% APR
//     assert_eq!(
//         staking.query_annualized_rewards(),
//         AnnualizedRewardsResponse {
//             rewards: vec![
//                 &env,
//                 AnnualizedReward {
//                     asset: reward_token.address.clone(),
//                     amount: String::from_str(&env, "100000.975274725274725274")
//                 }
//             ]
//         }
//     );
//
//     let reward_amount: u128 = 50_000;
//     reward_token.mint(&admin, &(reward_amount as i128));
//
//     staking.fund_distribution(
//         &admin,
//         &(2 * &ONE_DAY),
//         &reward_duration,
//         &reward_token.address,
//         &(reward_amount as i128),
//     );
//
//     // having another 50k in rewards increases APR
//     assert_eq!(
//         staking.query_annualized_rewards(),
//         AnnualizedRewardsResponse {
//             rewards: vec![
//                 &env,
//                 AnnualizedReward {
//                     asset: reward_token.address.clone(),
//                     amount: String::from_str(&env, "149727")
//                 }
//             ]
//         }
//     );
// }
//
// #[test]
// fn test_v_phx_vul_010_unbond_breakes_reward_distribution() {
//     let env = Env::default();
//     env.mock_all_auths();
//
//     let admin = Address::generate(&env);
//     let user_1 = Address::generate(&env);
//     let user_2 = Address::generate(&env);
//     let manager = Address::generate(&env);
//     let owner = Address::generate(&env);
//     let lp_token = deploy_token_contract(&env, &admin);
//     let reward_token = deploy_token_contract(&env, &admin);
//
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &owner,
//         &50u32,
//     );
//
//     staking.create_distribution_flow(&manager, &reward_token.address);
//
//     let reward_amount: u128 = 100_000;
//     reward_token.mint(&admin, &(reward_amount as i128));
//
//     // bond tokens for user to enable distribution for him
//     lp_token.mint(&user_1, &1_000);
//     lp_token.mint(&user_2, &1_000);
//
//     env.ledger().with_mut(|li| li.timestamp = ONE_DAY);
//     staking.bond(&user_1, &1_000);
//     staking.bond(&user_2, &1_000);
//     let reward_duration = 10_000;
//     staking.fund_distribution(
//         &admin,
//         &ONE_DAY,
//         &reward_duration,
//         &reward_token.address,
//         &(reward_amount as i128),
//     );
//
//     env.ledger().with_mut(|li| {
//         li.timestamp += 2_000;
//     });
//
//     staking.distribute_rewards();
//     assert_eq!(
//         staking.query_undistributed_rewards(&reward_token.address),
//         80_000 // 100k total rewards, we have 2000 seconds passed, so we have 80k undistributed rewards
//     );
//
//     // at the 1/2 of the distribution time, user_1 unbonds
//     env.ledger().with_mut(|li| {
//         li.timestamp += 3_000;
//     });
//     staking.distribute_rewards();
//     assert_eq!(
//         staking.query_undistributed_rewards(&reward_token.address),
//         50_000
//     );
//
//     // user1 unbonds, which automatically withdraws the rewards
//     assert_eq!(
//         staking.query_withdrawable_rewards(&user_1),
//         WithdrawableRewardsResponse {
//             rewards: vec![
//                 &env,
//                 WithdrawableReward {
//                     reward_address: reward_token.address.clone(),
//                     reward_amount: 25_000
//                 }
//             ]
//         }
//     );
//     staking.unbond(&user_1, &1_000, &ONE_DAY);
//     assert_eq!(
//         staking.query_withdrawable_rewards(&user_1),
//         WithdrawableRewardsResponse {
//             rewards: vec![
//                 &env,
//                 WithdrawableReward {
//                     reward_address: reward_token.address.clone(),
//                     reward_amount: 0
//                 }
//             ]
//         }
//     );
//
//     env.ledger().with_mut(|li| {
//         li.timestamp += 10_000;
//     });
//
//     staking.distribute_rewards();
//     assert_eq!(
//         staking.query_undistributed_rewards(&reward_token.address),
//         0
//     );
//     assert_eq!(
//         staking.query_distributed_rewards(&reward_token.address),
//         reward_amount
//     );
//
//     assert_eq!(
//         staking.query_withdrawable_rewards(&user_2),
//         WithdrawableRewardsResponse {
//             rewards: vec![
//                 &env,
//                 WithdrawableReward {
//                     reward_address: reward_token.address.clone(),
//                     reward_amount: 75_000
//                 }
//             ]
//         }
//     );
//
//     staking.withdraw_rewards(&user_1);
//     assert_eq!(reward_token.balance(&user_1), 25_000i128);
// }
//
// #[should_panic(expected = "Stake: Fund distribution: Curve complexity validation failed")]
// #[test]
// fn panic_when_funding_distribution_with_curve_too_complex() {
//     const DISTRIBUTION_MAX_COMPLEXITY: u32 = 3;
//     const FIVE_MINUTES: u64 = 300;
//     const TEN_MINUTES: u64 = 600;
//     const ONE_WEEK: u64 = 604_800;
//
//     let env = Env::default();
//     env.mock_all_auths();
//
//     let admin = Address::generate(&env);
//     let manager = Address::generate(&env);
//     let owner = Address::generate(&env);
//     let lp_token = deploy_token_contract(&env, &admin);
//     let reward_token = deploy_token_contract(&env, &admin);
//
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &owner,
//         &DISTRIBUTION_MAX_COMPLEXITY,
//     );
//
//     staking.create_distribution_flow(&manager, &reward_token.address);
//
//     reward_token.mint(&admin, &3000);
//
//     staking.fund_distribution(&admin, &0, &FIVE_MINUTES, &reward_token.address, &1000);
//     staking.fund_distribution(
//         &admin,
//         &FIVE_MINUTES,
//         &TEN_MINUTES,
//         &reward_token.address,
//         &1000,
//     );
//
//     // assert just to prove that we have 2 successful fund distributions
//     assert_eq!(
//         staking.query_undistributed_rewards(&reward_token.address),
//         2000
//     );
//
//     // uh-oh fail
//     staking.fund_distribution(
//         &admin,
//         &TEN_MINUTES,
//         &ONE_WEEK,
//         &reward_token.address,
//         &1000,
//     );
// }
//
// #[test]
// fn one_user_bond_twice_in_a_day_bond_one_more_time_after_a_week_get_rewards() {
//     let env = Env::default();
//     env.mock_all_auths();
//
//     let admin = Address::generate(&env);
//     let user = Address::generate(&env);
//     let manager = Address::generate(&env);
//     let lp_token = deploy_token_contract(&env, &admin);
//     let reward_token = deploy_token_contract(&env, &admin);
//
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &Address::generate(&env),
//         &50u32,
//     );
//
//     staking.create_distribution_flow(&manager, &reward_token.address);
//
//     let reward_amount: u128 = 100_000;
//     reward_token.mint(&admin, &(reward_amount as i128));
//
//     // bond tokens for user to enable distribution for him
//     lp_token.mint(&user, &3000);
//     env.ledger().with_mut(|li| {
//         li.timestamp = ONE_DAY;
//     });
//
//     staking.fund_distribution(
//         &admin,
//         &ONE_DAY,
//         &(2 * ONE_WEEK),
//         &reward_token.address,
//         &(reward_amount as i128),
//     );
//
//     // first bond for the day
//     staking.bond(&user, &1000);
//
//     staking.distribute_rewards();
//
//     // it's the start of the rewards distribution, so we should have 0 distributed rewards
//     assert_eq!(
//         staking.query_undistributed_rewards(&reward_token.address),
//         reward_amount
//     );
//
//     // user bonds for the second time in a day (12 hours later minus one second)
//     env.ledger()
//         .with_mut(|li| li.timestamp += (ONE_DAY / 2) - 1);
//     staking.bond(&user, &1000);
//
//     assert_eq!(staking.query_staked(&user).stakes.len(), 1);
//
//     env.ledger().with_mut(|li| {
//         li.timestamp += ONE_DAY / 2;
//     });
//
//     staking.distribute_rewards();
//     // distribuion rewards duration is 2 weeks, so after 1 days after starting we should have 1/14 of the rewards
//     // 1/14 out of 100_000 is 7142
//     assert_eq!(
//         staking.query_undistributed_rewards(&reward_token.address),
//         92858
//     );
//     assert_eq!(
//         staking.query_distributed_rewards(&reward_token.address),
//         7142
//     );
//
//     // user bonds for a third time in the middle of the distribution period
//     env.ledger().with_mut(|li| li.timestamp = ONE_WEEK);
//     staking.bond(&user, &1_000);
//
//     env.ledger()
//         .with_mut(|li| li.timestamp = ONE_WEEK + ONE_DAY);
//
//     staking.distribute_rewards();
//
//     // after a week and a day we should have %50 of the rewards
//     assert_eq!(
//         staking.query_undistributed_rewards(&reward_token.address),
//         50_000
//     );
//     assert_eq!(
//         staking.query_distributed_rewards(&reward_token.address),
//         50_000
//     );
//
//     // reward period is over
//     env.ledger()
//         .with_mut(|li| li.timestamp = 2 * ONE_WEEK + ONE_DAY);
//     staking.distribute_rewards();
//
//     assert_eq!(
//         staking.query_withdrawable_rewards(&user),
//         WithdrawableRewardsResponse {
//             rewards: vec![
//                 &env,
//                 WithdrawableReward {
//                     reward_address: reward_token.address.clone(),
//                     reward_amount: 99999
//                 }
//             ]
//         }
//     );
//
//     staking.withdraw_rewards(&user);
//     // it's not actually one whole token that's left, but a fraction of that. I guess we owe this to the
//     // rewards distribution timestamp I'm working with
//     assert_eq!(reward_token.balance(&user), 99999);
//     assert_eq!(reward_token.balance(&staking.address), 1);
// }
