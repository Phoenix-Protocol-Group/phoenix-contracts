use soroban_sdk::{testutils::Address as _, Address, Env};

use super::setup::{deploy_staking_contract, deploy_token_contract};
use crate::{msg::ConfigResponse, storage::Config, token_contract};

#[test]
fn initializa_staking_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let lp_token = deploy_token_contract(&env, &admin);

    let staking = deploy_staking_contract(&env, admin, &lp_token.address);

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
}
