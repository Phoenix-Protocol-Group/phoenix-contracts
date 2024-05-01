use soroban_sdk::{testutils::Address as _, Address, Env, String};

use super::setup::{deploy_token_client, deploy_trader_client};

// make a test that initializes the contract succesffully
#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let contract_name = String::from_str(&env, "LXC/USDC");
    let token_a = deploy_token_client(&env, Address::generate(&env));
    let token_b = deploy_token_client(&env, Address::generate(&env));
    let pho_token = deploy_token_client(&env, Address::generate(&env));
    let max_spread_bps = &None::<u64>;

    let trader_client = deploy_trader_client(&env);
    trader_client.initialize(
        &admin,
        &contract_name,
        &(token_a.address.clone(), token_b.address.clone()),
        &pho_token.address,
        &max_spread_bps,
    );

    assert_eq!(trader_client.query_admin_address(), admin);
    assert_eq!(trader_client.query_contract_name(), contract_name);
    assert_eq!(
        trader_client.query_trading_pairs(),
        (token_a.address, token_b.address)
    );
}
