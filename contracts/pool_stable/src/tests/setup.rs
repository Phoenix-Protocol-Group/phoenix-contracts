use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

use crate::{
    contract::{StableLiquidityPool, StableLiquidityPoolClient},
    token_contract,
};

use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(env, &env.register_stellar_asset_contract(admin.clone()))
}

pub fn install_token_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

pub fn install_stake_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

#[allow(clippy::too_many_arguments)]
pub fn deploy_stable_liquidity_pool_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
    token_a_b: (&Address, &Address),
    swap_fees: i64,
    fee_recipient: impl Into<Option<Address>>,
    max_allowed_slippage_bps: impl Into<Option<i64>>,
    max_allowed_spread_bps: impl Into<Option<i64>>,
) -> StableLiquidityPoolClient<'a> {
    let admin = admin.into().unwrap_or(Address::generate(env));
    let pool =
        StableLiquidityPoolClient::new(env, &env.register_contract(None, StableLiquidityPool {}));
    let fee_recipient = fee_recipient
        .into()
        .unwrap_or_else(|| Address::generate(env));

    let token_init_info = TokenInitInfo {
        token_a: token_a_b.0.clone(),
        token_b: token_a_b.1.clone(),
    };
    let stake_init_info = StakeInitInfo {
        min_bond: 10i128,
        max_distributions: 10u32,
        min_reward: 5i128,
    };

    let token_wasm_hash = install_token_wasm(env);
    let stake_wasm_hash = install_stake_wasm(env);

    let lp_init_info = LiquidityPoolInitInfo {
        admin,
        share_token_decimals: 7u32,
        swap_fee_bps: swap_fees,
        fee_recipient,
        max_allowed_slippage_bps: max_allowed_slippage_bps.into().unwrap_or(5_000),
        max_allowed_spread_bps: max_allowed_spread_bps.into().unwrap_or(500),
        max_referral_bps: 5_000,
        token_init_info,
        stake_init_info,
    };

    pool.initialize(
        &stake_wasm_hash,
        &token_wasm_hash,
        &lp_init_info,
    );
    pool
}
