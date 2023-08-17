use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

use crate::{
    contract::{LiquidityPool, LiquidityPoolClient},
    token_contract,
};

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(env, &env.register_stellar_asset_contract(admin.clone()))
}

fn install_token_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

fn install_stake_wasm(env: &Env) -> BytesN<32> {
    // todo remove comment
    // Err is: the trait bound `(u64, u128): SorobanArbitrary` is not satisfied [E0277] Help: the trait `SorobanArbitrary` is implemented for `()` Note: required for `soroban_sdk::Vec<(u64, u128)>` to implement `SorobanArbitrary` Note: required because it appears within the type `ArbitraryPiecewiseLinear`
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

#[allow(clippy::too_many_arguments)]
pub fn deploy_liquidity_pool_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
    token_a: &Address,
    token_b: &Address,
    swap_fees: i64,
    fee_recipient: impl Into<Option<Address>>,
    max_allowed_slippage_bps: impl Into<Option<i64>>,
    max_allowed_spread_bps: impl Into<Option<i64>>,
) -> LiquidityPoolClient<'a> {
    let admin = admin.into().unwrap_or(Address::random(env));
    let pool = LiquidityPoolClient::new(env, &env.register_contract(None, LiquidityPool {}));
    let token_wasm_hash = install_token_wasm(env);
    let stake_wasm_hash = install_stake_wasm(env);
    let fee_recipient = fee_recipient.into().unwrap_or_else(|| Address::random(env));
    let max_allowed_slippage = max_allowed_slippage_bps.into().unwrap_or(5_000); // 50% if not specified
    let max_allowed_spread = max_allowed_spread_bps.into().unwrap_or(500); // 5% if not specified
    let share_token_decimals = 7u32;
    pool.initialize(
        &admin,
        &token_wasm_hash,
        &stake_wasm_hash,
        token_a,
        token_b,
        &share_token_decimals,
        &swap_fees,
        &fee_recipient,
        &max_allowed_slippage,
        &max_allowed_spread,
    );
    pool
}
