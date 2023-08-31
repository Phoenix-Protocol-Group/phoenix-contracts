use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env,
};

use super::setup::{deploy_staking_contract, deploy_token_contract};

use crate::msg::{WithdrawableReward, WithdrawableRewardsResponse};

const DAY_IN_SECONDS: u64 = 60 * 60 * 24;

#[test]
fn four_users_same_stakes_different_multipliers() {
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

    // bond tokens for users; each user has a different amount staked
    lp_token.mint(&user, &1000);
    staking.bond(&user, &1000);
    lp_token.mint(&user2, &1000);
    staking.bond(&user2, &1000);
    lp_token.mint(&user3, &1000);
    staking.bond(&user3, &1000);
    lp_token.mint(&user4, &1000);
    staking.bond(&user4, &1000);

    env.ledger().with_mut(|li| {
        li.timestamp = 0;
    });

    // fund distribution that spans over 5 days
    let reward_amount: u128 = 1_000_000;
    reward_token.mint(&admin, &(reward_amount as i128));
    staking.fund_distribution(
        &admin,
        &0,
        &(DAY_IN_SECONDS * 4),
        &reward_token.address,
        &(reward_amount as i128),
    );

    env.ledger().with_mut(|li| {
        li.timestamp = DAY_IN_SECONDS;
    });
    staking.distribute_rewards();
    dbg!("\n\nfirst distribution");
    // user1 claims rewards everyday, after that increasing his multiplier to 1.005
    assert_eq!(
        staking.query_withdrawable_rewards(&user),
        WithdrawableRewardsResponse {
            rewards: vec![
                &env,
                WithdrawableReward {
                    reward_address: reward_token.address.clone(),
                    reward_amount: 62_500
                }
            ]
        }
    );
    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), 62_500);
    assert_eq!(
        staking.query_distributed_rewards(&reward_token.address),
        250_000
    );
    env.ledger().with_mut(|li| {
        li.timestamp = DAY_IN_SECONDS + 1;
    });
    staking.distribute_rewards();
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

    // env.ledger().with_mut(|li| {
    //     li.timestamp = DAY_IN_SECONDS * 2;
    // });
    // staking.distribute_rewards();
    // dbg!("\n\nsecond distribution");
    // // user1 claims rewards everyday, after that  at 1.01
    // staking.withdraw_rewards(&user);
    // // user2 claims rewards every two days, after that at 1.005
    // staking.withdraw_rewards(&user2);

    // env.ledger().with_mut(|li| {
    //     li.timestamp = DAY_IN_SECONDS * 3;
    // });
    // staking.distribute_rewards();
    // dbg!("\n\nthird distribution");
    // // user1 claims rewards everyday, after that at 1.015
    // staking.withdraw_rewards(&user);
    // // user3 claims rewards at 3rd day, after that at 1.005
    // staking.withdraw_rewards(&user3);

    // env.ledger().with_mut(|li| {
    //     li.timestamp = DAY_IN_SECONDS * 4;
    // });
    // dbg!("\n\nfourth distribution");
    // staking.distribute_rewards();
    // // user1 claims rewards everyday, after that at 1.02
    // staking.withdraw_rewards(&user);
    // // user2 claims rewards every second day, after that at 1.01
    // staking.withdraw_rewards(&user2);

    // env.ledger().with_mut(|li| {
    //     li.timestamp = DAY_IN_SECONDS * 5;
    // });
    // dbg!("\n\nfifth distribution");
    // staking.distribute_rewards();
    // staking.withdraw_rewards(&user);
    // staking.withdraw_rewards(&user2);
    // staking.withdraw_rewards(&user3);
    // // user 4 claims his rewards first time; his bonus multiplier is 1.005 after that withdrawal
    // staking.withdraw_rewards(&user4);

    // assert_eq!(
    //     staking.query_withdrawable_rewards(&user),
    //     WithdrawableRewardsResponse {
    //         rewards: vec![
    //             &env,
    //             WithdrawableReward {
    //                 reward_address: reward_token.address.clone(),
    //                 reward_amount: 11_293
    //             }
    //         ]
    //     }
    // );
    // assert_eq!(
    //     staking.query_withdrawable_rewards(&user2),
    //     WithdrawableRewardsResponse {
    //         rewards: vec![
    //             &env,
    //             WithdrawableReward {
    //                 reward_address: reward_token.address.clone(),
    //                 reward_amount: 11_293
    //             }
    //         ]
    //     }
    // );
    // assert_eq!(
    //     staking.query_withdrawable_rewards(&user3),
    //     WithdrawableRewardsResponse {
    //         rewards: vec![
    //             &env,
    //             WithdrawableReward {
    //                 reward_address: reward_token.address.clone(),
    //                 reward_amount: 11_292
    //             }
    //         ]
    //     }
    // );
    // assert_eq!(
    //     staking.query_withdrawable_rewards(&user4),
    //     WithdrawableRewardsResponse {
    //         rewards: vec![
    //             &env,
    //             WithdrawableReward {
    //                 reward_address: reward_token.address.clone(),
    //                 reward_amount: 11_293
    //             }
    //         ]
    //     }
    // );
    // assert_eq!(reward_token.balance(&user), 271_019);
    // assert_eq!(staking.query_staked(&user).current_rewards_bps, 2500);
    // assert_eq!(reward_token.balance(&user2), 248_434);
    // assert_eq!(staking.query_staked(&user2).current_rewards_bps, 1500);
    // assert_eq!(reward_token.balance(&user3), 237_142);
    // assert_eq!(staking.query_staked(&user3).current_rewards_bps, 1000);
    // assert_eq!(reward_token.balance(&user4), 225_849);
    // assert_eq!(staking.query_staked(&user4).current_rewards_bps, 500);

    // assert_eq!(staking.query_undistributed_rewards(&reward_token.address), 0);
    // assert_eq!(staking.query_distributed_rewards(&reward_token.address), reward_amount);

    // assert_eq!(reward_token.balance(&user) + reward_token.balance(&user2) + reward_token.balance(&user3) + reward_token.balance(&user4), reward_amount as i128);
}
