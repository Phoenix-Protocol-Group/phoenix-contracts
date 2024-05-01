use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::tests::setup::deploy_token_contract;

use super::setup::{deploy_and_init_lp_client, deploy_trader_client};

// make a test that initializes the contract succesffully
#[test]
fn initialize() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let contract_name = String::from_str(&env, "LXC/USDC");
    let token_a = deploy_token_contract(&env, &admin);
    let token_b = deploy_token_contract(&env, &admin);
    let pho_token = deploy_token_contract(&env, &admin);
    let max_spread_bps = &None::<u64>;

    let trader_client = deploy_trader_client(&env);
    trader_client.initialize(
        &admin,
        &contract_name,
        &(token_a.address.clone(), token_b.address.clone()),
        &pho_token.address,
        max_spread_bps,
    );

    assert_eq!(trader_client.query_admin_address(), admin);
    assert_eq!(trader_client.query_contract_name(), contract_name);
    assert_eq!(
        trader_client.query_trading_pairs(),
        (token_a.address, token_b.address)
    );
}

#[test]
fn trade_token() {
    let env = Env::default();

    env.mock_all_auths_allowing_non_root_auth();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let contract_name = String::from_str(&env, "XLM/USDC");
    let xlm_token = deploy_token_contract(&env, &admin);
    let usdc_token = deploy_token_contract(&env, &admin);
    let pho_token = deploy_token_contract(&env, &admin);

    xlm_token.mint(&admin, &1_010_000);
    usdc_token.mint(&admin, &1_010_000);
    pho_token.mint(&admin, &2_010_000);

    let trader_client = deploy_trader_client(&env);

    let xlm_pho_client = deploy_and_init_lp_client(
        &env,
        admin.clone(),
        xlm_token.address.clone(),
        pho_token.address.clone(),
        1_000_000,
        1_000_000,
    );

    soroban_sdk::testutils::arbitrary::std::dbg!(xlm_pho_client.query_pool_info());
    soroban_sdk::testutils::arbitrary::std::dbg!(xlm_pho_client.query_config());
    soroban_sdk::testutils::arbitrary::std::dbg!(
        pho_token.balance(&xlm_pho_client.address),
        xlm_token.balance(&xlm_pho_client.address),
        usdc_token.balance(&xlm_pho_client.address),
        xlm_token.balance(&admin),
    );

    trader_client.initialize(
        &admin,
        &contract_name,
        &(xlm_token.address.clone(), usdc_token.address.clone()),
        &pho_token.address,
        &None::<u64>,
    );

    trader_client.trade_token(
        &admin.clone(),
        &xlm_token.address.clone(),
        &xlm_pho_client.address,
        &Some(1_000),
    )
}
