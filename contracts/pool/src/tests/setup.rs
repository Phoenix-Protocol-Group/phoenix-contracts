use soroban_sdk::{
    testutils::{arbitrary::std, Address as _},
    Address, BytesN, Env, String,
};

use crate::{
    contract::{LiquidityPool, LiquidityPoolClient},
    token_contract,
};

use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(
        env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    )
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "upgrade")]
pub mod old_liquidity_pool {
    soroban_sdk::contractimport!(file = "../../.artifacts_sdk_update/old_phoenix_pool.wasm");
}

#[allow(clippy::too_many_arguments)]
pub mod latest_liquidity_pool {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_pool.wasm"
    );
}

#[cfg(feature = "upgrade")]
pub fn install_old_token_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../.artifacts_sdk_update/old_soroban_token_contract.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

#[cfg(feature = "upgrade")]
pub fn install_old_stake_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(file = "../../.artifacts_sdk_update/old_phoenix_stake.wasm");
    env.deployer().upload_contract_wasm(WASM)
}

pub fn install_token_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

#[allow(clippy::too_many_arguments)]
pub fn install_stake_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "upgrade")]
pub fn install_new_lp_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_pool.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

#[allow(clippy::too_many_arguments)]
pub fn deploy_liquidity_pool_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
    token_a_b: (&Address, &Address),
    swap_fees: i64,
    fee_recipient: impl Into<Option<Address>>,
    max_allowed_slippage_bps: impl Into<Option<i64>>,
    max_allowed_spread_bps: impl Into<Option<i64>>,
    stake_manager: Address,
    stake_owner: Address,
) -> LiquidityPoolClient<'a> {
    let admin = admin.into().unwrap_or(Address::generate(env));

    let fee_recipient = fee_recipient
        .into()
        .unwrap_or_else(|| Address::generate(env));

    let token_init_info = TokenInitInfo {
        token_a: token_a_b.0.clone(),
        token_b: token_a_b.1.clone(),
    };
    let stake_init_info = StakeInitInfo {
        min_bond: 10i128,
        min_reward: 5i128,
        manager: stake_manager,
        max_complexity: 10u32,
    };
    let stake_wasm_hash = install_stake_wasm(env);
    let token_wasm_hash = install_token_wasm(env);

    let lp_init_info = LiquidityPoolInitInfo {
        admin,
        swap_fee_bps: swap_fees,
        fee_recipient,
        max_allowed_slippage_bps: max_allowed_slippage_bps.into().unwrap_or(5_000),
        default_slippage_bps: 2_500,
        max_allowed_spread_bps: max_allowed_spread_bps.into().unwrap_or(1_000),
        max_referral_bps: 5_000,
        token_init_info,
        stake_init_info,
    };

    let pool = LiquidityPoolClient::new(
        env,
        &env.register(
            LiquidityPool,
            (
                &stake_wasm_hash,
                &token_wasm_hash,
                lp_init_info,
                &stake_owner,
                String::from_str(env, "Pool"),
                String::from_str(env, "PHOBTC"),
                &100i64,
                &1_000i64,
            ),
        ),
    );

    pool
}

#[test]
#[allow(deprecated)]
#[cfg(feature = "upgrade")]
fn update_liquidity_pool() {
    use soroban_sdk::testutils::Ledger;

    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let mut admin1 = Address::generate(&env);
    let mut admin2 = Address::generate(&env);
    let user1 = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin1);
    let mut token2 = deploy_token_contract(&env, &admin2);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut admin1, &mut admin2);
    }

    let old_lp_addr = env.register_contract_wasm(None, old_liquidity_pool::WASM);
    let old_lp_client = old_liquidity_pool::Client::new(&env, &old_lp_addr);

    let token_init_info = old_liquidity_pool::TokenInitInfo {
        token_a: token1.address.clone(),
        token_b: token2.address.clone(),
    };
    let stake_init_info = old_liquidity_pool::StakeInitInfo {
        min_bond: 10i128,
        min_reward: 5i128,
        manager: Address::generate(&env),
        max_complexity: 10u32,
    };
    let stake_wasm_hash = install_old_stake_wasm(&env);
    let token_wasm_hash = install_old_token_wasm(&env);

    let lp_init_info = old_liquidity_pool::LiquidityPoolInitInfo {
        admin: admin1.clone(),
        swap_fee_bps: 0i64,
        fee_recipient: admin1.clone(),
        max_allowed_slippage_bps: 5_000i64,
        default_slippage_bps: 2_500i64,
        max_allowed_spread_bps: 1_000i64,
        max_referral_bps: 5_000i64,
        token_init_info,
        stake_init_info,
    };

    soroban_sdk::testutils::arbitrary::std::dbg!("DBG");

    old_lp_client.initialize(
        &stake_wasm_hash,
        &token_wasm_hash,
        &lp_init_info,
        &Address::generate(&env),
        &7u32,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "PHOBTC"),
        &100i64,
        &1_000i64,
    );

    soroban_sdk::testutils::arbitrary::std::dbg!("DBG");
    assert_eq!(old_lp_client.query_config().fee_recipient, admin1);

    env.ledger().with_mut(|li| li.timestamp = 100);

    token1.mint(&user1, &1_000_000_000_000_000);
    token2.mint(&user1, &1_000_000_000_000_000);

    old_lp_client.provide_liquidity(
        &user1,
        &Some(1_000_000_000_000_000),
        &Some(1_000_000_000_000_000),
        &Some(1_000_000_000_000_000),
        &Some(1_000_000_000_000_000),
        &None,
        &None,
    );

    let new_lp_wasm = install_new_lp_wasm(&env);
    old_lp_client.update(&new_lp_wasm);

    let new_lp_client = latest_liquidity_pool::Client::new(&env, &old_lp_addr);

    assert_eq!(new_lp_client.query_config().fee_recipient, admin1);

    env.ledger().with_mut(|li| li.timestamp = 10_000);

    new_lp_client.withdraw_liquidity(
        &user1,
        &500_000_000_000_000,
        &500_000_000_000_000,
        &500_000_000_000_000,
        &None,
        &None,
    );

    let pool_info_after_upgrade = new_lp_client.query_pool_info_for_factory();
    assert_eq!(
        pool_info_after_upgrade.pool_response,
        latest_liquidity_pool::PoolResponse {
            asset_a: latest_liquidity_pool::Asset {
                address: token1.address,
                amount: 500000000000000
            },
            asset_b: latest_liquidity_pool::Asset {
                address: token2.address,
                amount: 500000000000000,
            },
            asset_lp_share: latest_liquidity_pool::Asset {
                address: new_lp_client.query_share_token_address(),
                amount: 500000000000000
            },
            stake_address: new_lp_client.query_stake_contract_address(),
        }
    );
}
