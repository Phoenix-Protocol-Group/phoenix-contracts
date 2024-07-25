extern crate std;
use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

use super::setup::{deploy_liquidity_pool_contract, deploy_token_contract};
use crate::{
    stake_contract,
    storage::{Config, PairType},
};

use crate::contract::{LiquidityPool, LiquidityPoolClient};
use crate::tests::setup::{install_stake_wasm, install_token_wasm};

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
    let stake_manager = Address::generate(&env);
    let stake_owner = Address::generate(&env);

    let swap_fees = 0i64;
    let pool = deploy_liquidity_pool_contract(
        &env,
        Some(admin1.clone()),
        (&token1.address, &token2.address),
        swap_fees,
        user1.clone(),
        500,
        200,
        stake_manager.clone(),
        stake_owner.clone(),
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
            pool_type: PairType::Xyk,
            total_fee_bps: 0,
            fee_recipient: user1,
            max_allowed_slippage_bps: 500,
            max_allowed_spread_bps: 200,
            max_referral_bps: 5_000,
            default_slippage_bps: 100i64,
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
                manager: stake_manager,
                owner: stake_owner,
                max_complexity: 10,
            }
        }
    );
}

#[test]
#[should_panic(expected = "Pool: Initialize: initializing contract twice is not allowed")]
fn second_pool_deployment_should_fail() {
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

    let pool = LiquidityPoolClient::new(&env, &env.register_contract(None, LiquidityPool {}));

    let token_wasm_hash = install_token_wasm(&env);
    let stake_wasm_hash = install_stake_wasm(&env);
    let fee_recipient = user;
    let max_allowed_slippage = 5_000i64; // 50% if not specified
    let max_allowed_spread = 500i64; // 5% if not specified

    let token_init_info = TokenInitInfo {
        token_a: token1.address.clone(),
        token_b: token2.address.clone(),
    };
    let stake_init_info = StakeInitInfo {
        min_bond: 10i128,
        min_reward: 5i128,
        manager: Address::generate(&env),
        max_complexity: 10u32,
    };

    let lp_init_info = LiquidityPoolInitInfo {
        admin: admin1,
        swap_fee_bps: 0i64,
        fee_recipient,
        max_allowed_slippage_bps: max_allowed_slippage,
        default_slippage_bps: 2_500,
        max_allowed_spread_bps: max_allowed_spread,
        max_referral_bps: 500,
        token_init_info,
        stake_init_info,
    };

    pool.initialize(
        &stake_wasm_hash,
        &token_wasm_hash,
        &lp_init_info,
        &Address::generate(&env),
        &10u32,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "PHOBTC"),
        &100i64,
        &1_000,
        &10i128,
    );

    pool.initialize(
        &stake_wasm_hash,
        &token_wasm_hash,
        &lp_init_info,
        &Address::generate(&env),
        &10u32,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "PHOBTC"),
        &100i64,
        &1_000,
        &10i128,
    );
}
