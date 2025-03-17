use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{
    msg::ConfigResponse,
    storage::Config,
    tests::setup::{deploy_staking_contract, deploy_token_contract},
};

const DEFAULT_COMPLEXITY: u32 = 7;

#[test]
fn should_update_config() {
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
                lp_token: lp_token.address.clone(),
                min_bond: 1_000i128,
                min_reward: 1_000i128,
                manager: manager.clone(),
                owner: owner.clone(),
                max_complexity: 7,
            }
        }
    );

    let new_min_bond = 2_000i128;
    let new_min_reward = 2_000i128;

    let new_config = Config {
        lp_token: lp_token.address,
        min_bond: new_min_bond,
        min_reward: new_min_reward,
        manager,
        owner,
        max_complexity: DEFAULT_COMPLEXITY,
    };

    staking.update_config(
        &None,
        &Some(new_min_bond),
        &Some(new_min_reward),
        &None,
        &None,
        &None,
    );

    assert_eq!(new_config, staking.query_config().config)
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn update_config_should_fail_when_not_authorized() {
    let env = Env::default();

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
                lp_token: lp_token.address.clone(),
                min_bond: 1_000i128,
                min_reward: 1_000i128,
                manager: manager.clone(),
                owner: owner.clone(),
                max_complexity: 7,
            }
        }
    );

    staking.update_config(
        &Some(Address::generate(&env)),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
}

#[test]
fn should_update_admin() {
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

    let admin_result = staking.query_admin();
    assert_eq!(admin, admin_result);

    let new_admin = Address::generate(&env);

    staking.update_admin(&new_admin);

    assert_eq!(new_admin, staking.query_admin())
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn update_admin_should_panic_when_unauthorized() {
    let env = Env::default();

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

    let admin_result = staking.query_admin();
    assert_eq!(admin, admin_result);

    let new_admin = Address::generate(&env);

    staking.update_admin(&new_admin);
}
