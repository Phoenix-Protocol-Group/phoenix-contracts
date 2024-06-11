extern crate std;
use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

use super::setup::{deploy_stable_liquidity_pool_contract, deploy_token_contract};
use crate::contract::{StableLiquidityPool, StableLiquidityPoolClient};
use crate::tests::setup::{install_stake_wasm, install_token_wasm};
use crate::{
    stake_contract,
    storage::{Config, PairType},
};

#[test]
fn confirm_stake_contract_deployment() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }
    let user1 = Address::generate(&env);
    let swap_fees = 0i64;
    let factory = Address::generate(&env);
    let stake_manager = Address::generate(&env);
    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        Some(admin1.clone()),
        (&token1.address, &token2.address),
        swap_fees,
        user1.clone(),
        500,
        200,
        stake_manager.clone(),
        factory.clone(),
        None,
    );

    let share_token_address = pool.query_share_token_address();
    let stake_token_address = pool.query_stake_contract_address();

    assert_eq!(
        pool.query_config(),
        Config {
            token_a: token1.address.clone(),
            token_b: token2.address.clone(),
            share_token: share_token_address.clone(),
            stake_contract: stake_token_address.clone(),
            pool_type: PairType::Stable,
            total_fee_bps: 0,
            fee_recipient: user1,
            max_allowed_slippage_bps: 500,
            max_allowed_spread_bps: 200,
        }
    );

    let stake_client = stake_contract::Client::new(&env, &stake_token_address);
    assert_eq!(
        stake_client.query_config(),
        stake_contract::ConfigResponse {
            config: stake_contract::Config {
                lp_token: share_token_address,
                min_bond: 10,
                min_reward: 5,
                owner: factory,
                manager: stake_manager,
                max_complexity: 10,
            }
        }
    );
}

#[test]
#[should_panic(expected = "Pool stable: Initialize: initializing contract twice is not allowed")]
fn second_pool_stable_deployment_should_fail() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);
    let user = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }

    let pool =
        StableLiquidityPoolClient::new(&env, &env.register_contract(None, StableLiquidityPool {}));

    let token_wasm_hash = install_token_wasm(&env);
    let stake_wasm_hash = install_stake_wasm(&env);
    let fee_recipient = user;
    let max_allowed_slippage = 5_000i64; // 50% if not specified
    let max_allowed_spread = 500i64; // 5% if not specified
    let amp = 6u64;
    let stake_manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let token_init_info = TokenInitInfo {
        token_a: token1.address.clone(),
        token_b: token2.address.clone(),
    };
    let stake_init_info = StakeInitInfo {
        min_bond: 10i128,
        min_reward: 5i128,
        manager: stake_manager.clone(),
        max_complexity: 10,
    };

    let lp_init_info = LiquidityPoolInitInfo {
        admin: admin1,
        swap_fee_bps: 0i64,
        fee_recipient,
        max_allowed_slippage_bps: max_allowed_slippage,
        max_allowed_spread_bps: max_allowed_spread,
        max_referral_bps: 500,
        token_init_info,
        stake_init_info,
    };

    pool.initialize(
        &stake_wasm_hash,
        &token_wasm_hash,
        &lp_init_info,
        &factory,
        &10, // LP share decimals, unused
        &String::from_str(&env, "LP_SHARE_TOKEN"),
        &String::from_str(&env, "PHOBTCLP"),
        &amp,
    );
    pool.initialize(
        &stake_wasm_hash,
        &token_wasm_hash,
        &lp_init_info,
        &factory,
        &10, // LP share decimals, unused
        &String::from_str(&env, "LP_SHARE_TOKEN"),
        &String::from_str(&env, "PHOBTCLP"),
        &amp,
    );
}

#[test]
#[should_panic(
    expected = "Pool Stable: Initialize: First token must be alphabetically smaller than second token"
)]
fn pool_stable_initialization_should_fail_with_token_a_bigger_than_token_b() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);
    let user = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);
    if token2.address >= token1.address {
        std::mem::swap(&mut token2, &mut token1);
        std::mem::swap(&mut admin2, &mut admin1);
    }

    let pool =
        StableLiquidityPoolClient::new(&env, &env.register_contract(None, StableLiquidityPool {}));

    let token_wasm_hash = install_token_wasm(&env);
    let stake_wasm_hash = install_stake_wasm(&env);
    let fee_recipient = user;
    let max_allowed_slippage = 5_000i64; // 50% if not specified
    let max_allowed_spread = 500i64; // 5% if not specified
    let amp = 6u64;
    let stake_manager = Address::generate(&env);
    let factory = Address::generate(&env);

    let token_init_info = TokenInitInfo {
        token_a: token1.address.clone(),
        token_b: token2.address.clone(),
    };
    let stake_init_info = StakeInitInfo {
        min_bond: 10i128,
        min_reward: 5i128,
        manager: stake_manager.clone(),
        max_complexity: 10,
    };

    let lp_init_info = LiquidityPoolInitInfo {
        admin: admin1,
        swap_fee_bps: 0i64,
        fee_recipient,
        max_allowed_slippage_bps: max_allowed_slippage,
        max_allowed_spread_bps: max_allowed_spread,
        max_referral_bps: 500,
        token_init_info,
        stake_init_info,
    };

    pool.initialize(
        &stake_wasm_hash,
        &token_wasm_hash,
        &lp_init_info,
        &factory,
        &10, // LP share decimals, unused
        &String::from_str(&env, "LP_SHARE_TOKEN"),
        &String::from_str(&env, "PHOBTCLP"),
        &amp,
    );
}
