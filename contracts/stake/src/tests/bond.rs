use pretty_assertions::assert_eq;
use soroban_sdk::arbitrary::std::dbg;
use soroban_sdk::{testutils::{Address as _, Ledger}, vec, Address, Env, Vec};

use super::setup::{deploy_staking_contract, deploy_token_contract};

use crate::error::ContractError::{StakeLessThenMinBond, StakeNotFound};
use crate::{
    msg::ConfigResponse,
    storage::{Config, Stake},
};

#[test]
fn initializa_staking_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    let response = staking.query_config();
    assert_eq!(
        response,
        ConfigResponse {
            config: Config {
                lp_token: lp_token.address,
                token_per_power: 1u128,
                min_bond: 1_000i128,
                max_distributions: 7u32,
                min_reward: 1_000i128
            }
        }
    );

    let response = staking.query_admin();
    assert_eq!(response, admin);
}

#[test]
fn bond_too_few() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user = Address::random(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    lp_token.mint(&user, &999);

    assert_eq!(staking.try_bond(&user, &999), Err(Ok(StakeLessThenMinBond)));
}

#[test]
#[should_panic = "HostError: Error(Value, InvalidInput)"]
fn bond_not_having_tokens() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user = Address::random(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    // fails with error[E0600]: cannot apply unary operator `!` to type `()`; not sure if fixing this
    // won't be too hacky
    // assert_with_error!(&env, staking.bond(&user, &10_000), StakeLessThenMinBond);

    // fails with error[E0423]: cannot initialize a tuple struct which contains private fields
    // the way it is commented now; Basically I'm not sure for the correct import of `Error`
    // tried all the variants and this one hit closest to home.
    // assert_eq!(staking.try_bond(&user, &10_000i128), Err(Err(Error(Val::default()))))

    // For now I'm leaving it with should_panic macro
    staking.bond(&user, &10_000);
}

#[test]
fn bond_simple() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user = Address::random(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    lp_token.mint(&user, &10_000);

    staking.bond(&user, &10_000);

    let bonds = staking.query_staked(&user).stakes;
    assert_eq!(
        bonds,
        vec![
            &env,
            Stake {
                stake: 10_000,
                stake_timestamp: 0,
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

    let admin = Address::random(&env);
    let user = Address::random(&env);
    let user2 = Address::random(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    lp_token.mint(&user, &35_000);
    lp_token.mint(&user2, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp = 2000;
    });
    staking.bond(&user, &10_000);
    env.ledger().with_mut(|li| {
        li.timestamp = 4000;
    });
    staking.bond(&user, &10_000);
    staking.bond(&user2, &10_000);
    env.ledger().with_mut(|li| {
        li.timestamp = 4000;
    });
    staking.bond(&user, &15_000);

    assert_eq!(staking.query_staked(&user).stakes.len(), 3);
    assert_eq!(lp_token.balance(&user), 0);
    assert_eq!(lp_token.balance(&staking.address), 45_000);

    let stake_timestamp = 4000;
    staking.unbond(&user, &10_000, &stake_timestamp);

    let bonds = staking.query_staked(&user).stakes;
    assert_eq!(
        bonds,
        vec![
            &env,
            Stake {
                stake: 10_000,
                stake_timestamp: 2_000,
            },
            Stake {
                stake: 15_000,
                stake_timestamp: 4_000,
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

    let admin = Address::random(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    assert_eq!(staking.query_total_staked(), 0);
}

#[test]
fn unbond_wrong_user_stake_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user = Address::random(&env);
    let user2 = Address::random(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    lp_token.mint(&user, &35_000);
    lp_token.mint(&user2, &10_000);

    env.ledger().with_mut(|li| {
        li.timestamp = 2_000;
    });
    staking.bond(&user, &10_000);
    env.ledger().with_mut(|li| {
        li.timestamp = 4_000;
    });
    staking.bond(&user, &10_000);
    staking.bond(&user2, &10_000);

    assert_eq!(lp_token.balance(&user), 15_000);
    assert_eq!(lp_token.balance(&user2), 0);
    assert_eq!(lp_token.balance(&staking.address), 30_000);

    assert_eq!(
        staking.try_unbond(&user2, &10_000, &2_000),
        Err(Ok(StakeNotFound))
    );
}
