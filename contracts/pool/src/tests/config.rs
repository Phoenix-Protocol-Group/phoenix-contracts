use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};
use soroban_sdk::{
    testutils::{arbitrary::std, Address as _},
    Address, Env, String,
};

use super::setup::{
    deploy_liquidity_pool_contract, deploy_token_contract, install_stake_wasm, install_token_wasm,
};
use crate::{
    contract::{LiquidityPool, LiquidityPoolClient},
    storage::{Config, PairType},
};

#[should_panic(
    expected = "Pool: Initialize: First token must be alphabetically smaller than second token"
)]
#[test]
fn test_initialize_with_bigger_first_token_should_fail() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token1.address < token2.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let fee_recipient = Address::generate(&env);

    let token_init_info = TokenInitInfo {
        token_a: token1.address,
        token_b: token2.address,
    };
    let stake_init_info = StakeInitInfo {
        min_bond: 10i128,
        min_reward: 5i128,
        manager: Address::generate(&env),
        max_complexity: 10u32,
    };
    let stake_wasm_hash = install_stake_wasm(&env);
    let token_wasm_hash = install_token_wasm(&env);

    let lp_init_info = LiquidityPoolInitInfo {
        admin,
        swap_fee_bps: 0,
        fee_recipient,
        max_allowed_slippage_bps: 5_000,
        default_slippage_bps: 2_500,
        max_allowed_spread_bps: 1_000,
        max_referral_bps: 5_000,
        token_init_info,
        stake_init_info,
    };

    let _ = LiquidityPoolClient::new(
        &env,
        &env.register(
            LiquidityPool,
            (
                &stake_wasm_hash,
                &token_wasm_hash,
                lp_init_info,
                &Address::generate(&env),
                String::from_str(&env, "Pool"),
                String::from_str(&env, "PHOBTC"),
                &100i64,
                &1_000i64,
            ),
        ),
    );
}

#[test]
fn update_config() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

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
        stake_manager,
        stake_owner,
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
        }
    );

    // update fees and recipient
    pool.update_config(
        &None,
        &Some(500i64), // 5% fees
        &Some(admin2.clone()),
        &None,
        &None,
        &Some(1_000i64),
    );
    assert_eq!(
        pool.query_config(),
        Config {
            token_a: token1.address.clone(),
            token_b: token2.address.clone(),
            share_token: share_token_address.clone(),
            stake_contract: stake_token_address.clone(),
            pool_type: PairType::Xyk,
            total_fee_bps: 500,
            fee_recipient: admin2.clone(),
            max_allowed_slippage_bps: 500,
            max_allowed_spread_bps: 200,
            max_referral_bps: 1_000,
        }
    );

    // update slippage and spread
    pool.update_config(&None, &None, &None, &None, &Some(5_000i64), &Some(500));
    assert_eq!(
        pool.query_config(),
        Config {
            token_a: token1.address.clone(),
            token_b: token2.address,
            share_token: share_token_address,
            stake_contract: stake_token_address,
            pool_type: PairType::Xyk,
            total_fee_bps: 500,
            fee_recipient: admin2,
            max_allowed_slippage_bps: 500,
            max_allowed_spread_bps: 5_000,
            max_referral_bps: 500,
        }
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn update_config_unauthorized() {
    let env = Env::default();

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
        user1,
        500,
        200,
        stake_manager,
        stake_owner,
    );

    pool.update_config(
        &None,
        &Some(500i64), // 5% fees
        &Some(admin2.clone()),
        &None,
        &None,
        &None,
    );
}

#[test]
fn update_config_update_admin() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

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
        stake_manager,
        stake_owner,
    );

    // update admin to new admin
    pool.update_config(&Some(admin2.clone()), &None, &None, &None, &None, &None);

    let share_token_address = pool.query_share_token_address();
    let stake_token_address = pool.query_stake_contract_address();

    // now update succeeds
    pool.update_config(&Some(admin2.clone()), &None, &None, &None, &None, &None);
    assert_eq!(
        pool.query_config(),
        Config {
            token_a: token1.address.clone(),
            token_b: token2.address,
            share_token: share_token_address,
            stake_contract: stake_token_address,
            pool_type: PairType::Xyk,
            total_fee_bps: 0,
            fee_recipient: user1,
            max_allowed_slippage_bps: 500,
            max_allowed_spread_bps: 200,
            max_referral_bps: 5_000,
        }
    );
}

#[test]
#[should_panic(expected = "The value 10100 is out of range. Must be between 0 and 10000 bps.")]
fn update_config_too_high_fees() {
    let env = Env::default();
    env.mock_all_auths();

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
        user1,
        500,
        200,
        stake_manager,
        stake_owner,
    );

    // update fees and recipient
    pool.update_config(
        &None,
        &Some(10_100i64), // 101% fees
        &Some(admin2.clone()),
        &None,
        &None,
        &None,
    );
}

#[test]
fn update_configs_all_bps_values_should_work() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

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
        stake_manager,
        stake_owner,
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
        }
    );

    // we update all the bps values to be %10
    pool.update_config(
        &None,
        &Some(1000i64),
        &Some(admin2.clone()),
        &Some(1000i64),
        &Some(1000i64),
        &Some(1000i64),
    );

    // assert the changes
    assert_eq!(
        pool.query_config(),
        Config {
            token_a: token1.address.clone(),
            token_b: token2.address.clone(),
            share_token: share_token_address.clone(),
            stake_contract: stake_token_address.clone(),
            pool_type: PairType::Xyk,
            total_fee_bps: 1000,
            fee_recipient: admin2.clone(),
            max_allowed_slippage_bps: 1000,
            max_allowed_spread_bps: 1000,
            max_referral_bps: 1000,
        }
    );
}

#[should_panic(expected = "Pool: Initialize: swap fee is higher than the maximum allowed!")]
#[test]
fn test_initialize_with_maximum_allowed_swap_fee_bps_over_the_cap_should_fail() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token1.address < token2.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let fee_recipient = Address::generate(&env);

    let token_init_info = TokenInitInfo {
        token_a: token1.address,
        token_b: token2.address,
    };
    let stake_init_info = StakeInitInfo {
        min_bond: 10i128,
        min_reward: 5i128,
        manager: Address::generate(&env),
        max_complexity: 10u32,
    };
    let stake_wasm_hash = install_stake_wasm(&env);
    let token_wasm_hash = install_token_wasm(&env);

    let lp_init_info = LiquidityPoolInitInfo {
        admin,
        swap_fee_bps: 1_501, // we are just slightly over the cap of `1_500`, this will error
        fee_recipient,
        max_allowed_slippage_bps: 5_000,
        max_allowed_spread_bps: 1_000,
        max_referral_bps: 5_000,
        default_slippage_bps: 1_000,
        token_init_info,
        stake_init_info,
    };

    let _ = LiquidityPoolClient::new(
        &env,
        &env.register(
            LiquidityPool,
            (
                &stake_wasm_hash,
                &token_wasm_hash,
                lp_init_info,
                &Address::generate(&env),
                String::from_str(&env, "Pool"),
                String::from_str(&env, "PHOBTC"),
                &100i64,
                &1_000i64,
            ),
        ),
    );
}
