extern crate std;

use pretty_assertions::assert_eq;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Ledger},
    vec, Address, Env, IntoVal, Symbol, Vec,
};

use super::setup::{deploy_staking_contract, deploy_token_contract};

use crate::{
    contract::{Staking, StakingClient},
    msg::{ConfigResponse, StakedResponse},
    storage::{Config, Stake},
    tests::setup::{ONE_DAY, ONE_WEEK},
};

const DEFAULT_COMPLEXITY: u32 = 7;

#[test]
fn initialize_staking_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &DEFAULT_COMPLEXITY,
    );

    let response = staking.query_config();
    assert_eq!(
        response,
        ConfigResponse {
            config: Config {
                lp_token: lp_token.address,
                min_bond: 1_000i128,
                min_reward: 1_000i128,
                manager,
                owner,
                max_complexity: 7,
            }
        }
    );

    let response = staking.query_admin();
    assert_eq!(response, admin);
}

#[test]
#[should_panic(expected = "Stake: Initialize: initializing contract twice is not allowed")]
fn test_deploying_stake_twice_should_fail() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    let first = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &DEFAULT_COMPLEXITY,
    );

    first.initialize(
        &admin,
        &lp_token.address,
        &100i128,
        &50i128,
        &manager,
        &owner,
        &DEFAULT_COMPLEXITY,
    );
}

#[test]
#[should_panic = "Stake: Bond: Trying to stake less than minimum required"]
fn bond_too_few() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &DEFAULT_COMPLEXITY,
    );

    lp_token.mint(&user, &999);

    staking.bond(&user, &999);
}

#[test]
fn bond_simple() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &DEFAULT_COMPLEXITY,
    );

    env.ledger().with_mut(|li| {
        li.timestamp = ONE_WEEK;
    });

    lp_token.mint(&user, &10_000);

    staking.bond(&user, &10_000);

    assert_eq!(
        env.auths(),
        [(
            user.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    staking.address.clone(),
                    Symbol::new(&env, "bond"),
                    (&user.clone(), 10_000i128,).into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        lp_token.address.clone(),
                        symbol_short!("transfer"),
                        (&user, &staking.address.clone(), 10_000i128).into_val(&env)
                    )),
                    sub_invocations: std::vec![],
                },],
            }
        ),]
    );

    let bonds = staking.query_staked(&user).stakes;
    assert_eq!(
        bonds,
        vec![
            &env,
            Stake {
                stake: 10_000,
                stake_timestamp: ONE_WEEK,
            }
        ]
    );
    assert_eq!(staking.query_total_staked(), 10_000);

    assert_eq!(lp_token.balance(&user), 0);
    assert_eq!(lp_token.balance(&staking.address), 10_000);
}

#[test]
fn unbond_simple() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let user2 = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &DEFAULT_COMPLEXITY,
    );

    lp_token.mint(&user, &35_000);
    lp_token.mint(&user2, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp += ONE_DAY;
    });
    staking.bond(&user, &10_000);
    env.ledger().with_mut(|li| {
        li.timestamp += ONE_DAY;
    });
    staking.bond(&user, &10_000);
    staking.bond(&user2, &10_000);
    env.ledger().with_mut(|li| {
        li.timestamp += ONE_DAY;
    });
    staking.bond(&user, &15_000);

    assert_eq!(staking.query_staked(&user).stakes.len(), 3);
    assert_eq!(lp_token.balance(&user), 0);
    assert_eq!(lp_token.balance(&staking.address), 45_000);

    staking.unbond(&user, &10_000, &(ONE_DAY + ONE_DAY));

    assert_eq!(
        env.auths(),
        [(
            user.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    staking.address.clone(),
                    Symbol::new(&env, "unbond"),
                    (&user.clone(), 10_000i128, (ONE_DAY + ONE_DAY)).into_val(&env),
                )),
                sub_invocations: std::vec![],
            }
        ),]
    );

    let bonds = staking.query_staked(&user).stakes;
    assert_eq!(
        bonds,
        vec![
            &env,
            Stake {
                stake: 10_000,
                stake_timestamp: ONE_DAY,
            },
            Stake {
                stake: 15_000,
                stake_timestamp: 3 * ONE_DAY,
            }
        ]
    );
    assert_eq!(staking.query_total_staked(), 35_000);

    assert_eq!(lp_token.balance(&user), 10_000);
    assert_eq!(lp_token.balance(&user2), 0);
    assert_eq!(lp_token.balance(&staking.address), 35_000);
}

#[test]
fn initializing_contract_sets_total_staked_var() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &DEFAULT_COMPLEXITY,
    );

    assert_eq!(staking.query_total_staked(), 0);
}

#[test]
#[should_panic(expected = "Stake: Remove stake: Stake not found")]
fn unbond_wrong_user_stake_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let user2 = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &owner,
        &DEFAULT_COMPLEXITY,
    );

    lp_token.mint(&user, &35_000);
    lp_token.mint(&user2, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp = ONE_DAY;
    });
    staking.bond(&user, &10_000);
    env.ledger().with_mut(|li| {
        li.timestamp += ONE_DAY;
    });
    staking.bond(&user, &10_000);
    staking.bond(&user2, &10_000);

    assert_eq!(lp_token.balance(&user), 15_000);
    assert_eq!(lp_token.balance(&user2), 0);
    assert_eq!(lp_token.balance(&staking.address), 30_000);

    let non_existing_timestamp = ONE_DAY / 2;
    staking.unbond(&user2, &10_000, &non_existing_timestamp);
}

#[test]
fn pay_rewards_during_unbond() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let full_bonding_multiplier = ONE_DAY * 60;

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
        &DEFAULT_COMPLEXITY,
    );

    lp_token.mint(&user, &10_000);
    reward_token.mint(&admin, &20_000);

    let staked = 1_000;
    staking.bond(&user, &staked);

    // Move so that user would have 100% APR from bonding after 60 days
    env.ledger().with_mut(|li| {
        li.timestamp = full_bonding_multiplier;
    });

    staking.create_distribution_flow(&admin, &reward_token.address);

    // simulate passing 20 days and distributing 1000 tokens each day
    for _ in 0..20 {
        staking.distribute_rewards(&admin, &1_000, &reward_token.address);
        env.ledger().with_mut(|li| {
            li.timestamp += 3600 * 24;
        });
    }

    assert_eq!(
        staking
            .query_withdrawable_rewards(&user)
            .rewards
            .iter()
            .map(|reward| reward.reward_amount)
            .sum::<u128>(),
        20_000
    );
    assert_eq!(reward_token.balance(&user), 0);

    // we first have to withdraw_rewards _before_ unbonding
    // as this messes up with the reward calculation
    // if we unbond first then we get no rewards
    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), 20_000);

    // user bonded at timestamp 0
    staking.unbond(&user, &staked, &0);
    assert_eq!(lp_token.balance(&staking.address), 0);
    assert_eq!(lp_token.balance(&user), 9000 + staked);
    assert_eq!(
        staking.query_staked(&user),
        StakedResponse {
            stakes: Vec::new(&env),
            total_stake: 0i128,
            last_reward_time: 6_912_000
        }
    );
}

#[should_panic(
    expected = "Stake: initialize: Minimum amount of lp share tokens to bond can not be smaller or equal to 0"
)]
#[test]
fn initialize_staking_contract_should_panic_when_min_bond_invalid() {
    let env = Env::default();
    env.mock_all_auths();

    let staking = StakingClient::new(&env, &env.register_contract(None, Staking {}));

    staking.initialize(
        &Address::generate(&env),
        &Address::generate(&env),
        &0,
        &1_000,
        &Address::generate(&env),
        &Address::generate(&env),
        &DEFAULT_COMPLEXITY,
    );
}

#[should_panic(expected = "Stake: initialize: min_reward must be bigger than 0!")]
#[test]
fn initialize_staking_contract_should_panic_when_min_rewards_invalid() {
    let env = Env::default();
    env.mock_all_auths();

    let staking = StakingClient::new(&env, &env.register_contract(None, Staking {}));

    staking.initialize(
        &Address::generate(&env),
        &Address::generate(&env),
        &1_000,
        &0,
        &Address::generate(&env),
        &Address::generate(&env),
        &DEFAULT_COMPLEXITY,
    );
}

#[should_panic(expected = "Stake: initialize: max_complexity must be bigger than 0!")]
#[test]
fn initialize_staking_contract_should_panic_when_max_complexity_invalid() {
    let env = Env::default();
    env.mock_all_auths();

    let staking = StakingClient::new(&env, &env.register_contract(None, Staking {}));

    staking.initialize(
        &Address::generate(&env),
        &Address::generate(&env),
        &1_000,
        &1_000,
        &Address::generate(&env),
        &Address::generate(&env),
        &0u32,
    );
}
