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
fn simple_trade_token_and_transfer_token() {
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

    let trader_client = deploy_trader_client(&env);

    let xlm_pho_client: crate::lp_contract::Client<'_> = deploy_and_init_lp_client(
        &env,
        admin.clone(),
        xlm_token.address.clone(),
        1_000_000,
        output_token.address.clone(),
        1_000_000,
        0,
    );

    trader_client.initialize(
        &admin,
        &contract_name,
        &(xlm_token.address.clone(), usdc_token.address.clone()),
        &output_token.address,
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
        &None::<u64>,
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
fn extended_trade_token_and_transfer_token() {
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

    let trader_client = deploy_trader_client(&env);

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

    trader_client.initialize(
        &admin,
        &contract_name,
        &(xlm_token.address.clone(), usdc_token.address.clone()),
        &output_token.address,
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
        &None::<u64>,
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
        &None::<u64>,
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
        &None::<u64>,
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
        &None::<u64>,
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
        0,
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
        0,
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
        0,
    );

    trader_client.initialize(
        &admin,
        &contract_name,
        &(xlm_token.address.clone(), Address::generate(&env)),
        &pho_token.address,
    );

    trader_client.trade_token(
        &admin.clone(),
        &xlm_token.address.clone(),
        &xlm_pho_client.address,
        &Some(1_000),
        &None::<u64>,
    );

    trader_client.transfer(&Address::generate(&env), &rcpt, &1_000, &None);
}
