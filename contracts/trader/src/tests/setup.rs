use soroban_sdk::{
    testutils::{arbitrary::std, Address as _},
    Address, BytesN, Env, String,
};

use crate::{
    contract::{Trader, TraderClient},
    lp_contract::{self, LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo},
    token_contract,
};

const TOKEN_WASM: &[u8] =
    include_bytes!("../../../../target/wasm32-unknown-unknown/release/soroban_token_contract.wasm");

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

pub fn deploy_token_contract<'a>(
    env: &Env,
    admin: &Address,
    decimal: &u32,
    name: &String,
    symbol: &String,
) -> token_contract::Client<'a> {
    let token_addr = env.register(TOKEN_WASM, (admin, *decimal, name.clone(), symbol.clone()));
    let token_client = token_contract::Client::new(env, &token_addr);

    token_client
}

pub fn deploy_and_init_lp_client(
    env: &Env,
    admin: Address,
    token_a: Address,
    token_a_amount: i128,
    token_b: Address,
    token_b_amount: i128,
    swap_fee_bps: i64,
) -> lp_contract::Client {
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
        default_slippage_bps: 2_500,
        max_allowed_spread_bps: 5000,
        swap_fee_bps,
        max_referral_bps: 5_000,
        token_init_info,
        stake_init_info,
    };

    let lp_client = lp_contract::Client::new(
        env,
        &env.register(
            lp_contract::WASM,
            (
                &stake_wasm_hash,
                &token_wasm_hash,
                lp_init_info,
                &Address::generate(env),
                String::from_str(env, "staked Phoenix"),
                String::from_str(env, "sPHO"),
                &100i64,
                &1_000i64,
            ),
        ),
    );

    lp_client.provide_liquidity(
        &admin.clone(),
        &Some(token_a_amount),
        &None::<i128>,
        &Some(token_b_amount),
        &None::<i128>,
        &None::<i64>,
        &None,
    );
    lp_client
}

pub fn deploy_trader_client<'a>(
    env: &Env,
    admin: &Address,
    contract_name: String,
    token_tuple: &(Address, Address),
    pho_token: &Address,
) -> TraderClient<'a> {
    let trader_client = TraderClient::new(
        env,
        &env.register(Trader, (admin, contract_name, token_tuple, pho_token)),
    );

    trader_client
}
