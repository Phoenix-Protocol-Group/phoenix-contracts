use soroban_sdk::{testutils::Address as _, Address, Env};

use super::setup::{deploy_staking_contract, deploy_token_contract};
use crate::{msg::ConfigResponse, storage::Config};

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
                max_distributions: 7u32
            }
        }
    );

    let response = staking.query_admin();
    assert_eq!(response, admin);
}

#[test]
#[should_panic = "Trying to bond I128(999) which is less then minimum I128(1000) required!"]
fn bond_too_few() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user = Address::random(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    lp_token.mint(&user, &999);

    staking.bond(&user, &999);
}

#[test]
#[should_panic = "balance is not sufficient to spend: 0 < I128(10000)"]
fn bond_not_having_tokens() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user = Address::random(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin.clone(), &lp_token.address);

    staking.bond(&user, &10_000);
}
