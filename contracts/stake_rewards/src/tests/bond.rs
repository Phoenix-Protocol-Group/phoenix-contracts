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

    let start_timestamp = 100;
    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp;
    });

    reward_token.mint(&admin, &1_000_000);
    let reward_duration = 600;
    staking_rewards.fund_distribution(&admin, &start_timestamp, &reward_duration, &1_000_000);

    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    assert_eq!(lp_token.balance(&user1), 10_000);
    assert_eq!(lp_token.balance(&staking.address), 0);
    assert_eq!(staking.query_config().config.lp_token, lp_token.address);
    staking.bond(&user1, &10_000);

    staking_rewards.calculate_bond(&user1);

    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp + 300; // move to a middle of distribution
    });

    staking_rewards.distribute_rewards();

    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        500_000 // half of the reward are undistributed
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        500_000
    );

    staking_rewards.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 500_000);

    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp + reward_duration; // move to the end of the distribution
    });

    staking_rewards.distribute_rewards();

    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        0
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        1_000_000
    );

    staking_rewards.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 1_000_000);
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

    let start_timestamp = 100;
    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp;
    });

    reward_token.mint(&admin, &1_000_000);
    let reward_duration = 600;
    staking_rewards.fund_distribution(&admin, &start_timestamp, &reward_duration, &1_000_000);

    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    assert_eq!(lp_token.balance(&user1), 10_000);
    assert_eq!(lp_token.balance(&staking.address), 0);
    assert_eq!(staking.query_config().config.lp_token, lp_token.address);
    staking.bond(&user1, &10_000);

    staking_rewards.calculate_bond(&user1);

    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp + 300; // move to a middle of distribution
    });

    staking_rewards.distribute_rewards();

    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        500_000 // half of the reward are undistributed
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        500_000
    );

    staking_rewards.withdraw_rewards(&user1);
    assert_eq!(reward_token.balance(&user1), 500_000);

    // now calculate unbond and unbond tokens, which should result
    // in the rest of the reward being undistributed

    staking_rewards.calculate_unbond(&user1);
    staking.unbond(&user1, &10_000, &start_timestamp);

    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp + reward_duration; // move to the end of the distribution
    });

    staking_rewards.distribute_rewards();

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

    let start_timestamp = 100;
    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp;
    });

    reward_token.mint(&admin, &1_000_000);
    let reward_duration = 600;
    staking_rewards.fund_distribution(&admin, &start_timestamp, &reward_duration, &1_000_000);

    let user1 = Address::generate(&env);
    lp_token.mint(&user1, &10_000);
    assert_eq!(lp_token.balance(&user1), 10_000);
    assert_eq!(lp_token.balance(&staking.address), 0);
    assert_eq!(staking.query_config().config.lp_token, lp_token.address);
    staking.bond(&user1, &10_000);

    staking_rewards.calculate_bond(&user1);

    env.ledger().with_mut(|li| {
        li.timestamp = start_timestamp + reward_duration; // move to the end of the distribution
    });

    staking_rewards.distribute_rewards();

    assert_eq!(
        staking_rewards.query_undistributed_reward(&reward_token.address),
        0
    );
    assert_eq!(
        staking_rewards.query_distributed_reward(&reward_token.address),
        1_000_000
    );

    // unbonding and automatically withdraws rewards
    staking_rewards.calculate_unbond(&user1);
    staking.unbond(&user1, &10_000, &start_timestamp);
    assert_eq!(reward_token.balance(&user1), 1_000_000);
}

// #[should_panic(
//     expected = "Stake: initialize: Minimum amount of lp share tokens to bond can not be smaller or equal to 0"
// )]
// #[test]
// fn initialize_staking_contract_should_panic_when_min_bond_invalid() {
//     let env = Env::default();
//     env.mock_all_auths();
//
//     let staking = StakingClient::new(&env, &env.register_contract(None, Staking {}));
//
//     staking.initialize(
//         &Address::generate(&env),
//         &Address::generate(&env),
//         &0,
//         &1_000,
//         &Address::generate(&env),
//         &Address::generate(&env),
//         &DEFAULT_COMPLEXITY,
//     );
// }
//
// #[should_panic(expected = "Stake: initialize: min_reward must be bigger than 0!")]
// #[test]
// fn initialize_staking_contract_should_panic_when_min_rewards_invalid() {
//     let env = Env::default();
//     env.mock_all_auths();
//
//     let staking = StakingClient::new(&env, &env.register_contract(None, Staking {}));
//
//     staking.initialize(
//         &Address::generate(&env),
//         &Address::generate(&env),
//         &1_000,
//         &0,
//         &Address::generate(&env),
//         &Address::generate(&env),
//         &DEFAULT_COMPLEXITY,
//     );
// }
//
// #[should_panic(expected = "Stake: initialize: max_complexity must be bigger than 0!")]
// #[test]
// fn initialize_staking_contract_should_panic_when_max_complexity_invalid() {
//     let env = Env::default();
//     env.mock_all_auths();
//
//     let staking = StakingClient::new(&env, &env.register_contract(None, Staking {}));
//
//     staking.initialize(
//         &Address::generate(&env),
//         &Address::generate(&env),
//         &1_000,
//         &1_000,
//         &Address::generate(&env),
//         &Address::generate(&env),
//         &0u32,
//     );
// }
