use soroban_sdk::{
    testutils::{arbitrary::std, Address as _},
    xdr::ToXdr,
    Address, Bytes, BytesN, Env, FromVal, String,
};

use crate::{
    contract::{Trader, TraderClient},
    lp_contract::{self, LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo},
    token_contract,
};

pub fn install_token_wasm(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(token_contract::WASM)
}

#[allow(clippy::too_many_arguments)]
pub fn install_stake_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

pub fn deploy_lp_wasm(env: &Env, admin: Address) -> Address {
    let factory_wasm = env.deployer().upload_contract_wasm(lp_contract::WASM);
    let salt = Bytes::new(env);
    let salt = env.crypto().sha256(&salt);

    env.deployer()
        .with_address(admin, salt)
        .deploy(factory_wasm)
}

pub fn deploy_token_contract<'a>(
    env: &Env,
    admin: &Address,
    decimal: &u32,
    name: &String,
    symbol: &String,
) -> token_contract::Client<'a> {
    let token_wasm = install_token_wasm(env);

    let mut salt = Bytes::new(env);
    salt.append(&name.clone().to_xdr(&env));
    let salt = env.crypto().sha256(&salt);
    let token_addr = env
        .deployer()
        .with_address(admin.clone(), salt)
        .deploy(token_wasm);

    let token_client = token_contract::Client::new(env, &token_addr);

    token_client.initialize(admin, decimal, name, symbol);

    token_client
}

pub fn deploy_and_init_lp_client(
    env: &Env,
    admin: Address,
    token_a: Address,
    token_a_amount: i128,
    token_b: Address,
    token_b_amount: i128,
) -> lp_contract::Client {
    let lp_addr = deploy_lp_wasm(env, admin.clone());

    let lp_client = lp_contract::Client::new(env, &lp_addr);

    let stake_wasm_hash = install_stake_wasm(env);
    let token_wasm_hash = install_token_wasm(env);

    let token_init_info = TokenInitInfo {
        token_a: token_a.clone(),
        token_b: token_b.clone(),
    };
    let stake_init_info = StakeInitInfo {
        min_bond: 10i128,
        min_reward: 5i128,
        manager: Address::generate(env),
        max_complexity: 10u32,
    };

    let lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        max_allowed_slippage_bps: 5000,
        max_allowed_spread_bps: 5000,
        swap_fee_bps: 0,
        max_referral_bps: 5_000,
        token_init_info,
        stake_init_info,
    };

    lp_client.initialize(
        &stake_wasm_hash,
        &token_wasm_hash,
        &lp_init_info,
        &Address::generate(env),
        &6,
        &String::from_str(env, "staked Phoenix"),
        &String::from_str(env, "sPHO"),
    );

    lp_client.provide_liquidity(
        &admin.clone(),
        &Some(token_a_amount),
        &None::<i128>,
        &Some(token_b_amount),
        &None::<i128>,
        &None::<i64>,
    );
    lp_client
}

pub fn deploy_trader_client(env: &Env) -> TraderClient {
    TraderClient::new(env, &env.register_contract(None, Trader))
}
