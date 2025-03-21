use super::setup::{deploy_factory_contract, generate_lp_init_info};
use crate::storage::{Asset, DataKey, LpPortfolio, Stake, StakePortfolio, UserPortfolio};
use crate::tests::setup::{
    install_and_deploy_token_contract, lp_contract, stake_contract, ONE_DAY,
};
use crate::token_contract;
use phoenix::ttl::{
    DAY_IN_LEDGERS, INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL, PERSISTENT_TARGET_TTL,
};
use phoenix::utils::{LiquidityPoolInitInfo, PoolType, StakeInitInfo, TokenInitInfo};
use soroban_sdk::testutils::storage::{Instance, Persistent};
use soroban_sdk::testutils::Ledger;
use soroban_sdk::vec;
use soroban_sdk::{
    contracttype,
    testutils::{arbitrary::std, Address as _},
    Address, Env, String, Symbol, Vec,
};

#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PairType {
    Xyk = 0,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquidityPoolConfig {
    pub token_a: Address,
    pub token_b: Address,
    pub share_token: Address,
    pub stake_contract: Address,
    pub pool_type: PairType,
    /// The total fees (in bps) charged by a pool of this type.
    /// In relation to the returned amount of tokens
    pub total_fee_bps: i64,
    pub fee_recipient: Address,
    /// The maximum amount of slippage (in bps) that is tolerated during providing liquidity
    pub max_allowed_slippage_bps: i64,
    /// The maximum amount of spread (in bps) that is tolerated during swap
    pub max_allowed_spread_bps: i64,
    pub max_referral_bps: i64,
}

#[test]
fn test_deploy_multiple_liquidity_pools() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let mut token1 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        9,
        String::from_str(&env, "Phoenix"),
        String::from_str(&env, "PHO"),
    );
    let mut token2 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        14,
        String::from_str(&env, "Stellar"),
        String::from_str(&env, "XLM"),
    );
    let mut token3 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        6,
        String::from_str(&env, "Polkadot"),
        String::from_str(&env, "DOT"),
    );
    let mut token4 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        14,
        String::from_str(&env, "Cosmos"),
        String::from_str(&env, "ATOM"),
    );
    let mut token5 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        4,
        String::from_str(&env, "Osmosis"),
        String::from_str(&env, "OSMO"),
    );
    let mut token6 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        9,
        String::from_str(&env, "Dog wiff hat"),
        String::from_str(&env, "WIFF"),
    );

    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    if token4.address < token3.address {
        std::mem::swap(&mut token3, &mut token4);
    }

    if token6.address < token5.address {
        std::mem::swap(&mut token5, &mut token6);
    }

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    let first_token_init_info = TokenInitInfo {
        token_a: token1.address.clone(),
        token_b: token2.address.clone(),
    };
    let first_stake_init_info = StakeInitInfo {
        min_bond: 10i128,
        min_reward: 5i128,
        manager: Address::generate(&env),
        max_complexity: 10u32,
    };

    let second_token_init_info = TokenInitInfo {
        token_a: token3.address.clone(),
        token_b: token4.address.clone(),
    };
    let second_stake_init_info = StakeInitInfo {
        min_bond: 5i128,
        min_reward: 2i128,
        manager: Address::generate(&env),
        max_complexity: 10u32,
    };

    let third_token_init_info = TokenInitInfo {
        token_a: token5.address.clone(),
        token_b: token6.address.clone(),
    };
    let third_stake_init_info = StakeInitInfo {
        min_bond: 6i128,
        min_reward: 3i128,
        manager: Address::generate(&env),
        max_complexity: 10u32,
    };

    let first_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        max_allowed_slippage_bps: 5_000,
        default_slippage_bps: 2_500,
        max_allowed_spread_bps: 500,
        swap_fee_bps: 0,
        max_referral_bps: 5_000,
        token_init_info: first_token_init_info.clone(),
        stake_init_info: first_stake_init_info,
    };

    let second_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        max_allowed_slippage_bps: 4_000,
        default_slippage_bps: 2_500,
        max_allowed_spread_bps: 400,
        swap_fee_bps: 0,
        max_referral_bps: 5_000,
        token_init_info: second_token_init_info,
        stake_init_info: second_stake_init_info,
    };

    let third_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        max_allowed_slippage_bps: 4_000,
        default_slippage_bps: 2_500,
        max_allowed_spread_bps: 400,
        swap_fee_bps: 0,
        max_referral_bps: 5_000,
        token_init_info: third_token_init_info,
        stake_init_info: third_stake_init_info,
    };

    let lp_contract_addr = factory.create_liquidity_pool(
        &admin.clone(),
        &first_lp_init_info,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "PHO/BTC"),
        &PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );
    let second_lp_contract_addr = factory.create_liquidity_pool(
        &admin.clone(),
        &second_lp_init_info,
        &String::from_str(&env, "Pool #2"),
        &String::from_str(&env, "PHO/ETH"),
        &PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );
    let third_lp_contract_addr = factory.create_liquidity_pool(
        &admin.clone(),
        &third_lp_init_info,
        &String::from_str(&env, "Pool #3"),
        &String::from_str(&env, "PHO/XLM"),
        &PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );

    let first_result = factory.query_pool_details(&lp_contract_addr);
    let share_token_addr: Address = env.invoke_contract(
        &lp_contract_addr,
        &Symbol::new(&env, "query_share_token_address"),
        Vec::new(&env),
    );
    let first_lp_config: LiquidityPoolConfig = env.invoke_contract(
        &lp_contract_addr,
        &Symbol::new(&env, "query_config"),
        Vec::new(&env),
    );

    assert_eq!(
        first_lp_init_info.max_allowed_spread_bps,
        first_lp_config.max_allowed_spread_bps
    );

    assert_eq!(token1.address, first_result.pool_response.asset_a.address);
    assert_eq!(token2.address, first_result.pool_response.asset_b.address);
    assert_eq!(
        share_token_addr,
        first_result.pool_response.asset_lp_share.address
    );
    assert_eq!(lp_contract_addr, first_result.pool_address);

    let second_result = factory.query_pool_details(&second_lp_contract_addr);
    let second_share_token_addr: Address = env.invoke_contract(
        &second_lp_contract_addr,
        &Symbol::new(&env, "query_share_token_address"),
        Vec::new(&env),
    );
    let second_lp_config: LiquidityPoolConfig = env.invoke_contract(
        &second_lp_contract_addr,
        &Symbol::new(&env, "query_config"),
        Vec::new(&env),
    );

    assert_eq!(
        second_lp_init_info.max_allowed_spread_bps,
        second_lp_config.max_allowed_spread_bps
    );

    assert_eq!(token3.address, second_result.pool_response.asset_a.address);
    assert_eq!(token4.address, second_result.pool_response.asset_b.address);
    assert_eq!(
        second_share_token_addr,
        second_result.pool_response.asset_lp_share.address
    );
    assert_eq!(second_lp_contract_addr, second_result.pool_address);

    let third_result = factory.query_pool_details(&third_lp_contract_addr);
    let third_share_token_addr: Address = env.invoke_contract(
        &third_lp_contract_addr,
        &Symbol::new(&env, "query_share_token_address"),
        Vec::new(&env),
    );
    let third_lp_config: LiquidityPoolConfig = env.invoke_contract(
        &third_lp_contract_addr,
        &Symbol::new(&env, "query_config"),
        Vec::new(&env),
    );

    assert_eq!(
        third_lp_init_info.max_allowed_spread_bps,
        third_lp_config.max_allowed_spread_bps
    );

    assert_eq!(token5.address, third_result.pool_response.asset_a.address);
    assert_eq!(token6.address, third_result.pool_response.asset_b.address);
    assert_eq!(
        third_share_token_addr,
        third_result.pool_response.asset_lp_share.address
    );
    assert_eq!(third_lp_contract_addr, third_result.pool_address);

    let all_pools = factory.query_all_pools_details();
    assert_eq!(all_pools.len(), 3);
    all_pools.iter().for_each(|pool| {
        assert!(all_pools.contains(pool));
    });

    let first_lp_address_by_tuple =
        factory.query_for_pool_by_token_pair(&token1.address, &token2.address);
    assert_eq!(first_lp_address_by_tuple, lp_contract_addr);
}

#[test]
fn test_queries_by_tuple() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let mut token1 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        9,
        String::from_str(&env, "Phoenix"),
        String::from_str(&env, "PHO"),
    );
    let mut token2 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        14,
        String::from_str(&env, "Stellar"),
        String::from_str(&env, "XLM"),
    );
    let mut token3 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        6,
        String::from_str(&env, "Polkadot"),
        String::from_str(&env, "DOT"),
    );
    let mut token4 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        14,
        String::from_str(&env, "Cosmos"),
        String::from_str(&env, "ATOM"),
    );
    let mut token5 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        4,
        String::from_str(&env, "Osmosis"),
        String::from_str(&env, "OSMO"),
    );
    let mut token6 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        9,
        String::from_str(&env, "Dog wiff hat"),
        String::from_str(&env, "WIFF"),
    );

    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    if token4.address < token3.address {
        std::mem::swap(&mut token3, &mut token4);
    }

    if token6.address < token5.address {
        std::mem::swap(&mut token5, &mut token6);
    }

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    let first_token_init_info = TokenInitInfo {
        token_a: token1.address.clone(),
        token_b: token2.address.clone(),
    };
    let first_stake_init_info = StakeInitInfo {
        min_bond: 10i128,
        min_reward: 5i128,
        manager: Address::generate(&env),
        max_complexity: 10u32,
    };

    let second_token_init_info = TokenInitInfo {
        token_a: token3.address.clone(),
        token_b: token4.address.clone(),
    };
    let second_stake_init_info = StakeInitInfo {
        min_bond: 5i128,
        min_reward: 2i128,
        manager: Address::generate(&env),
        max_complexity: 10u32,
    };

    let third_token_init_info = TokenInitInfo {
        token_a: token5.address.clone(),
        token_b: token6.address.clone(),
    };
    let third_stake_init_info = StakeInitInfo {
        min_bond: 6i128,
        min_reward: 3i128,
        manager: Address::generate(&env),
        max_complexity: 10u32,
    };

    let first_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        max_allowed_slippage_bps: 5_000,
        default_slippage_bps: 2_500,
        max_allowed_spread_bps: 500,
        swap_fee_bps: 0,
        max_referral_bps: 5_000,
        token_init_info: first_token_init_info.clone(),
        stake_init_info: first_stake_init_info,
    };

    let second_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        max_allowed_slippage_bps: 4_000,
        default_slippage_bps: 2_500,
        max_allowed_spread_bps: 400,
        swap_fee_bps: 0,
        max_referral_bps: 5_000,
        token_init_info: second_token_init_info,
        stake_init_info: second_stake_init_info,
    };

    let third_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        max_allowed_slippage_bps: 4_000,
        default_slippage_bps: 2_500,
        max_allowed_spread_bps: 400,
        swap_fee_bps: 0,
        max_referral_bps: 5_000,
        token_init_info: third_token_init_info,
        stake_init_info: third_stake_init_info,
    };

    let lp_contract_addr = factory.create_liquidity_pool(
        &admin.clone(),
        &first_lp_init_info,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "PHO/BTC"),
        &PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );
    let second_lp_contract_addr = factory.create_liquidity_pool(
        &admin.clone(),
        &second_lp_init_info,
        &String::from_str(&env, "Pool #2"),
        &String::from_str(&env, "PHO/ETH"),
        &PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );
    let third_lp_contract_addr = factory.create_liquidity_pool(
        &admin.clone(),
        &third_lp_init_info,
        &String::from_str(&env, "Pool #3"),
        &String::from_str(&env, "PHO/XLM"),
        &PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );

    let first_result = factory.query_pool_details(&lp_contract_addr);

    assert_eq!(token1.address, first_result.pool_response.asset_a.address);
    assert_eq!(token2.address, first_result.pool_response.asset_b.address);
    assert_eq!(lp_contract_addr, first_result.pool_address);

    let second_result = factory.query_pool_details(&second_lp_contract_addr);
    let second_share_token_addr: Address = env.invoke_contract(
        &second_lp_contract_addr,
        &Symbol::new(&env, "query_share_token_address"),
        Vec::new(&env),
    );

    let second_lp_config: LiquidityPoolConfig = env.invoke_contract(
        &second_lp_contract_addr,
        &Symbol::new(&env, "query_config"),
        Vec::new(&env),
    );

    assert_eq!(
        second_lp_init_info.max_allowed_spread_bps,
        second_lp_config.max_allowed_spread_bps
    );

    assert_eq!(token3.address, second_result.pool_response.asset_a.address);
    assert_eq!(token4.address, second_result.pool_response.asset_b.address);
    assert_eq!(
        second_share_token_addr,
        second_result.pool_response.asset_lp_share.address
    );
    assert_eq!(second_lp_contract_addr, second_result.pool_address);

    let first_lp_address_by_tuple =
        factory.query_for_pool_by_token_pair(&token2.address, &token1.address);
    let second_lp_address_by_tuple =
        factory.query_for_pool_by_token_pair(&token3.address, &token4.address);
    let third_lp_address_by_tuple =
        factory.query_for_pool_by_token_pair(&token5.address, &token6.address);

    assert_eq!(first_lp_address_by_tuple, lp_contract_addr);
    assert_eq!(second_lp_address_by_tuple, second_lp_contract_addr);
    assert_eq!(third_lp_address_by_tuple, third_lp_contract_addr);
}

#[test]
#[should_panic(expected = "Factory: query_for_pool_by_token_pair failed: No liquidity pool found")]
fn test_queries_by_tuple_errors() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let admin = Address::generate(&env);
    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    factory.query_for_pool_by_token_pair(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_query_user_portfolio_with_stake() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let manager = Address::generate(&env);
    let user_1 = Address::generate(&env);
    let user_2 = Address::generate(&env);

    let mut token1 = token_contract::Client::new(
        &env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    );
    let mut token2 = token_contract::Client::new(
        &env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    );
    let mut token3 = token_contract::Client::new(
        &env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    );
    let mut token4 = token_contract::Client::new(
        &env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    );

    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    if token4.address < token3.address {
        std::mem::swap(&mut token3, &mut token4);
    }

    token1.mint(&user_1, &1_000_000i128);
    token2.mint(&user_1, &1_000_000i128);
    token3.mint(&user_2, &2_000_000i128);
    token4.mint(&user_2, &2_000_000i128);

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    let first_lp_init_info = generate_lp_init_info(
        token1.address.clone(),
        token2.address.clone(),
        manager.clone(),
        admin.clone(),
        fee_recipient.clone(),
    );

    let first_lp_contract_addr = factory.create_liquidity_pool(
        &admin.clone(),
        &first_lp_init_info,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "PHO/BTC"),
        &PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );

    let first_lp_client = lp_contract::Client::new(&env, &first_lp_contract_addr);

    let first_stake_address = factory
        .query_pool_details(&first_lp_contract_addr)
        .pool_response
        .stake_address;

    let first_stake_client = stake_contract::Client::new(&env, &first_stake_address);

    first_lp_client.provide_liquidity(
        &user_1.clone(),
        &Some(1_000_000),
        &Some(950_000i128),
        &Some(1_000_000),
        &Some(950_000i128),
        &None::<i64>,
        &None::<u64>,
        &false,
    );

    // first user portfolio after providing liquidity
    let first_portfolio = factory.query_user_portfolio(&user_1, &true);
    assert_eq!(
        first_portfolio,
        UserPortfolio {
            lp_portfolio: vec![
                &env,
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token1.address.clone(),
                            amount: 999_000i128,
                        },
                        Asset {
                            address: token2.address.clone(),
                            amount: 999_000i128
                        }
                    )
                }
            ],
            stake_portfolio: vec![&env,]
        }
    );

    env.ledger().with_mut(|li| li.timestamp = ONE_DAY);
    first_stake_client.bond(&user_1, &173i128);

    // first user portfolio after staking
    let first_portfolio = factory.query_user_portfolio(&user_1, &true);
    assert_eq!(
        first_portfolio,
        UserPortfolio {
            lp_portfolio: vec![
                &env,
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token1.address.clone(),
                            amount: 999_000i128,
                        },
                        Asset {
                            address: token2.address.clone(),
                            amount: 999_000i128
                        }
                    )
                }
            ],
            stake_portfolio: vec![
                &env,
                StakePortfolio {
                    staking_contract: first_stake_address.clone(),
                    stakes: vec![
                        &env,
                        Stake {
                            stake: 173i128,
                            stake_timestamp: ONE_DAY
                        }
                    ]
                }
            ]
        }
    );

    let second_lp_init_info = generate_lp_init_info(
        token3.address.clone(),
        token4.address.clone(),
        manager.clone(),
        admin.clone(),
        fee_recipient,
    );

    let second_lp_contract_addr = factory.create_liquidity_pool(
        &admin.clone(),
        &second_lp_init_info,
        &String::from_str(&env, "Second Pool"),
        &String::from_str(&env, "PHO/ETH"),
        &PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );

    let second_lp_client = lp_contract::Client::new(&env, &second_lp_contract_addr);
    let second_stake_address = factory
        .query_pool_details(&second_lp_contract_addr)
        .pool_response
        .stake_address;

    let second_stake_client = stake_contract::Client::new(&env, &second_stake_address);

    second_lp_client.provide_liquidity(
        &user_2.clone(),
        &Some(2_000_000),
        &Some(1_999_999i128),
        &Some(2_000_000),
        &Some(1_999_999i128),
        &None::<i64>,
        &None::<u64>,
        &false,
    );

    let second_portfolio = factory.query_user_portfolio(&user_2, &true);
    assert_eq!(
        second_portfolio,
        UserPortfolio {
            lp_portfolio: vec![
                &env,
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token3.address.clone(),
                            amount: 1_999_000i128,
                        },
                        Asset {
                            address: token4.address.clone(),
                            amount: 1_999_000i128
                        }
                    )
                }
            ],
            stake_portfolio: vec![&env,]
        }
    );

    second_stake_client.bond(&user_2, &223i128);

    let second_portfolio = factory.query_user_portfolio(&user_2, &true);
    assert_eq!(
        second_portfolio,
        UserPortfolio {
            lp_portfolio: vec![
                &env,
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token3.address.clone(),
                            amount: 1_999_000i128,
                        },
                        Asset {
                            address: token4.address.clone(),
                            amount: 1_999_000i128
                        }
                    )
                }
            ],
            stake_portfolio: vec![
                &env,
                StakePortfolio {
                    staking_contract: second_stake_address.clone(),
                    stakes: vec![
                        &env,
                        Stake {
                            stake: 223i128,
                            stake_timestamp: ONE_DAY
                        }
                    ]
                }
            ]
        }
    );
}

#[test]
fn test_query_user_portfolio_with_multiple_users_staking_in_multiple_liquidity_pools() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let manager = Address::generate(&env);
    let user_1 = Address::generate(&env);
    let user_2 = Address::generate(&env);

    let mut token1 = token_contract::Client::new(
        &env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    );
    let mut token2 = token_contract::Client::new(
        &env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    );
    let mut token3 = token_contract::Client::new(
        &env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    );
    let mut token4 = token_contract::Client::new(
        &env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    );

    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    if token4.address < token3.address {
        std::mem::swap(&mut token3, &mut token4);
    }

    token1.mint(&user_1, &100_000i128);
    token1.mint(&user_2, &2_000i128);
    token2.mint(&user_1, &100_000i128);
    token2.mint(&user_2, &2_000i128);

    token3.mint(&user_1, &1_000i128);
    token3.mint(&user_2, &2_000i128);
    token4.mint(&user_1, &4_000i128);
    token4.mint(&user_2, &8_000i128);

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    // first liquidity pool
    let first_lp_init_info = generate_lp_init_info(
        token1.address.clone(),
        token2.address.clone(),
        manager.clone(),
        admin.clone(),
        fee_recipient.clone(),
    );

    let first_lp_contract_addr = factory.create_liquidity_pool(
        &admin.clone(),
        &first_lp_init_info,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "PHO/BTC"),
        &PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );

    let first_lp_client = lp_contract::Client::new(&env, &first_lp_contract_addr);

    let first_stake_address = factory
        .query_pool_details(&first_lp_contract_addr)
        .pool_response
        .stake_address;

    let first_stake_client = stake_contract::Client::new(&env, &first_stake_address);

    // second liquidity pool
    let second_lp_init_info = generate_lp_init_info(
        token3.address.clone(),
        token4.address.clone(),
        manager.clone(),
        admin.clone(),
        fee_recipient.clone(),
    );

    let second_lp_contract_addr = factory.create_liquidity_pool(
        &admin.clone(),
        &second_lp_init_info,
        &String::from_str(&env, "Second Pool"),
        &String::from_str(&env, "PHO/ETH"),
        &PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );

    let second_lp_client = lp_contract::Client::new(&env, &second_lp_contract_addr);

    let second_stake_address = factory
        .query_pool_details(&second_lp_contract_addr)
        .pool_response
        .stake_address;

    let second_stake_client = stake_contract::Client::new(&env, &second_stake_address);

    // providing liquidity and assertions in first pool
    // provides liquidity in 50/50 ratio
    first_lp_client.provide_liquidity(
        &user_1.clone(),
        &Some(50_000i128),
        &Some(900i128),
        &Some(50_000i128),
        &Some(900i128),
        &None::<i64>,
        &None::<u64>,
        &false,
    );

    first_lp_client.provide_liquidity(
        &user_2.clone(),
        &Some(2_000i128),
        &Some(1_900i128),
        &Some(2_000i128),
        &Some(1_900i128),
        &None::<i64>,
        &None::<u64>,
        &false,
    );

    // first user portfolio in first pool after providing liquidity
    let first_user_first_portfolio = factory.query_user_portfolio(&user_1, &true);
    assert_eq!(
        first_user_first_portfolio,
        UserPortfolio {
            lp_portfolio: vec![
                &env,
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token1.address.clone(),
                            amount: 48_999i128,
                        },
                        Asset {
                            address: token2.address.clone(),
                            amount: 48_999i128,
                        }
                    )
                }
            ],
            stake_portfolio: vec![&env,]
        }
    );

    env.ledger().with_mut(|li| li.timestamp = ONE_DAY);
    first_stake_client.bond(&user_1, &1_000i128);

    // first user portfolio in first pool after staking
    let first_user_first_portfolio = factory.query_user_portfolio(&user_1, &true);
    assert_eq!(
        first_user_first_portfolio,
        UserPortfolio {
            lp_portfolio: vec![
                &env,
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token1.address.clone(),
                            amount: 48_999i128,
                        },
                        Asset {
                            address: token2.address.clone(),
                            amount: 48_999i128
                        }
                    )
                }
            ],
            stake_portfolio: vec![
                &env,
                StakePortfolio {
                    staking_contract: first_stake_address.clone(),
                    stakes: vec![
                        &env,
                        Stake {
                            stake: 1_000i128,
                            stake_timestamp: ONE_DAY
                        }
                    ]
                }
            ]
        }
    );

    // second user portfolio in first pool after providing liquidity
    let second_user_first_portfolio = factory.query_user_portfolio(&user_2, &true);
    assert_eq!(
        second_user_first_portfolio,
        UserPortfolio {
            lp_portfolio: vec![
                &env,
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token1.address.clone(),
                            amount: 1_999i128,
                        },
                        Asset {
                            address: token2.address.clone(),
                            amount: 1_999i128,
                        }
                    )
                }
            ],
            stake_portfolio: vec![&env,]
        }
    );

    // this time we bond just 50% of the lp share token for 2nd user
    first_stake_client.bond(&user_2, &1_000i128);

    // second user portfolio in first pool after staking
    let second_user_first_portfolio = factory.query_user_portfolio(&user_2, &true);
    assert_eq!(
        second_user_first_portfolio,
        UserPortfolio {
            lp_portfolio: vec![
                &env,
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token1.address.clone(),
                            amount: 1_999i128,
                        },
                        Asset {
                            address: token2.address.clone(),
                            amount: 1_999i128
                        }
                    )
                }
            ],
            stake_portfolio: vec![
                &env,
                StakePortfolio {
                    staking_contract: first_stake_address.clone(),
                    stakes: vec![
                        &env,
                        Stake {
                            stake: 1_000i128,
                            stake_timestamp: ONE_DAY
                        }
                    ]
                }
            ]
        }
    );

    // providing liquiditiy and assertions in second pool
    // provides liquidity in 25/75 ratio
    second_lp_client.provide_liquidity(
        &user_1.clone(),
        &Some(1_000i128),
        &Some(900i128),
        &Some(4_000i128),
        &Some(3_900i128),
        &None::<i64>,
        &None::<u64>,
        &false,
    );

    second_lp_client.provide_liquidity(
        &user_2.clone(),
        &Some(2_000i128),
        &Some(1_900i128),
        &Some(8_000i128),
        &Some(7_900i128),
        &None::<i64>,
        &None::<u64>,
        &false,
    );

    // first user portfolio with second pool after providing liquidity
    let first_user_with_second_portfolio = factory.query_user_portfolio(&user_1, &true);
    assert_eq!(
        first_user_with_second_portfolio,
        UserPortfolio {
            lp_portfolio: vec![
                &env,
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token1.address.clone(),
                            amount: 48_999i128,
                        },
                        Asset {
                            address: token2.address.clone(),
                            amount: 48_999i128,
                        }
                    )
                },
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token3.address.clone(),
                            amount: 499i128,
                        },
                        Asset {
                            address: token4.address.clone(),
                            amount: 1_999i128,
                        }
                    )
                },
            ],
            stake_portfolio: vec![
                &env,
                StakePortfolio {
                    staking_contract: first_stake_address.clone(),
                    stakes: vec![
                        &env,
                        Stake {
                            stake: 1_000i128,
                            stake_timestamp: ONE_DAY
                        }
                    ]
                }
            ]
        }
    );

    // after providing liquidity to 2nd pool user1 has 1_000 lp share tokens
    second_stake_client.bond(&user_1, &1_000i128);

    // first user portfolio with second pool after staking
    let first_user_first_portfolio = factory.query_user_portfolio(&user_1, &true);
    assert_eq!(
        first_user_first_portfolio,
        UserPortfolio {
            lp_portfolio: vec![
                &env,
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token1.address.clone(),
                            amount: 48_999i128,
                        },
                        Asset {
                            address: token2.address.clone(),
                            amount: 48_999i128
                        }
                    )
                },
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token3.address.clone(),
                            amount: 499i128,
                        },
                        Asset {
                            address: token4.address.clone(),
                            amount: 1_999i128,
                        }
                    )
                },
            ],
            stake_portfolio: vec![
                &env,
                StakePortfolio {
                    staking_contract: first_stake_address.clone(),
                    stakes: vec![
                        &env,
                        Stake {
                            stake: 1_000i128,
                            stake_timestamp: ONE_DAY
                        }
                    ]
                },
                StakePortfolio {
                    staking_contract: second_stake_address.clone(),
                    stakes: vec![
                        &env,
                        Stake {
                            stake: 1_000i128,
                            stake_timestamp: ONE_DAY
                        }
                    ]
                }
            ]
        }
    );

    // second user portfolio with second pool after providing liquidity
    let second_user_second_portfolio = factory.query_user_portfolio(&user_2, &true);
    assert_eq!(
        second_user_second_portfolio,
        UserPortfolio {
            lp_portfolio: vec![
                &env,
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token1.address.clone(),
                            amount: 1_999i128,
                        },
                        Asset {
                            address: token2.address.clone(),
                            amount: 1_999i128,
                        }
                    )
                },
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token3.address.clone(),
                            amount: 1_999i128,
                        },
                        Asset {
                            address: token4.address.clone(),
                            amount: 7_999i128,
                        }
                    )
                },
            ],
            stake_portfolio: vec![
                &env,
                StakePortfolio {
                    staking_contract: first_stake_address.clone(),
                    stakes: vec![
                        &env,
                        Stake {
                            stake: 1_000i128,
                            stake_timestamp: ONE_DAY
                        }
                    ]
                },
            ]
        }
    );

    // this time we bond just 75% of the lp share token for 2nd user
    second_stake_client.bond(&user_2, &3_000i128);

    // second user portfolio with second pool after staking
    let second_user_second_portfolio = factory.query_user_portfolio(&user_2, &true);
    assert_eq!(
        second_user_second_portfolio,
        UserPortfolio {
            lp_portfolio: vec![
                &env,
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token1.address.clone(),
                            amount: 1_999i128,
                        },
                        Asset {
                            address: token2.address.clone(),
                            amount: 1_999i128
                        }
                    )
                },
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token3.address.clone(),
                            amount: 1_999i128,
                        },
                        Asset {
                            address: token4.address.clone(),
                            amount: 7_999i128
                        }
                    )
                }
            ],
            stake_portfolio: vec![
                &env,
                StakePortfolio {
                    staking_contract: first_stake_address.clone(),
                    stakes: vec![
                        &env,
                        Stake {
                            stake: 1_000i128,
                            stake_timestamp: ONE_DAY
                        }
                    ]
                },
                StakePortfolio {
                    staking_contract: second_stake_address.clone(),
                    stakes: vec![
                        &env,
                        Stake {
                            stake: 3_000i128,
                            stake_timestamp: ONE_DAY
                        }
                    ]
                }
            ]
        }
    );

    // second user portfolio with second pool without staking
    let second_user_second_portfolio = factory.query_user_portfolio(&user_2, &false);
    assert_eq!(
        second_user_second_portfolio,
        UserPortfolio {
            lp_portfolio: vec![
                &env,
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token1.address.clone(),
                            amount: 1_999i128,
                        },
                        Asset {
                            address: token2.address.clone(),
                            amount: 1_999i128
                        }
                    )
                },
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token3.address.clone(),
                            amount: 1_999i128,
                        },
                        Asset {
                            address: token4.address.clone(),
                            amount: 7_999i128
                        }
                    )
                }
            ],
            stake_portfolio: vec![&env,]
        }
    );
}

#[test]
fn test_query_user_portfolio_without_stake() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let manager = Address::generate(&env);
    let user_1 = Address::generate(&env);

    let mut token1 = token_contract::Client::new(
        &env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    );
    let mut token2 = token_contract::Client::new(
        &env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    );

    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    token1.mint(&user_1, &50_000i128);
    token2.mint(&user_1, &50_000i128);
    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    let first_token_init_info = TokenInitInfo {
        token_a: token1.address.clone(),
        token_b: token2.address.clone(),
    };
    let first_stake_init_info = StakeInitInfo {
        min_bond: 1i128,
        min_reward: 1i128,
        manager,
        max_complexity: 10u32,
    };

    let first_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: fee_recipient.clone(),
        max_allowed_slippage_bps: 100,
        default_slippage_bps: 2_500,
        max_allowed_spread_bps: 100,
        swap_fee_bps: 0,
        max_referral_bps: 0,
        token_init_info: first_token_init_info.clone(),
        stake_init_info: first_stake_init_info,
    };

    let lp_contract_addr = factory.create_liquidity_pool(
        &admin.clone(),
        &first_lp_init_info,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "PHO/BTC"),
        &PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );

    let first_lp_client = lp_contract::Client::new(&env, &lp_contract_addr);

    first_lp_client.provide_liquidity(
        &user_1.clone(),
        &Some(50_000i128),
        &Some(40_000i128),
        &Some(50_000i128),
        &Some(40_000i128),
        &None::<i64>,
        &None::<u64>,
        &false,
    );

    let result = factory.query_user_portfolio(&user_1, &false);
    assert_eq!(
        result,
        UserPortfolio {
            lp_portfolio: vec![
                &env,
                LpPortfolio {
                    assets: (
                        Asset {
                            address: token1.address,
                            amount: 49_000i128
                        },
                        Asset {
                            address: token2.address,
                            amount: 49_000i128
                        }
                    )
                }
            ],
            stake_portfolio: vec![&env]
        }
    );
}

#[test]
fn test_ttl_extensions_with_multiple_pool_queries() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    env.ledger().with_mut(|li| {
        li.min_persistent_entry_ttl = DAY_IN_LEDGERS; // 17_280
        li.max_entry_ttl = 30 * DAY_IN_LEDGERS; // 518_400
    });

    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let manager = Address::generate(&env);

    let mut token1 = token_contract::Client::new(
        &env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    );
    let mut token2 = token_contract::Client::new(
        &env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    );

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let factory = deploy_factory_contract(&env, Some(admin.clone()));
    let factory_address = factory.address.clone();
    let lp_init_info = generate_lp_init_info(
        token1.address.clone(),
        token2.address.clone(),
        manager,
        admin.clone(),
        fee_recipient,
    );

    let lp_address = factory.create_liquidity_pool(
        &admin,
        &lp_init_info,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "PHO/BTC"),
        &PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );

    // initial TTL verification
    let (initial_instance_ttl, initial_persistent_ttl) = env.as_contract(&factory_address, || {
        (
            env.storage().instance().get_ttl(),
            env.storage().persistent().get_ttl(&DataKey::LpVec),
        )
    });
    // validate initial state
    assert_eq!(initial_instance_ttl, INSTANCE_TARGET_TTL);
    assert!(
        initial_persistent_ttl >= PERSISTENT_TARGET_TTL - 1,
        "Initial persistent TTL should be at least {} (was {})",
        PERSISTENT_TARGET_TTL - 1,
        initial_persistent_ttl
    );

    // first extension
    env.ledger().with_mut(|li| {
        li.sequence_number += INSTANCE_TARGET_TTL - INSTANCE_RENEWAL_THRESHOLD + 1;
    });

    let _ = factory.query_pools();
    let (instance_ttl1, persistent_ttl1) = env.as_contract(&factory_address, || {
        (
            env.storage().instance().get_ttl(),
            env.storage().persistent().get_ttl(&DataKey::LpVec),
        )
    });

    assert_eq!(instance_ttl1, INSTANCE_TARGET_TTL);
    assert!(
        persistent_ttl1 >= PERSISTENT_TARGET_TTL - 1,
        "Persistent TTL after first extension should be at least {} (was {})",
        PERSISTENT_TARGET_TTL - 1,
        persistent_ttl1
    );

    // second extension
    env.ledger()
        .with_mut(|li| li.sequence_number += INSTANCE_TARGET_TTL - INSTANCE_RENEWAL_THRESHOLD);
    let _ = factory.query_pools();
    let (instance_ttl2, persistent_ttl2) = env.as_contract(&factory_address, || {
        (
            env.storage().instance().get_ttl(),
            env.storage().persistent().get_ttl(&DataKey::LpVec),
        )
    });
    assert_eq!(instance_ttl2, INSTANCE_TARGET_TTL);
    assert!(
        persistent_ttl2 >= PERSISTENT_TARGET_TTL - 1,
        "Persistent TTL after second extension should be at least {} (was {})",
        PERSISTENT_TARGET_TTL - 1,
        persistent_ttl2
    );

    // third extension
    env.ledger()
        .with_mut(|li| li.sequence_number += INSTANCE_TARGET_TTL - INSTANCE_RENEWAL_THRESHOLD);
    let _ = factory.query_pools();
    let (instance_ttl3, persistent_ttl3) = env.as_contract(&factory_address, || {
        (
            env.storage().instance().get_ttl(),
            env.storage().persistent().get_ttl(&DataKey::LpVec),
        )
    });
    assert_eq!(instance_ttl3, INSTANCE_TARGET_TTL);
    assert!(
        persistent_ttl3 >= PERSISTENT_TARGET_TTL - 1,
        "Persistent TTL after third extension should be at least {} (was {})",
        PERSISTENT_TARGET_TTL - 1,
        persistent_ttl3
    );

    // fourth extension
    env.ledger()
        .with_mut(|li| li.sequence_number += INSTANCE_TARGET_TTL - INSTANCE_RENEWAL_THRESHOLD);
    let _ = factory.query_pools();
    let (instance_ttl4, persistent_ttl4) = env.as_contract(&factory_address, || {
        (
            env.storage().instance().get_ttl(),
            env.storage().persistent().get_ttl(&DataKey::LpVec),
        )
    });
    assert_eq!(instance_ttl4, INSTANCE_TARGET_TTL);
    assert!(
        persistent_ttl4 >= PERSISTENT_TARGET_TTL - 1,
        "Persistent TTL after fourth extension should be at least {} (was {})",
        PERSISTENT_TARGET_TTL - 1,
        persistent_ttl4
    );

    // fifth extension
    env.ledger()
        .with_mut(|li| li.sequence_number += INSTANCE_TARGET_TTL - INSTANCE_RENEWAL_THRESHOLD);
    let _ = factory.query_pools();
    let (instance_ttl5, persistent_ttl5) = env.as_contract(&factory_address, || {
        (
            env.storage().instance().get_ttl(),
            env.storage().persistent().get_ttl(&DataKey::LpVec),
        )
    });
    assert_eq!(instance_ttl5, INSTANCE_TARGET_TTL);
    assert!(
        persistent_ttl5 >= PERSISTENT_TARGET_TTL - 1,
        "Persistent TTL after fifth extension should be at least {} (was {})",
        PERSISTENT_TARGET_TTL - 1,
        persistent_ttl5
    );

    // final validation
    env.ledger()
        .with_mut(|li| li.sequence_number += INSTANCE_TARGET_TTL - 100);
    let final_pools = factory.query_pools();
    assert!(!final_pools.is_empty());
    assert_eq!(final_pools.get(0).unwrap(), lp_address);
}
