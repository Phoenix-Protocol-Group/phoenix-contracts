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

// #[test]
// fn initializing_contract_sets_total_staked_var() {
//     let env = Env::default();
//     env.mock_all_auths();
//
//     let admin = Address::generate(&env);
//     let manager = Address::generate(&env);
//     let owner = Address::generate(&env);
//     let lp_token = deploy_token_contract(&env, &admin);
//
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &owner,
//         &DEFAULT_COMPLEXITY,
//     );
//
//     assert_eq!(staking.query_total_staked(), 0);
// }
//
// #[test]
// #[should_panic(expected = "Stake: Remove stake: Stake not found")]
// fn unbond_wrong_user_stake_not_found() {
//     let env = Env::default();
//     env.mock_all_auths();
//
//     let admin = Address::generate(&env);
//     let user = Address::generate(&env);
//     let user2 = Address::generate(&env);
//     let manager = Address::generate(&env);
//     let owner = Address::generate(&env);
//     let lp_token = deploy_token_contract(&env, &admin);
//
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &owner,
//         &DEFAULT_COMPLEXITY,
//     );
//
//     lp_token.mint(&user, &35_000);
//     lp_token.mint(&user2, &10_000);
//
//     env.ledger().with_mut(|li| {
//         li.timestamp = ONE_DAY;
//     });
//     staking.bond(&user, &10_000);
//     env.ledger().with_mut(|li| {
//         li.timestamp += ONE_DAY;
//     });
//     staking.bond(&user, &10_000);
//     staking.bond(&user2, &10_000);
//
//     assert_eq!(lp_token.balance(&user), 15_000);
//     assert_eq!(lp_token.balance(&user2), 0);
//     assert_eq!(lp_token.balance(&staking.address), 30_000);
//
//     let non_existing_timestamp = ONE_DAY / 2;
//     staking.unbond(&user2, &10_000, &non_existing_timestamp);
// }
//
// #[test]
// fn pay_rewards_during_unbond() {
//     const STAKED_AMOUNT: i128 = 1_000;
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
//     let staking = deploy_staking_contract(
//         &env,
//         admin.clone(),
//         &lp_token.address,
//         &manager,
//         &owner,
//         &DEFAULT_COMPLEXITY,
//     );
//
//     lp_token.mint(&user, &10_000);
//     reward_token.mint(&admin, &10_000);
//
//     env.ledger().with_mut(|li| {
//         li.timestamp = ONE_WEEK;
//     });
//
//     staking.create_distribution_flow(&manager, &reward_token.address);
//     staking.fund_distribution(
//         &admin,
//         &ONE_WEEK,
//         &10_000u64,
//         &reward_token.address,
//         &10_000,
//     );
//
//     env.ledger().with_mut(|li| {
//         li.timestamp = ONE_WEEK + 5_000;
//     });
//     staking.bond(&user, &STAKED_AMOUNT);
//
//     staking.distribute_rewards();
//
//     // user has bonded for 5_000 time, initial rewards are 10_000
//     // so user should have 5_000 rewards
//     // 5_000 rewards are still undistributed
//     assert_eq!(
//         staking.query_undistributed_rewards(&reward_token.address),
//         5_000
//     );
//     assert_eq!(
//         staking
//             .query_withdrawable_rewards(&user)
//             .rewards
//             .iter()
//             .map(|reward| reward.reward_amount)
//             .sum::<u128>(),
//         5_000
//     );
//     assert_eq!(reward_token.balance(&user), 0);
//     staking.unbond(&user, &STAKED_AMOUNT, &(ONE_WEEK + 5_000));
//     assert_eq!(reward_token.balance(&user), 5_000);
// }
//
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
