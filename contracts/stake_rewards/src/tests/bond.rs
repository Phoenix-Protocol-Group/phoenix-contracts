use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

use super::setup::{deploy_staking_rewards_contract, deploy_token_contract};

#[test]
fn initialize_staking_rewards_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);

    assert_eq!(staking_rewards.query_admin(), admin);
    assert_eq!(staking.query_admin(), admin);
}

#[test]
fn calculate_bond_one_user() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);
    assert_eq!(staking.query_total_staked(), 0);

    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    assert_eq!(lp_token.balance(&user1), 10_000);
    assert_eq!(lp_token.balance(&staking.address), 0);
    assert_eq!(staking.query_config().config.lp_token, lp_token.address);
    staking.bond(&user1, &10_000);

    // we simulate full stake time
    let start_timestamp = 60 * 3600 * 24;
    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp;
    });

    reward_token.mint(&admin, &1_000_000);
    let reward_duration = 600;
    staking_rewards.fund_distribution(&start_timestamp, &reward_duration, &1_000_000);

    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp + 300; // move to a middle of distribution
    });

    staking.distribute_rewards();

    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        500_000 // half of the reward are undistributed
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        500_000
    );

    staking.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 500_000);

    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp + reward_duration; // move to the end of the distribution
    });

    staking.distribute_rewards();

    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        0
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        1_000_000
    );

    staking.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 1_000_000);
}

#[test]
fn calculate_bond_multiple_users() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);
    assert_eq!(staking.query_total_staked(), 0);

    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    staking.bond(&user1, &10_000);

    let user2 = Address::generate(&env);
    lp_token.mint(&user2, &20_000);
    staking.bond(&user2, &20_000);

    let user3 = Address::generate(&env);
    lp_token.mint(&user3, &30_000);
    staking.bond(&user3, &30_000);

    let user4 = Address::generate(&env);
    lp_token.mint(&user4, &40_000);
    staking.bond(&user4, &40_000);

    // now all users have 100% APR after 60 days of staking
    let start_timestamp = 3600 * 24 * 60;
    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp;
    });

    reward_token.mint(&admin, &1_000_000);
    let reward_duration = 500;
    staking_rewards.fund_distribution(&start_timestamp, &reward_duration, &1_000_000);

    env.ledger().with_mut(|li| {
        li.timestamp += 250; // move to a middle of distribution
    });

    staking.distribute_rewards();

    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        500_000 // half of the reward are undistributed
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        500_000
    );

    staking.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 50_000);
    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 100_000);
    staking.withdraw_rewards(&user3);
    assert_eq!(reward_token.balance(&user3), 150_000);
    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 200_000);

    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp + reward_duration; // move to the end of the distribution
    });

    staking.distribute_rewards();

    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        0
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        1_000_000
    );

    staking.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 100_000);
    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 200_000);
    staking.withdraw_rewards(&user3);
    assert_eq!(reward_token.balance(&user3), 300_000);
    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 400_000);
}

#[test]
fn calculate_unbond_one_user() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);
    assert_eq!(staking.query_total_staked(), 0);

    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    assert_eq!(lp_token.balance(&user1), 10_000);
    assert_eq!(lp_token.balance(&staking.address), 0);
    assert_eq!(staking.query_config().config.lp_token, lp_token.address);
    staking.bond(&user1, &10_000);

    // User has 100% APR after 60 days of staking
    let start_timestamp = 3600 * 24 * 60;
    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp;
    });

    reward_token.mint(&admin, &1_000_000);
    let reward_duration = 500;
    staking_rewards.fund_distribution(&start_timestamp, &reward_duration, &1_000_000);

    env.ledger().with_mut(|li| {
        li.timestamp += 250; // move to a middle of distribution
    });

    staking.distribute_rewards();

    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        500_000 // half of the reward are undistributed
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        500_000
    );

    staking.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 500_000);

    // now calculate unbond and unbond tokens, which should result
    // in the rest of the reward being undistributed

    staking.unbond(&user1, &10_000, &0);

    env.ledger().with_mut(|li| {
        li.timestamp += reward_duration; // move to the end of the distribution
    });

    staking.distribute_rewards();

    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        500_000
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        500_000
    );
}

#[test]
fn pay_rewards_during_calculate_unbond() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);
    assert_eq!(staking.query_total_staked(), 0);

    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    assert_eq!(lp_token.balance(&user1), 10_000);
    assert_eq!(lp_token.balance(&staking.address), 0);
    assert_eq!(staking.query_config().config.lp_token, lp_token.address);
    staking.bond(&user1, &10_000);

    // This simulates 100% APR for the bonded user
    let start_timestamp = 3600 * 24 * 60;
    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp;
    });

    reward_token.mint(&admin, &1_000_000);
    let reward_duration = 600;
    staking_rewards.fund_distribution(&start_timestamp, &reward_duration, &1_000_000);

    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp + reward_duration; // move to the end of the distribution
    });

    staking.distribute_rewards();

    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        0
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        1_000_000
    );

    // unbonding and automatically withdraws rewards
    staking.unbond(&user1, &10_000, &0);
    assert_eq!(reward_token.balance(&user1), 1_000_000);
}

#[test]
fn calculate_unbond_multiple_users() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);

    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    staking.bond(&user1, &10_000);

    let user2 = Address::generate(&env);
    lp_token.mint(&user2, &20_000);
    staking.bond(&user2, &20_000);

    let user3 = Address::generate(&env);
    lp_token.mint(&user3, &30_000);
    staking.bond(&user3, &30_000);

    let user4 = Address::generate(&env);
    lp_token.mint(&user4, &40_000);
    staking.bond(&user4, &40_000);

    // 60 days of staking simulates the full APR for bonded users
    let start_timestamp = 3600 * 24 * 60;
    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp;
    });

    reward_token.mint(&admin, &1_000_000);
    let reward_duration = 2000;
    staking_rewards.fund_distribution(&start_timestamp, &reward_duration, &1_000_000);

    env.ledger().with_mut(|li| {
        li.timestamp += 500; // move to a 1/4 of distribution
    });

    staking.distribute_rewards();

    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        750_000
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        250_000
    );

    // first user unbonds instead of withdrawing
    staking.unbond(&user1, &10_000, &0);
    assert_eq!(reward_token.balance(&user1), 25_000);

    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 50_000);
    staking.withdraw_rewards(&user3);
    assert_eq!(reward_token.balance(&user3), 75_000);
    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 100_000);

    env.ledger().with_mut(|li| {
        li.timestamp += 500; // move to the half of the distribution
    });

    staking.distribute_rewards();

    // 250_000 reward for 90_000 total staking points
    // user2 250 * 20 / 90 = 55.555
    // user3 250 * 30 / 90 = 83.333
    // user4 250 * 40 / 90 = 111.111

    // first user unbonds instead of withdrawing
    staking.unbond(&user2, &20_000, &0);
    assert_eq!(reward_token.balance(&user2), 50_000 + 55_555);

    staking.withdraw_rewards(&user3);
    assert_eq!(reward_token.balance(&user3), 75_000 + 83_333);
    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 100_000 + 111_111);

    env.ledger().with_mut(|li| {
        li.timestamp += 500; // move to the 3/4 of the distribution
    });

    staking.distribute_rewards();

    // 250_000 reward for 70_000 total staking points
    // user3 250 * 30 / 70 = 107.143
    // user4 250 * 40 / 70 = 142.857

    // third user unbonds instead of withdrawing
    staking.unbond(&user3, &30_000, &0);
    assert_eq!(reward_token.balance(&user3), 158_333 + 107_143);

    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 211_111 + 142_857);

    env.ledger().with_mut(|li| {
        li.timestamp += 500; // move to the end of the distribution
    });

    staking.distribute_rewards();

    // user4 is the only one left, so this time 250k goes to him

    // third user unbonds instead of withdrawing
    staking.unbond(&user4, &40_000, &0);
    assert_eq!(reward_token.balance(&user4), 353_968 + 250_000);

    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        0
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        1_000_000
    );
}

#[test]
fn multiple_equal_users_with_different_multipliers() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);

    let (staking, staking_rewards) =
        deploy_staking_rewards_contract(&env, &admin, &lp_token.address, &reward_token.address);

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
    // reward distribution starts at the latest timestamp and lasts just 1 second
    // the point is to prove that the multiplier works correctly
    staking_rewards.fund_distribution(&(fifteen_days * 4), &1, &1_000_000);

    env.ledger().with_mut(|li| {
        li.timestamp += 1;
    });

    staking.distribute_rewards();

    // The way it works - contract will treat all the funds as distributed, and the amount
    // that was not sent due to low staking bonus stays on the contract

    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        0
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        1_000_000
    );

    staking.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 250_000);
    staking.withdraw_rewards(&user2);
    assert_eq!(reward_token.balance(&user2), 187_500);
    staking.withdraw_rewards(&user3);
    assert_eq!(reward_token.balance(&user3), 125_000);
    staking.withdraw_rewards(&user4);
    assert_eq!(reward_token.balance(&user4), 62_500);
}
