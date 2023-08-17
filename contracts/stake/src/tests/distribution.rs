use pretty_assertions::assert_eq;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env,
};

use super::setup::{deploy_staking_contract, deploy_token_contract};

use crate::error::ContractError::{StakeLessThenMinBond, StakeNotFound};
use crate::{
    msg::{WithdrawableReward, WithdrawableRewardsResponse},
    storage::{Config, Stake},
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
