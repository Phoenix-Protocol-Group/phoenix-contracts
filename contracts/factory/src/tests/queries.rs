use super::setup::{
    deploy_factory_contract, install_lp_contract, install_stake_wasm, install_token_wasm,
};
use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};

use soroban_sdk::arbitrary::std;
use soroban_sdk::{contracttype, testutils::Address as _, Address, Env, Symbol, Vec};

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
    let admin = Address::random(&env);
    let user = Address::random(&env);

    let mut token1 = Address::random(&env);
    let mut token2 = Address::random(&env);
    let mut token3 = Address::random(&env);
    let mut token4 = Address::random(&env);
    let mut token5 = Address::random(&env);
    let mut token6 = Address::random(&env);

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
        token_wasm_hash: install_token_wasm(&env),
        token_a: token1.clone(),
        token_b: token2.clone(),
    };
    let first_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 10i128,
        max_distributions: 10u32,
        min_reward: 5i128,
    };

    let second_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: token3.clone(),
        token_b: token4.clone(),
    };
    let second_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 5i128,
        max_distributions: 5u32,
        min_reward: 2i128,
    };

    let third_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: token5.clone(),
        token_b: token6.clone(),
    };
    let third_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 6i128,
        max_distributions: 6u32,
        min_reward: 3i128,
    };

    let lp_wasm_hash = install_lp_contract(&env);

    let first_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 5_000,
        max_allowed_spread_bps: 500,
        share_token_decimals: 7,
        swap_fee_bps: 0,
        max_referral_bps: 5_000,
        token_init_info: first_token_init_info.clone(),
        stake_init_info: first_stake_init_info,
    };

    let second_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 4_000,
        max_allowed_spread_bps: 400,
        share_token_decimals: 6,
        swap_fee_bps: 0,
        max_referral_bps: 5_000,
        token_init_info: second_token_init_info,
        stake_init_info: second_stake_init_info,
    };

    let third_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        lp_wasm_hash,
        max_allowed_slippage_bps: 4_000,
        max_allowed_spread_bps: 400,
        share_token_decimals: 6,
        swap_fee_bps: 0,
        max_referral_bps: 5_000,
        token_init_info: third_token_init_info,
        stake_init_info: third_stake_init_info,
    };

    let lp_contract_addr = factory.create_liquidity_pool(&first_lp_init_info);
    let second_lp_contract_addr = factory.create_liquidity_pool(&second_lp_init_info);
    let third_lp_contract_addr = factory.create_liquidity_pool(&third_lp_init_info);

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
#[should_panic(expected = "Factory: query_for_pool_by_token_pair failed: No liquidity pool found")]
fn test_queries_by_tuple() {
    let env = Env::default();
    let admin = Address::random(&env);
    let user = Address::random(&env);

    let mut token1 = Address::random(&env);
    let mut token2 = Address::random(&env);
    let mut token3 = Address::random(&env);
    let mut token4 = Address::random(&env);
    let mut token5 = Address::random(&env);
    let mut token6 = Address::random(&env);

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
        token_wasm_hash: install_token_wasm(&env),
        token_a: token1.clone(),
        token_b: token2.clone(),
    };
    let first_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 10i128,
        max_distributions: 10u32,
        min_reward: 5i128,
    };

    let second_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: token3.clone(),
        token_b: token4.clone(),
    };
    let second_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 5i128,
        max_distributions: 5u32,
        min_reward: 2i128,
    };

    let third_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: token5.clone(),
        token_b: token6.clone(),
    };
    let third_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 6i128,
        max_distributions: 6u32,
        min_reward: 3i128,
    };

    let lp_wasm_hash = install_lp_contract(&env);

    let first_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 5_000,
        max_allowed_spread_bps: 500,
        share_token_decimals: 7,
        swap_fee_bps: 0,
        max_referral_bps: 5_000,
        token_init_info: first_token_init_info.clone(),
        stake_init_info: first_stake_init_info,
    };

    let second_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 4_000,
        max_allowed_spread_bps: 400,
        share_token_decimals: 6,
        swap_fee_bps: 0,
        max_referral_bps: 5_000,
        token_init_info: second_token_init_info,
        stake_init_info: second_stake_init_info,
    };

    let third_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        lp_wasm_hash,
        max_allowed_slippage_bps: 4_000,
        max_allowed_spread_bps: 400,
        share_token_decimals: 6,
        swap_fee_bps: 0,
        max_referral_bps: 5_000,
        token_init_info: third_token_init_info,
        stake_init_info: third_stake_init_info,
    };

    let lp_contract_addr = factory.create_liquidity_pool(&first_lp_init_info);
    let second_lp_contract_addr = factory.create_liquidity_pool(&second_lp_init_info);
    let third_lp_contract_addr = factory.create_liquidity_pool(&third_lp_init_info);

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

    factory.query_for_pool_by_token_pair(&Address::random(&env), &Address::random(&env));
}
