extern crate std;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env,
};

use super::setup::{deploy_staking_contract, deploy_token_contract};
use pretty_assertions::assert_eq;

use crate::{
    distribution::SECONDS_PER_DAY,
    msg::{WithdrawableReward, WithdrawableRewardsResponse},
    tests::setup::SIXTY_DAYS,
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

    staking.create_distribution_flow(&admin, &reward_token.address);

    let reward_amount: i128 = 100_000;
    reward_token.mint(&admin, &reward_amount);

    // bond tokens for user to enable distribution for him
    lp_token.mint(&user, &1000);
    staking.bond(&user, &1000);

    // simulate moving forward 60 days for the full APR multiplier
    env.ledger().with_mut(|li| {
        li.timestamp = SIXTY_DAYS;
    });

    for _ in 0..60 {
        staking.distribute_rewards(&admin, &(reward_amount / 60i128), &reward_token.address);
        env.ledger().with_mut(|li| {
            li.timestamp += 3600 * 24;
        });
    }

    assert_eq!(
        staking.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    // dividing 100k / 60 rounding
                    reward_amount: 99_960_u128
                }
            ]
        }
    );

    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), 99_960);
}

// #[test]
// fn two_distributions() {
//     let env = Env::default();
//     env.mock_all_auths();
//     env.budget().reset_unlimited();
//
//     let admin = Address::generate(&env);
//     let user = Address::generate(&env);
//     let manager = Address::generate(&env);
//     let lp_token = deploy_token_contract(&env, &admin);
//     let reward_token = deploy_token_contract(&env, &admin);
//     let reward_token_2 = deploy_token_contract(&env, &admin);
//
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &admin,
//         &50u32,
//     );
//
//     staking.create_distribution_flow(
//         &admin,
//         &reward_token.address,
//         &BytesN::from_array(&env, &[1; 32]),
//         &10,
//         &100,
//         &1,
//     );
//     staking.create_distribution_flow(
//         &admin,
//         &reward_token_2.address,
//         &BytesN::from_array(&env, &[2; 32]),
//         &10,
//         &100,
//         &1,
//     );
//
//     let reward_amount: u128 = 100_000;
//     reward_token.mint(&admin, &(reward_amount as i128));
//     reward_token_2.mint(&admin, &((reward_amount * 2) as i128));
//
//     // bond tokens for user to enable distribution for him
//     lp_token.mint(&user, &1000);
//     staking.bond(&user, &1000);
//     // simulate moving forward 60 days for the full APR multiplier
//     env.ledger().with_mut(|li| li.timestamp = SIXTY_DAYS);
//
//     let reward_duration = 600;
//     staking.fund_distribution(
//         &SIXTY_DAYS,
//         &reward_duration,
//         &reward_token.address,
//         &(reward_amount as i128),
//     );
//     staking.fund_distribution(
//         &SIXTY_DAYS,
//         &reward_duration,
//         &reward_token_2.address,
//         &((reward_amount * 2) as i128),
//     );
//
//     // distribute rewards during half time
//     env.ledger().with_mut(|li| {
//         li.timestamp += 300;
//     });
//     staking.distribute_rewards();
//     assert_eq!(
//         staking.query_withdrawable_rewards(&user),
//         WithdrawableRewardsResponse {
//             rewards: vec![
//                 &env,
//                 WithdrawableReward {
//                     reward_address: reward_token.address.clone(),
//                     reward_amount: reward_amount / 2
//                 },
//                 WithdrawableReward {
//                     reward_address: reward_token_2.address.clone(),
//                     reward_amount
//                 }
//             ]
//         }
//     );
//     staking.withdraw_rewards(&user);
//     assert_eq!(reward_token.balance(&user), (reward_amount / 2) as i128);
//     assert_eq!(reward_token_2.balance(&user), reward_amount as i128);
//
//     env.ledger().with_mut(|li| {
//         li.timestamp += 600;
//     });
//     staking.distribute_rewards();
//     // first reward token
//     assert_eq!(
//         staking.query_undistributed_rewards(&reward_token.address),
//         0
//     );
//     assert_eq!(
//         staking.query_distributed_rewards(&reward_token.address),
//         reward_amount
//     );
//     // second reward token
//     assert_eq!(
//         staking.query_undistributed_rewards(&reward_token_2.address),
//         0
//     );
//     assert_eq!(
//         staking.query_distributed_rewards(&reward_token_2.address),
//         reward_amount * 2
//     );
//
//     // since half of rewards were already distributed, after full distirubtion
//     // round another half is ready
//     assert_eq!(
//         staking.query_withdrawable_rewards(&user),
//         WithdrawableRewardsResponse {
//             rewards: vec![
//                 &env,
//                 WithdrawableReward {
//                     reward_address: reward_token.address.clone(),
//                     reward_amount: reward_amount / 2
//                 },
//                 WithdrawableReward {
//                     reward_address: reward_token_2.address.clone(),
//                     reward_amount
//                 }
//             ]
//         }
//     );
//
//     staking.withdraw_rewards(&user);
//     assert_eq!(reward_token.balance(&user), reward_amount as i128);
//     assert_eq!(reward_token_2.balance(&user), (reward_amount * 2) as i128);
// }

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

    staking.create_distribution_flow(&admin, &reward_token.address);

    let reward_amount: i128 = 100_000;
    reward_token.mint(&admin, &reward_amount);

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

    // distribute 100k of rewards once
    staking.distribute_rewards(&admin, &reward_amount, &reward_token.address);

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
#[should_panic(
    expected = "Stake: Distribute rewards: No distribution for this reward token exists"
)]
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

    staking.distribute_rewards(&admin, &2_000, &reward_token.address);
}

// #[test]
// fn try_to_withdraw_rewards_without_bonding() {
//     let env = Env::default();
//     env.mock_all_auths();
//     env.budget().reset_unlimited();
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
//         &admin,
//         &50u32,
//     );
//
//     staking.create_distribution_flow(
//         &admin,
//         &reward_token.address,
//         &BytesN::from_array(&env, &[1; 32]),
//         &10,
//         &100,
//         &1,
//     );
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
// fn calculate_apr() {
//     let env = Env::default();
//     env.mock_all_auths();
//     env.budget().reset_unlimited();
//
//     let admin = Address::generate(&env);
//     let user = Address::generate(&env);
//     let manager = Address::generate(&env);
//
//     let lp_token = deploy_token_contract(&env, &admin);
//     let reward_token = deploy_token_contract(&env, &admin);
//
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &admin,
//         &50u32,
//     );
//
//     staking.create_distribution_flow(
//         &admin,
//         &reward_token.address,
//         &BytesN::from_array(&env, &[1; 32]),
//         &10,
//         &100,
//         &1,
//     );
//
//     let reward_amount: u128 = 100_000;
//     reward_token.mint(&admin, &(reward_amount as i128));
//
//     // whole year of distribution
//     let reward_duration = 60 * 60 * 24 * 365;
//     staking.fund_distribution(
//         &SIXTY_DAYS,
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
//     // simulate moving forward 60 days for the full APR multiplier
//     env.ledger().with_mut(|li| {
//         li.timestamp = SIXTY_DAYS;
//     });
//
//     // 100k rewards distributed for the 10 months gives ~120% APR
//     assert_eq!(
//         staking.query_annualized_rewards(),
//         AnnualizedRewardsResponse {
//             rewards: vec![
//                 &env,
//                 AnnualizedReward {
//                     asset: reward_token.address.clone(),
//                     amount: String::from_str(&env, "119672.131147540983606557")
//                 }
//             ]
//         }
//     );
//
//     let reward_amount: u128 = 50_000;
//     reward_token.mint(&admin, &(reward_amount as i128));
//
//     staking.fund_distribution(
//         &(2 * &SIXTY_DAYS),
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
//                     amount: String::from_str(&env, "150000")
//                 }
//             ]
//         }
//     );
// }
//
// #[test]
// #[should_panic(expected = "Stake: create distribution: Non-authorized creation!")]
// fn add_distribution_should_fail_when_not_authorized() {
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
//     staking.create_distribution_flow(
//         &Address::generate(&env),
//         &reward_token.address,
//         &BytesN::from_array(&env, &[1; 32]),
//         &10,
//         &100,
//         &1,
//     );
// }
//
// #[test]
// fn test_v_phx_vul_010_unbond_breakes_reward_distribution() {
//     let env = Env::default();
//     env.mock_all_auths();
//     env.budget().reset_unlimited();
//
//     let admin = Address::generate(&env);
//     let user_1 = Address::generate(&env);
//     let user_2 = Address::generate(&env);
//     let manager = Address::generate(&env);
//     let lp_token = deploy_token_contract(&env, &admin);
//     let reward_token = deploy_token_contract(&env, &admin);
//
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &admin,
//         &50u32,
//     );
//
//     staking.create_distribution_flow(
//         &admin,
//         &reward_token.address,
//         &BytesN::from_array(&env, &[1; 32]),
//         &10,
//         &100,
//         &1,
//     );
//
//     let reward_amount: u128 = 100_000;
//     reward_token.mint(&admin, &(reward_amount as i128));
//
//     // bond tokens for user to enable distribution for him
//     lp_token.mint(&user_1, &1_000);
//     lp_token.mint(&user_2, &1_000);
//
//     staking.bond(&user_1, &1_000);
//     staking.bond(&user_2, &1_000);
//
//     // simulate moving forward 60 days for the full APR multiplier
//     env.ledger().with_mut(|li| li.timestamp = SIXTY_DAYS);
//
//     let reward_duration = 10_000;
//     staking.fund_distribution(
//         &SIXTY_DAYS,
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
//     staking.unbond(&user_1, &1_000, &0);
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
// #[test]
// fn test_bond_withdraw_unbond() {
//     let env = Env::default();
//     env.mock_all_auths();
//     env.budget().reset_unlimited();
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
//         &admin,
//         &50u32,
//     );
//
//     staking.create_distribution_flow(
//         &admin,
//         &reward_token.address,
//         &BytesN::from_array(&env, &[1; 32]),
//         &10,
//         &100,
//         &1,
//     );
//
//     let reward_amount: u128 = 100_000;
//     reward_token.mint(&admin, &(reward_amount as i128));
//
//     lp_token.mint(&user, &1_000);
//     staking.bond(&user, &1_000);
//
//     // simulate moving forward 60 days for the full APR multiplier
//     env.ledger().with_mut(|li| {
//         li.timestamp = SIXTY_DAYS;
//     });
//
//     let reward_duration = 10_000;
//
//     staking.fund_distribution(
//         &SIXTY_DAYS,
//         &reward_duration,
//         &reward_token.address,
//         &(reward_amount as i128),
//     );
//
//     env.ledger().with_mut(|li| {
//         li.timestamp += reward_duration;
//     });
//
//     staking.distribute_rewards();
//
//     staking.unbond(&user, &1_000, &0);
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
//     // one more time to make sure that calculations during unbond aren't off
//     staking.withdraw_rewards(&user);
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
// }
//
// #[should_panic(
//     expected = "Stake: Create distribution flow: Distribution for this reward token exists!"
// )]
// #[test]
// fn panic_when_adding_same_distribution_twice() {
//     let env = Env::default();
//     env.mock_all_auths();
//
//     let admin = Address::generate(&env);
//     let manager = Address::generate(&env);
//     let lp_token = deploy_token_contract(&env, &admin);
//     let reward_token = deploy_token_contract(&env, &admin);
//
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &admin,
//         &50u32,
//     );
//
//     staking.create_distribution_flow(
//         &admin,
//         &reward_token.address,
//         &BytesN::from_array(&env, &[1; 32]),
//         &10,
//         &100,
//         &1,
//     );
//     staking.create_distribution_flow(
//         &admin,
//         &reward_token.address,
//         &BytesN::from_array(&env, &[1; 32]),
//         &10,
//         &100,
//         &1,
//     );
// }
//
// // Error #12 at stake_rewards: InvalidMaxComplexity = 12
// #[should_panic(expected = "Error(Contract, #12)")]
// #[test]
// fn panic_when_funding_distribution_with_curve_too_complex() {
//     const DISTRIBUTION_MAX_COMPLEXITY: u32 = 3;
//     const FIVE_MINUTES: u64 = 300;
//     const TEN_MINUTES: u64 = 600;
//     const ONE_WEEK: u64 = 604_800;
//
//     let env = Env::default();
//     env.mock_all_auths();
//     env.budget().reset_unlimited();
//
//     let admin = Address::generate(&env);
//     let manager = Address::generate(&env);
//     let lp_token = deploy_token_contract(&env, &admin);
//     let reward_token = deploy_token_contract(&env, &admin);
//
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &admin,
//         &DISTRIBUTION_MAX_COMPLEXITY,
//     );
//
//     staking.create_distribution_flow(
//         &admin,
//         &reward_token.address,
//         &BytesN::from_array(&env, &[1; 32]),
//         &10,
//         &100,
//         &1,
//     );
//
//     reward_token.mint(&admin, &10000);
//
//     staking.fund_distribution(&0, &FIVE_MINUTES, &reward_token.address, &1000);
//     staking.fund_distribution(&FIVE_MINUTES, &TEN_MINUTES, &reward_token.address, &1000);
//     staking.fund_distribution(&TEN_MINUTES, &ONE_WEEK, &reward_token.address, &1000);
//     staking.fund_distribution(
//         &(ONE_WEEK + 1),
//         &(ONE_WEEK + 3),
//         &reward_token.address,
//         &1000,
//     );
//     staking.fund_distribution(
//         &(ONE_WEEK + 3),
//         &(ONE_WEEK + 5),
//         &reward_token.address,
//         &1000,
//     );
//     staking.fund_distribution(
//         &(ONE_WEEK + 6),
//         &(ONE_WEEK + 7),
//         &reward_token.address,
//         &1000,
//     );
//     staking.fund_distribution(
//         &(ONE_WEEK + 8),
//         &(ONE_WEEK + 9),
//         &reward_token.address,
//         &1000,
//     );
// }

#[test]
fn multiple_equal_users_with_different_multipliers() {
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
    staking.create_distribution_flow(&admin, &reward_token.address);

    // first user bonds at timestamp 0
    // he will get 100% of his rewards
    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    staking.bond(&user1, &10_000);

    let fifteen_days = 3600 * 24 * 15;
    env.ledger().with_mut(|li| {
        li.timestamp = fifteen_days;
    });

    // user2 will receive 75% of his reward
    let user2 = Address::generate(&env);
    lp_token.mint(&user2, &10_000);
    staking.bond(&user2, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp = fifteen_days * 2;
    });

    // user3 will receive 50% of his reward
    let user3 = Address::generate(&env);
    lp_token.mint(&user3, &10_000);
    staking.bond(&user3, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp = fifteen_days * 3;
    });

    // user4 will receive 25% of his reward
    let user4 = Address::generate(&env);
    lp_token.mint(&user4, &10_000);
    staking.bond(&user4, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp = fifteen_days * 4;
    });

    reward_token.mint(&admin, &1_000_000);
    staking.distribute_rewards(&admin, &1_000_000, &reward_token.address);

    // The way it works - contract will treat all the funds as distributed, and the amount
    // that was not sent due to low staking bonus stays on the contract

    staking.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 250_000);
    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 187_500);
    staking.withdraw_rewards(&user3);
    assert_eq!(reward_token.balance(&user3), 125_000);
    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 62_500);
}

#[test]
fn distribute_rewards_daily_multiple_times_different_stakes() {
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
    staking.create_distribution_flow(&admin, &reward_token.address);

    // first user bonds at timestamp 0
    // he will get 100% of his rewards
    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    staking.bond(&user1, &10_000);

    let fifteen_days = 3600 * 24 * 15;
    env.ledger().with_mut(|li| {
        li.timestamp = fifteen_days;
    });

    // user2 will receive 75% of his reward
    let user2 = Address::generate(&env);
    lp_token.mint(&user2, &10_000);
    staking.bond(&user2, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp = fifteen_days * 2;
    });

    // user3 will receive 50% of his reward
    let user3 = Address::generate(&env);
    lp_token.mint(&user3, &10_000);
    staking.bond(&user3, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp = fifteen_days * 3;
    });

    // user4 will receive 25% of his reward
    let user4 = Address::generate(&env);
    lp_token.mint(&user4, &10_000);
    staking.bond(&user4, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp = fifteen_days * 4;
    });

    reward_token.mint(&admin, &4_000_000);
    staking.distribute_rewards(&admin, &1_000_000, &reward_token.address);

    // The way it works - contract will treat all the funds as distributed, and the amount
    // that was not sent due to low staking bonus stays on the contract

    staking.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 250_000);
    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 187_500);
    staking.withdraw_rewards(&user3);
    assert_eq!(reward_token.balance(&user3), 125_000);
    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 62_500);

    // 24h later
    env.ledger().with_mut(|li| {
        li.timestamp += 3600 * 24;
    });
    staking.distribute_rewards(&admin, &1_000_000, &reward_token.address);

    staking.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 500_000);
    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 379_166);
    staking.withdraw_rewards(&user3);
    assert_eq!(reward_token.balance(&user3), 254_166);
    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 129_166);

    // 24h later
    env.ledger().with_mut(|li| {
        li.timestamp += 3600 * 24;
    });
    staking.distribute_rewards(&admin, &1_000_000, &reward_token.address);

    staking.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 750_000);
    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 574_999);
    staking.withdraw_rewards(&user3);
    assert_eq!(reward_token.balance(&user3), 387_499);
    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 199_999);

    // 24h later
    env.ledger().with_mut(|li| {
        li.timestamp += 3600 * 24;
    });
    staking.distribute_rewards(&admin, &1_000_000, &reward_token.address);

    staking.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 1_000_000);
    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 774_999);
    staking.withdraw_rewards(&user3);
    assert_eq!(reward_token.balance(&user3), 524_999);
    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 274_999);
}

#[test]
fn distribute_rewards_daily_multiple_times_same_stakes() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let user = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let user4 = Address::generate(&env);

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
    staking.create_distribution_flow(&admin, &reward_token.address);

    // bond tokens for users; each user has a different amount staked
    env.ledger().with_mut(|li| {
        li.timestamp = 1706968777;
    });

    lp_token.mint(&user, &1000);
    staking.bond(&user, &1000);

    env.ledger().with_mut(|li| {
        li.timestamp = 1714741177;
    });

    lp_token.mint(&user2, &2000);
    staking.bond(&user2, &2000);

    env.ledger().with_mut(|li| {
        li.timestamp = 1714741177;
    });

    lp_token.mint(&user3, &3000);
    staking.bond(&user3, &3000);
    env.ledger().with_mut(|li| {
        li.timestamp = 1715741177;
    });

    lp_token.mint(&user4, &4000);
    staking.bond(&user4, &4000);

    // simulate moving forward 60 days for the full APR multiplier
    env.ledger().with_mut(|li| {
        li.timestamp += SIXTY_DAYS;
    });

    reward_token.mint(&admin, &4_000_000);
    staking.distribute_rewards(&admin, &1_000_000, &reward_token.address);

    // The way it works - contract will treat all the funds as distributed, and the amount
    // that was not sent due to low staking bonus stays on the contract

    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), 100_000);
    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 200_000);
    staking.withdraw_rewards(&user3);
    assert_eq!(reward_token.balance(&user3), 300_000);
    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 400_000);

    // 24h later
    env.ledger().with_mut(|li| {
        li.timestamp += 3600 * 24;
    });
    staking.distribute_rewards(&admin, &1_000_000, &reward_token.address);

    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), 200_000);
    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 400_000);
    staking.withdraw_rewards(&user3);
    assert_eq!(reward_token.balance(&user3), 600_000);
    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 800_000);

    // 24h later
    env.ledger().with_mut(|li| {
        li.timestamp += 3600 * 24;
    });
    staking.distribute_rewards(&admin, &1_000_000, &reward_token.address);

    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), 300_000);
    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 600_000);
    staking.withdraw_rewards(&user3);
    assert_eq!(reward_token.balance(&user3), 900_000);
    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 1_200_000);

    // 24h later
    env.ledger().with_mut(|li| {
        li.timestamp += 3600 * 24;
    });
    staking.distribute_rewards(&admin, &1_000_000, &reward_token.address);

    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), 400_000);
    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 800_000);
    staking.withdraw_rewards(&user3);
    assert_eq!(reward_token.balance(&user3), 1_200_000);
    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 1_600_000);

    assert_eq!(reward_token.balance(&staking.address), 0);
}

#[test]
fn add_distribution_and_distribute_reward_in_chunks() {
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

    staking.create_distribution_flow(&admin, &reward_token.address);

    let reward_amount: i128 = 100_000;
    reward_token.mint(&admin, &reward_amount);

    // Bond tokens for the user
    lp_token.mint(&user, &1000);
    staking.bond(&user, &1000);

    // Simulate moving forward 60 days for full APR multiplier
    env.ledger().with_mut(|li| {
        li.timestamp = SIXTY_DAYS;
    });

    // Distribute rewards daily for 60 days
    for _ in 0..60 {
        staking.distribute_rewards(&admin, &(reward_amount / 60i128), &reward_token.address);
        env.ledger().with_mut(|li| {
            li.timestamp += 3600 * 24; // Move forward 1 day
        });
    }

    // Query withdrawable rewards in the first chunk (e.g., chunk_size = 30 days)
    let chunk_size = 30u32;
    let withdrawable_chunk_1 = staking.query_withdrawable_rewards_ch(&user, &chunk_size, &None);

    assert_eq!(
        withdrawable_chunk_1,
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 49_980_u128 // 50% of total rewards
                }
            ]
        }
    );

    // Withdraw the first chunk of rewards
    staking.withdraw_rewards_chunks(&user, &reward_token.address, &chunk_size);
    assert_eq!(reward_token.balance(&user), 49_980);

    // Ensure `last_reward_time` was updated correctly
    assert_eq!(
        staking.query_staked(&user).last_reward_time,
        (chunk_size - 1) as u64 * SECONDS_PER_DAY
    );

    // Query withdrawable rewards for the second chunk
    let withdrawable_chunk_2 = staking.query_withdrawable_rewards_ch(
        &user,
        &chunk_size,
        &Some(chunk_size as u64 * SECONDS_PER_DAY),
    );

    assert_eq!(
        withdrawable_chunk_2,
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 49_980_u128 // Remaining 50%
                }
            ]
        }
    );

    // Withdraw the second chunk of rewards
    staking.withdraw_rewards_chunks(&user, &reward_token.address, &chunk_size);
    assert_eq!(reward_token.balance(&user), 99_960);

    // Ensure no more rewards are withdrawable
    let withdrawable_chunk_3 = staking.query_withdrawable_rewards_ch(
        &user,
        &chunk_size,
        &Some(2 * (chunk_size - 1) as u64 * SECONDS_PER_DAY),
    );
    assert_eq!(
        withdrawable_chunk_3,
        WithdrawableRewardsResponse {
            rewards: vec![&env]
        }
    );
}
