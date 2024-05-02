use soroban_sdk::{
    testutils::{arbitrary::std, Address as _},
    Address, Env, String,
};

use crate::{
    storage::{Asset, BalanceInfo},
    tests::setup::deploy_token_contract,
};

use super::setup::{deploy_and_init_lp_client, deploy_trader_client};

#[test]
fn initialize() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let contract_name = String::from_str(&env, "XLM/USDC");
    let xlm_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "XLM"),
    );
    let usdc_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "USD Coin"),
        &String::from_str(&env, "USDC"),
    );
    let pho_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "Phoenix"),
        &String::from_str(&env, "PHO"),
    );

    let trader_client = deploy_trader_client(&env);
    trader_client.initialize(
        &admin,
        &contract_name,
        &(xlm_token.address.clone(), usdc_token.address.clone()),
        &pho_token.address,
    );

    assert_eq!(trader_client.query_admin_address(), admin);
    assert_eq!(trader_client.query_contract_name(), contract_name);
    assert_eq!(
        trader_client.query_trading_pairs(),
        (xlm_token.address, usdc_token.address)
    );
}

#[test]
fn trade_token_and_transfer_token() {
    let env = Env::default();

    env.mock_all_auths_allowing_non_root_auth();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let rcpt = Address::generate(&env);

    let contract_name = String::from_str(&env, "XLM/USDC");
    let mut xlm_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "XLM"),
    );
    let usdc_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "USD Coin"),
        &String::from_str(&env, "USDC"),
    );
    let mut pho_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "Phoenix"),
        &String::from_str(&env, "PHO"),
    );

    if xlm_token.address >= pho_token.address {
        std::mem::swap(&mut pho_token, &mut xlm_token);
    }

    xlm_token.mint(&admin, &1_000_000);
    pho_token.mint(&admin, &2_000_000);

    let trader_client = deploy_trader_client(&env);

    xlm_token.mint(&trader_client.address, &1_000);

    let xlm_pho_client: crate::lp_contract::Client<'_> = deploy_and_init_lp_client(
        &env,
        admin.clone(),
        xlm_token.address.clone(),
        1_000_000,
        pho_token.address.clone(),
        1_000_000,
    );

    trader_client.initialize(
        &admin,
        &contract_name,
        &(xlm_token.address.clone(), usdc_token.address.clone()),
        &pho_token.address,
    );

    assert_eq!(
        trader_client.query_balances(),
        BalanceInfo {
            output_token: Asset {
                symbol: String::from_str(&env, "XLM"),
                amount: 0
            },
            token_a: Asset {
                symbol: String::from_str(&env, "PHO"),
                amount: 1_000
            },
            token_b: Asset {
                symbol: String::from_str(&env, "USDC"),
                amount: 0
            }
        }
    );

    trader_client.trade_token(
        &admin.clone(),
        &xlm_token.address.clone(),
        &xlm_pho_client.address,
        &Some(1_000),
        &None::<u64>,
    );

    assert_eq!(
        trader_client.query_balances(),
        BalanceInfo {
            output_token: Asset {
                symbol: String::from_str(&env, "XLM"),
                amount: 1_000
            },
            token_a: Asset {
                symbol: String::from_str(&env, "PHO"),
                amount: 0
            },
            token_b: Asset {
                symbol: String::from_str(&env, "USDC"),
                amount: 0
            }
        }
    );

    assert_eq!(pho_token.balance(&rcpt), 0);
    trader_client.transfer(&admin, &rcpt, &1_000, &None);
    assert_eq!(pho_token.balance(&rcpt), 1_000);
}

#[test]
#[should_panic(expected = "Trader: Trade_token: Unauthorized trade")]
fn trade_token_should_fail_when_unauthorized() {
    let env = Env::default();

    env.mock_all_auths_allowing_non_root_auth();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);

    let contract_name = String::from_str(&env, "XLM/USDC");
    let mut xlm_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "XLM"),
    );

    let mut pho_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "Phoenix"),
        &String::from_str(&env, "PHO"),
    );

    if xlm_token.address >= pho_token.address {
        std::mem::swap(&mut pho_token, &mut xlm_token);
    }

    xlm_token.mint(&admin, &1_000_000);
    pho_token.mint(&admin, &2_000_000);

    let trader_client = deploy_trader_client(&env);

    xlm_token.mint(&trader_client.address, &1_000);

    let xlm_pho_client: crate::lp_contract::Client<'_> = deploy_and_init_lp_client(
        &env,
        admin.clone(),
        xlm_token.address.clone(),
        1_000_000,
        pho_token.address.clone(),
        1_000_000,
    );

    trader_client.initialize(
        &admin,
        &contract_name,
        &(xlm_token.address.clone(), Address::generate(&env)),
        &pho_token.address,
    );

    trader_client.trade_token(
        &Address::generate(&env),
        &xlm_token.address.clone(),
        &xlm_pho_client.address,
        &Some(1_000),
        &None::<u64>,
    );
}

#[test]
#[should_panic(expected = "Trader: Trade_token: Token to swap is not part of the trading pair")]
fn trade_token_should_fail_when_offered_token_not_in_pair() {
    let env = Env::default();

    env.mock_all_auths_allowing_non_root_auth();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);

    let contract_name = String::from_str(&env, "XLM/USDC");
    let mut xlm_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "XLM"),
    );

    let mut pho_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "Phoenix"),
        &String::from_str(&env, "PHO"),
    );

    if xlm_token.address >= pho_token.address {
        std::mem::swap(&mut pho_token, &mut xlm_token);
    }

    xlm_token.mint(&admin, &1_000_000);
    pho_token.mint(&admin, &2_000_000);

    let trader_client = deploy_trader_client(&env);

    xlm_token.mint(&trader_client.address, &1_000);

    let xlm_pho_client: crate::lp_contract::Client<'_> = deploy_and_init_lp_client(
        &env,
        admin.clone(),
        xlm_token.address.clone(),
        1_000_000,
        pho_token.address.clone(),
        1_000_000,
    );

    trader_client.initialize(
        &admin,
        &contract_name,
        &(xlm_token.address.clone(), Address::generate(&env)),
        &pho_token.address,
    );

    trader_client.trade_token(
        &admin.clone(),
        &Address::generate(&env),
        &xlm_pho_client.address,
        &Some(1_000),
        &None::<u64>,
    );
}

#[test]
#[should_panic(expected = "Trader: Transfer: Unauthorized transfer")]
fn transfer_should_fail_when_unauthorized() {
    let env = Env::default();

    env.mock_all_auths_allowing_non_root_auth();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let rcpt = Address::generate(&env);

    let contract_name = String::from_str(&env, "XLM/USDC");
    let mut xlm_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "Stellar"),
        &String::from_str(&env, "XLM"),
    );
    let usdc_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "USD Coin"),
        &String::from_str(&env, "USDC"),
    );
    let mut pho_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "Phoenix"),
        &String::from_str(&env, "PHO"),
    );

    if xlm_token.address >= pho_token.address {
        std::mem::swap(&mut pho_token, &mut xlm_token);
    }

    xlm_token.mint(&admin, &1_000_000);
    pho_token.mint(&admin, &2_000_000);

    let trader_client = deploy_trader_client(&env);

    xlm_token.mint(&trader_client.address, &1_000);

    let xlm_pho_client: crate::lp_contract::Client<'_> = deploy_and_init_lp_client(
        &env,
        admin.clone(),
        xlm_token.address.clone(),
        1_000_000,
        pho_token.address.clone(),
        1_000_000,
    );

    trader_client.initialize(
        &admin,
        &contract_name,
        &(xlm_token.address.clone(), usdc_token.address.clone()),
        &pho_token.address,
    );

    assert_eq!(
        trader_client.query_balances(),
        BalanceInfo {
            output_token: Asset {
                symbol: String::from_str(&env, "XLM"),
                amount: 0
            },
            token_a: Asset {
                symbol: String::from_str(&env, "PHO"),
                amount: 1_000
            },
            token_b: Asset {
                symbol: String::from_str(&env, "USDC"),
                amount: 0
            }
        }
    );

    trader_client.trade_token(
        &admin.clone(),
        &xlm_token.address.clone(),
        &xlm_pho_client.address,
        &Some(1_000),
        &None::<u64>,
    );

    assert_eq!(
        trader_client.query_balances(),
        BalanceInfo {
            output_token: Asset {
                symbol: String::from_str(&env, "XLM"),
                amount: 1_000
            },
            token_a: Asset {
                symbol: String::from_str(&env, "PHO"),
                amount: 0
            },
            token_b: Asset {
                symbol: String::from_str(&env, "USDC"),
                amount: 0
            }
        }
    );

    assert_eq!(pho_token.balance(&rcpt), 0);
    trader_client.transfer(&Address::generate(&env), &rcpt, &1_000, &None);
    assert_eq!(pho_token.balance(&rcpt), 1_000);
}
