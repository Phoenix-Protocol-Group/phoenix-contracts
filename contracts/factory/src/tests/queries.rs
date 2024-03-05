use super::setup::{deploy_factory_contract, deploy_token_contract, install_token_wasm};
use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};
use soroban_sdk::testutils::arbitrary::std::dbg;

use soroban_sdk::vec;
use soroban_sdk::{
    contracttype,
    testutils::{arbitrary::std, Address as _},
    Address, Env, IntoVal, String, Symbol, Val, Vec,
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

    let mut token1 = Address::generate(&env);
    let mut token2 = Address::generate(&env);
    let mut token3 = Address::generate(&env);
    let mut token4 = Address::generate(&env);
    let mut token5 = Address::generate(&env);
    let mut token6 = Address::generate(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    if token2 < token1 {
        std::mem::swap(&mut token1, &mut token2);
    }

    if token4 < token3 {
        std::mem::swap(&mut token3, &mut token4);
    }

    if token6 < token5 {
        std::mem::swap(&mut token5, &mut token6);
    }

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    let first_token_init_info = TokenInitInfo {
        token_a: token1.clone(),
        token_b: token2.clone(),
    };
    let first_stake_init_info = StakeInitInfo {
        min_bond: 10i128,
        min_reward: 5i128,
        manager: Address::generate(&env),
    };

    let second_token_init_info = TokenInitInfo {
        token_a: token3.clone(),
        token_b: token4.clone(),
    };
    let second_stake_init_info = StakeInitInfo {
        min_bond: 5i128,
        min_reward: 2i128,
        manager: Address::generate(&env),
    };

    let third_token_init_info = TokenInitInfo {
        token_a: token5.clone(),
        token_b: token6.clone(),
    };
    let third_stake_init_info = StakeInitInfo {
        min_bond: 6i128,
        min_reward: 3i128,
        manager: Address::generate(&env),
    };

    let first_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        max_allowed_slippage_bps: 5_000,
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
    );
    let second_lp_contract_addr = factory.create_liquidity_pool(
        &admin.clone(),
        &second_lp_init_info,
        &String::from_str(&env, "Pool #2"),
        &String::from_str(&env, "PHO/ETH"),
    );
    let third_lp_contract_addr = factory.create_liquidity_pool(
        &admin.clone(),
        &third_lp_init_info,
        &String::from_str(&env, "Pool #3"),
        &String::from_str(&env, "PHO/XLM"),
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

    assert_eq!(token1, first_result.pool_response.asset_a.address);
    assert_eq!(token2, first_result.pool_response.asset_b.address);
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

    assert_eq!(token3, second_result.pool_response.asset_a.address);
    assert_eq!(token4, second_result.pool_response.asset_b.address);
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

    assert_eq!(token5, third_result.pool_response.asset_a.address);
    assert_eq!(token6, third_result.pool_response.asset_b.address);
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

    let first_lp_address_by_tuple = factory.query_for_pool_by_token_pair(&token1, &token2);
    assert_eq!(first_lp_address_by_tuple, lp_contract_addr);
}

#[test]
fn test_queries_by_tuple() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let mut token1 = Address::generate(&env);
    let mut token2 = Address::generate(&env);
    let mut token3 = Address::generate(&env);
    let mut token4 = Address::generate(&env);
    let mut token5 = Address::generate(&env);
    let mut token6 = Address::generate(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    if token2 < token1 {
        std::mem::swap(&mut token1, &mut token2);
    }

    if token4 < token3 {
        std::mem::swap(&mut token3, &mut token4);
    }

    if token6 < token5 {
        std::mem::swap(&mut token5, &mut token6);
    }

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    let first_token_init_info = TokenInitInfo {
        token_a: token1.clone(),
        token_b: token2.clone(),
    };
    let first_stake_init_info = StakeInitInfo {
        min_bond: 10i128,
        min_reward: 5i128,
        manager: Address::generate(&env),
    };

    let second_token_init_info = TokenInitInfo {
        token_a: token3.clone(),
        token_b: token4.clone(),
    };
    let second_stake_init_info = StakeInitInfo {
        min_bond: 5i128,
        min_reward: 2i128,
        manager: Address::generate(&env),
    };

    let third_token_init_info = TokenInitInfo {
        token_a: token5.clone(),
        token_b: token6.clone(),
    };
    let third_stake_init_info = StakeInitInfo {
        min_bond: 6i128,
        min_reward: 3i128,
        manager: Address::generate(&env),
    };

    let first_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        max_allowed_slippage_bps: 5_000,
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
    );
    let second_lp_contract_addr = factory.create_liquidity_pool(
        &admin.clone(),
        &second_lp_init_info,
        &String::from_str(&env, "Pool #2"),
        &String::from_str(&env, "PHO/ETH"),
    );
    let third_lp_contract_addr = factory.create_liquidity_pool(
        &admin.clone(),
        &third_lp_init_info,
        &String::from_str(&env, "Pool #3"),
        &String::from_str(&env, "PHO/XLM"),
    );

    let first_result = factory.query_pool_details(&lp_contract_addr);

    assert_eq!(token1, first_result.pool_response.asset_a.address);
    assert_eq!(token2, first_result.pool_response.asset_b.address);
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

    assert_eq!(token3, second_result.pool_response.asset_a.address);
    assert_eq!(token4, second_result.pool_response.asset_b.address);
    assert_eq!(
        second_share_token_addr,
        second_result.pool_response.asset_lp_share.address
    );
    assert_eq!(second_lp_contract_addr, second_result.pool_address);

    let first_lp_address_by_tuple = factory.query_for_pool_by_token_pair(&token2, &token1);
    let second_lp_address_by_tuple = factory.query_for_pool_by_token_pair(&token3, &token4);
    let third_lp_address_by_tuple = factory.query_for_pool_by_token_pair(&token5, &token6);

    assert_eq!(first_lp_address_by_tuple, lp_contract_addr);
    assert_eq!(second_lp_address_by_tuple, second_lp_contract_addr);
    assert_eq!(third_lp_address_by_tuple, third_lp_contract_addr);
}

#[test]
#[should_panic(expected = "Factory: query_for_pool_by_token_pair failed: No liquidity pool found")]
fn test_queries_by_tuple_errors() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    factory.query_for_pool_by_token_pair(&Address::generate(&env), &Address::generate(&env));
}

#[test]
fn test_query_token_amount_per_liquidity_pool_per_user() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let user_1 = Address::generate(&env);
    let user_2 = Address::generate(&env);
    let user_3 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    let mut token3 = deploy_token_contract(&env, &admin);
    let mut token4 = deploy_token_contract(&env, &admin);
    let mut token5 = deploy_token_contract(&env, &admin);
    let mut token6 = deploy_token_contract(&env, &admin);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    if token4.address < token3.address {
        std::mem::swap(&mut token3, &mut token4);
    }

    if token6.address < token5.address {
        std::mem::swap(&mut token5, &mut token6);
    }

    token1.mint(&user_1, &10_000i128);
    let user1_token1_balance: i128 = env.invoke_contract(
        &token1.address,
        &Symbol::new(&env, "balance"),
        vec![&env, user_1.into_val(&env)],
    );

    assert_eq!(user1_token1_balance, 10_000i128);

    token2.mint(&user_1, &10_000i128);
    let user1_token2_balance: i128 = env.invoke_contract(
        &token2.address,
        &Symbol::new(&env, "balance"),
        vec![&env, user_1.into_val(&env)],
    );

    assert_eq!(user1_token2_balance, 10_000i128);

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    let first_token_init_info = TokenInitInfo {
        token_a: token1.address.clone(),
        token_b: token2.address.clone(),
    };
    let first_stake_init_info = StakeInitInfo {
        min_bond: 1i128,
        min_reward: 1i128,
        manager: admin.clone(),
    };

    let first_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: fee_recipient.clone(),
        max_allowed_slippage_bps: 100,
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

    // testing the liquidity providing

    // this is just to test why it always fail
    // let init_fn_args: Vec<Val> =
    //     (token1.address, 10).into_val(&env);
    // env.invoke_contract::<Val>(
    //     &lp_contract_addr,
    //     &Symbol::new(&env, "simulate_swap"),
    //     init_fn_args,
    // );
    dbg!("before");

    let init_fn_args: Vec<Val> = (
        user_1.clone(),
        Some(100),
        Some(100),
        Some(100),
        Some(100),
        None::<i128>,
    )
        .into_val(&env);
    env.invoke_contract::<Val>(
        &lp_contract_addr,
        &Symbol::new(&env, "provide_liquidity"),
        init_fn_args,
    );
    dbg!("after");

    let result = factory.get_user_portfolio(&user_1);
    dbg!(result);
}
