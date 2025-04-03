use soroban_sdk::{
    testutils::{arbitrary::std, Address as _},
    Address, Env, String,
};

use crate::{
    contract::{Trader, TraderClient},
    error::ContractError,
    storage::{Asset, BalanceInfo, DataKey, OutputTokenInfo, ADMIN},
    tests::setup::deploy_token_contract,
};
use test_case::test_case;

use super::setup::deploy_and_init_lp_client;

#[test]
fn initialize() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

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

    let output_token_name = String::from_str(&env, "Phoenix");
    let output_token_sym = String::from_str(&env, "PHO");

    let pho_token = deploy_token_contract(&env, &admin, &6, &output_token_name, &output_token_sym);

    let trader_client = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                contract_name.clone(),
                &(xlm_token.address.clone(), usdc_token.address.clone()),
                &pho_token.address,
            ),
        ),
    );

    assert_eq!(trader_client.query_admin_address(), admin);
    assert_eq!(trader_client.query_contract_name(), contract_name);
    assert_eq!(
        trader_client.query_trading_pairs(),
        (xlm_token.address, usdc_token.address)
    );

    assert_eq!(
        trader_client.query_output_token_info(),
        OutputTokenInfo {
            address: pho_token.address,
            name: output_token_name,
            symbol: output_token_sym,
            decimal: 6
        }
    )
}

#[test]
fn simple_trade_token_and_transfer_token() {
    let env = Env::default();

    env.mock_all_auths_allowing_non_root_auth();
    env.cost_estimate().budget().reset_unlimited();

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
    let mut output_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "Phoenix"),
        &String::from_str(&env, "PHO"),
    );

    if xlm_token.address >= output_token.address {
        std::mem::swap(&mut output_token, &mut xlm_token);
    }

    xlm_token.mint(&admin, &1_000_000);
    output_token.mint(&admin, &2_000_000);

    let xlm_pho_client: crate::lp_contract::Client<'_> = deploy_and_init_lp_client(
        &env,
        admin.clone(),
        xlm_token.address.clone(),
        1_000_000,
        output_token.address.clone(),
        1_000_000,
        0,
    );

    let trader_client = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                contract_name.clone(),
                &(xlm_token.address.clone(), usdc_token.address.clone()),
                &output_token.address,
            ),
        ),
    );

    xlm_token.mint(&trader_client.address, &1_000);

    assert_eq!(
        trader_client.query_balances(),
        BalanceInfo {
            output_token: Asset {
                symbol: output_token.symbol(),
                amount: 0
            },
            token_a: Asset {
                symbol: xlm_token.symbol(),
                amount: 1_000
            },
            token_b: Asset {
                symbol: usdc_token.symbol(),
                amount: 0
            }
        }
    );

    trader_client.trade_token(
        &admin.clone(),
        &xlm_token.address.clone(),
        &xlm_pho_client.address,
        &Some(1_000),
        &None::<i64>,
        &None,
        &None,
        &None,
    );

    assert_eq!(
        trader_client.query_balances(),
        BalanceInfo {
            output_token: Asset {
                symbol: output_token.symbol(),
                amount: 1_000
            },
            token_a: Asset {
                symbol: xlm_token.symbol(),
                amount: 0
            },
            token_b: Asset {
                symbol: usdc_token.symbol(),
                amount: 0
            }
        }
    );

    assert_eq!(output_token.balance(&rcpt), 0);
    trader_client.transfer(&admin, &rcpt, &1_000, &None);
    assert_eq!(output_token.balance(&rcpt), 1_000);
}

#[test]
fn extended_trade_and_transfer_token() {
    let env = Env::default();

    env.mock_all_auths_allowing_non_root_auth();
    env.cost_estimate().budget().reset_unlimited();

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
    let mut usdc_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "USD Coin"),
        &String::from_str(&env, "USDC"),
    );
    let mut output_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "Phoenix"),
        &String::from_str(&env, "PHO"),
    );

    if xlm_token.address >= output_token.address {
        std::mem::swap(&mut output_token, &mut xlm_token);
    }

    if usdc_token.address >= output_token.address {
        std::mem::swap(&mut output_token, &mut usdc_token);
    }

    xlm_token.mint(&admin, &1_000_000);
    usdc_token.mint(&admin, &3_000_000);
    output_token.mint(&admin, &2_000_000);

    // 1:1 xlm/pho pool
    let xlm_pho_client: crate::lp_contract::Client<'_> = deploy_and_init_lp_client(
        &env,
        admin.clone(),
        xlm_token.address.clone(),
        1_000_000,
        output_token.address.clone(),
        1_000_000,
        500, // 5% swap fee
    );

    // 3:1 usdc/pho pool
    let usdc_pho_client: crate::lp_contract::Client<'_> = deploy_and_init_lp_client(
        &env,
        admin.clone(),
        usdc_token.address.clone(),
        3_000_000,
        output_token.address.clone(),
        1_000_000,
        1_000, // 10% swap fee
    );

    let trader_client = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                contract_name.clone(),
                &(xlm_token.address.clone(), usdc_token.address.clone()),
                &output_token.address,
            ),
        ),
    );

    // collected fees from previous txs so we have something to trade against PHO token
    xlm_token.mint(&trader_client.address, &2_000);
    usdc_token.mint(&trader_client.address, &3_000);

    assert_eq!(
        trader_client.query_balances(),
        BalanceInfo {
            output_token: Asset {
                symbol: output_token.symbol(),
                amount: 0
            },
            token_a: Asset {
                symbol: xlm_token.symbol(),
                amount: 2_000
            },
            token_b: Asset {
                symbol: usdc_token.symbol(),
                amount: 3_000
            }
        }
    );

    // admin trades 1/2 of their XLM for PHO
    // there is %5 fee
    // so user will receive ~950 PHO
    trader_client.trade_token(
        &admin.clone(),
        &xlm_token.address.clone(),
        &xlm_pho_client.address,
        &Some(1_000),
        &None::<i64>,
        &None,
        &None,
        &None,
    );

    assert_eq!(
        trader_client.query_balances(),
        BalanceInfo {
            output_token: Asset {
                symbol: output_token.symbol(),
                amount: 950
            },
            token_a: Asset {
                symbol: xlm_token.symbol(),
                amount: 1_000
            },
            token_b: Asset {
                symbol: usdc_token.symbol(),
                amount: 3_000
            }
        }
    );

    // admin trades the rest of their XLM for PHO
    // there is %5 fee
    // so user will receive ~950 PHO
    trader_client.trade_token(
        &admin.clone(),
        &xlm_token.address.clone(),
        &xlm_pho_client.address,
        &Some(1_000),
        &None::<i64>,
        &None,
        &None,
        &None,
    );

    assert_eq!(
        trader_client.query_balances(),
        BalanceInfo {
            output_token: Asset {
                symbol: output_token.symbol(),
                amount: 1_899
            },
            token_a: Asset {
                symbol: xlm_token.symbol(),
                amount: 0
            },
            token_b: Asset {
                symbol: usdc_token.symbol(),
                amount: 3_000
            }
        }
    );

    // admin trades 1/2 of their USDC for PHO
    // this time the fee is %10
    // pool is with 3:1 ratio
    // we will receive ~450 PHO
    trader_client.trade_token(
        &admin.clone(),
        &usdc_token.address.clone(),
        &usdc_pho_client.address,
        &Some(1_500),
        &None::<i64>,
        &None,
        &None,
        &None,
    );

    // 1899 + 450 = 2_349
    assert_eq!(
        trader_client.query_balances(),
        BalanceInfo {
            output_token: Asset {
                symbol: output_token.symbol(),
                amount: 2_349
            },
            token_a: Asset {
                symbol: xlm_token.symbol(),
                amount: 0
            },
            token_b: Asset {
                symbol: usdc_token.symbol(),
                amount: 1_500
            }
        }
    );

    // admin trades what's left of their USDC for PHO
    // pool with 3:1 ratio and %10 fee
    // we will receive ~450 PHO once again
    trader_client.trade_token(
        &admin.clone(),
        &usdc_token.address.clone(),
        &usdc_pho_client.address,
        &Some(1_500),
        &None::<i64>,
        &None,
        &None,
        &None,
    );

    // 2_349 + 450 = 2_799
    assert_eq!(
        trader_client.query_balances(),
        BalanceInfo {
            output_token: Asset {
                symbol: output_token.symbol(),
                amount: 2_799
            },
            token_a: Asset {
                symbol: xlm_token.symbol(),
                amount: 0
            },
            token_b: Asset {
                symbol: usdc_token.symbol(),
                amount: 0
            }
        }
    );

    // finally we will check the balance of the rcpt before and after we transfer
    assert_eq!(output_token.balance(&rcpt), 0);
    trader_client.transfer(&admin, &rcpt, &1_000, &None);
    assert_eq!(output_token.balance(&rcpt), 1_000);
}

#[test]
#[should_panic(expected = "Trader: Trade_token: Unauthorized trade")]
fn trade_token_should_fail_when_unauthorized() {
    let env = Env::default();

    env.mock_all_auths_allowing_non_root_auth();
    env.cost_estimate().budget().reset_unlimited();

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

    let xlm_pho_client: crate::lp_contract::Client<'_> = deploy_and_init_lp_client(
        &env,
        admin.clone(),
        xlm_token.address.clone(),
        1_000_000,
        pho_token.address.clone(),
        1_000_000,
        0,
    );

    let trader_client = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                contract_name.clone(),
                &(xlm_token.address.clone(), Address::generate(&env)),
                &pho_token.address,
            ),
        ),
    );

    xlm_token.mint(&trader_client.address, &1_000);

    trader_client.trade_token(
        &Address::generate(&env),
        &xlm_token.address.clone(),
        &xlm_pho_client.address,
        &Some(1_000),
        &None::<i64>,
        &None,
        &None,
        &None,
    );
}

#[test]
#[should_panic(expected = "Trader: Trade_token: Token to swap is not part of the trading pair")]
fn trade_token_should_fail_when_offered_token_not_in_pair() {
    let env = Env::default();

    env.mock_all_auths_allowing_non_root_auth();
    env.cost_estimate().budget().reset_unlimited();

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

    let xlm_pho_client: crate::lp_contract::Client<'_> = deploy_and_init_lp_client(
        &env,
        admin.clone(),
        xlm_token.address.clone(),
        1_000_000,
        pho_token.address.clone(),
        1_000_000,
        0,
    );

    let trader_client = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                contract_name.clone(),
                &(xlm_token.address.clone(), Address::generate(&env)),
                &pho_token.address,
            ),
        ),
    );

    xlm_token.mint(&trader_client.address, &1_000);

    trader_client.trade_token(
        &admin.clone(),
        &Address::generate(&env),
        &xlm_pho_client.address,
        &Some(1_000),
        &None::<i64>,
        &None,
        &None,
        &None,
    );
}

#[test]
#[should_panic(expected = "Trader: Transfer: Unauthorized transfer")]
fn transfer_should_fail_when_unauthorized() {
    let env = Env::default();

    env.mock_all_auths_allowing_non_root_auth();
    env.cost_estimate().budget().reset_unlimited();

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

    let xlm_pho_client: crate::lp_contract::Client<'_> = deploy_and_init_lp_client(
        &env,
        admin.clone(),
        xlm_token.address.clone(),
        1_000_000,
        pho_token.address.clone(),
        1_000_000,
        0,
    );

    let trader_client = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                contract_name.clone(),
                &(xlm_token.address.clone(), Address::generate(&env)),
                &pho_token.address,
            ),
        ),
    );

    xlm_token.mint(&trader_client.address, &1_000);

    trader_client.trade_token(
        &admin.clone(),
        &xlm_token.address.clone(),
        &xlm_pho_client.address,
        &Some(1_000),
        &None::<i64>,
        &None,
        &None,
        &None,
    );

    trader_client.transfer(&Address::generate(&env), &rcpt, &1_000, &None);
}

#[test_case(-1 ; "when negative")]
#[test_case(10001; "when bigger than 100 percent")]
#[should_panic(expected = "Error(Contract, #608)")]
fn transfer_should_fail_with_invalid_spread_bps(max_spread_bps: i64) {
    let env = Env::default();

    env.mock_all_auths_allowing_non_root_auth();
    env.cost_estimate().budget().reset_unlimited();

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

    let xlm_pho_client: crate::lp_contract::Client<'_> = deploy_and_init_lp_client(
        &env,
        admin.clone(),
        xlm_token.address.clone(),
        1_000_000,
        pho_token.address.clone(),
        1_000_000,
        0,
    );

    let trader_client = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                contract_name.clone(),
                &(xlm_token.address.clone(), Address::generate(&env)),
                &pho_token.address,
            ),
        ),
    );

    xlm_token.mint(&trader_client.address, &1_000);

    trader_client.trade_token(
        &admin.clone(),
        &xlm_token.address.clone(),
        &xlm_pho_client.address,
        &Some(1_000),
        &Some(max_spread_bps),
        &None,
        &None,
        &None,
    );
}

#[test]
fn simple_trade_token_and_transfer_token_with_some_ask_asset_min_amount() {
    let env = Env::default();

    env.mock_all_auths_allowing_non_root_auth();
    env.cost_estimate().budget().reset_unlimited();

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
    let mut output_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "Phoenix"),
        &String::from_str(&env, "PHO"),
    );

    if xlm_token.address >= output_token.address {
        std::mem::swap(&mut output_token, &mut xlm_token);
    }

    xlm_token.mint(&admin, &1_000_000);
    output_token.mint(&admin, &2_000_000);

    let xlm_pho_client: crate::lp_contract::Client<'_> = deploy_and_init_lp_client(
        &env,
        admin.clone(),
        xlm_token.address.clone(),
        1_000_000,
        output_token.address.clone(),
        1_000_000,
        0,
    );

    let trader_client = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                contract_name.clone(),
                &(xlm_token.address.clone(), usdc_token.address.clone()),
                &output_token.address,
            ),
        ),
    );

    xlm_token.mint(&trader_client.address, &1_000);

    // pretty much the same test as `simple_trade_token_and_transfer` but with `Some` value for
    // `ask_asset_min_amount`
    trader_client.trade_token(
        &admin.clone(),
        &xlm_token.address.clone(),
        &xlm_pho_client.address,
        &Some(1_000),
        &None::<i64>,
        &None,
        &Some(1_000),
        &None,
    );

    assert_eq!(
        trader_client.query_balances(),
        BalanceInfo {
            output_token: Asset {
                symbol: output_token.symbol(),
                amount: 1_000
            },
            token_a: Asset {
                symbol: xlm_token.symbol(),
                amount: 0
            },
            token_b: Asset {
                symbol: usdc_token.symbol(),
                amount: 0
            }
        }
    );

    assert_eq!(output_token.balance(&rcpt), 0);
    trader_client.transfer(&admin, &rcpt, &1_000, &None);
    assert_eq!(output_token.balance(&rcpt), 1_000);
}

#[test]
fn update_contract_metadata() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

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

    soroban_sdk::testutils::arbitrary::std::dbg!();
    let new_pair_a = Address::generate(&env);
    let new_pair_b = Address::generate(&env);

    assert_eq!(
        trader_client.try_update_pair_addresses(&(new_pair_a.clone(), pho_token.address)),
        Err(Ok(ContractError::OutputTokenInPair))
    );

    trader_client.update_pair_addresses(&(new_pair_a.clone(), new_pair_b.clone()));

    assert_eq!(
        trader_client.query_trading_pairs(),
        (new_pair_a, new_pair_b)
    );

    let new_output_token = deploy_token_contract(
        &env,
        &admin,
        &6,
        &String::from_str(&env, "Phoenix2"),
        &String::from_str(&env, "MOREPHO"),
    );

    trader_client.update_output_token(&new_output_token.address);

    assert_eq!(
        trader_client.query_output_token_info().address,
        new_output_token.address
    );

    soroban_sdk::testutils::arbitrary::std::dbg!();
    let new_trader_name = String::from_str(&env, "Some new name");
    trader_client.update_contract_name(&new_trader_name);

    assert_eq!(trader_client.query_contract_name(), new_trader_name);
}

#[test]
fn test_query_version() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_name = String::from_str(&env, "XLM/USDC");
    let pair_a = Address::generate(&env);
    let pair_b = Address::generate(&env);

    let output_addr = Address::generate(&env);

    let trader_client = deploy_trader_client(&env);
    trader_client.initialize(&admin, &contract_name, &(pair_a, pair_b), &output_addr);

    let expected_version = env!("CARGO_PKG_VERSION");
    let version = trader_client.query_version();
    assert_eq!(String::from_str(&env, expected_version), version);
}

#[test]
fn migrate_admin_key() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_name = String::from_str(&env, "XLM/USDC");
    let pair_a = Address::generate(&env);
    let pair_b = Address::generate(&env);

    let output_addr = Address::generate(&env);

    let trader_client = deploy_trader_client(&env);
    trader_client.initialize(&admin, &contract_name, &(pair_a, pair_b), &output_addr);

    let before_migration: Address = env.as_contract(&trader_client.address, || {
        env.storage().persistent().get(&DataKey::Admin).unwrap()
    });

    trader_client.migrate_admin_key();

    let after_migration: Address = env.as_contract(&trader_client.address, || {
        env.storage().instance().get(&ADMIN).unwrap()
    });

    assert_eq!(before_migration, after_migration);
    assert_ne!(Address::generate(&env), after_migration)
}
