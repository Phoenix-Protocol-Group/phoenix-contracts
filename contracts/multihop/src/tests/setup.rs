use crate::contract::{Multihop, MultihopClient};
use crate::factory_contract::{LiquidityPoolInitInfo, PoolType, StakeInitInfo, TokenInitInfo};
use crate::{factory_contract, stable_pool, token_contract, xyk_pool};

use soroban_sdk::{
    testutils::{arbitrary::std, Address as _},
    Address, Bytes, BytesN, Env,
};
use soroban_sdk::{vec, String};
pub fn create_token_contract_with_metadata<'a>(
    env: &Env,
    admin: &Address,
    decimals: u32,
    name: String,
    symbol: String,
    amount: i128,
) -> token_contract::Client<'a> {
    let token = token_contract::Client::new(
        env,
        &env.register(token_contract::WASM, (admin, decimals, name, symbol)),
    );
    token.mint(admin, &amount);
    token
}

pub fn install_lp_contract(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(xyk_pool::WASM)
}

pub fn install_stable_lp_contract(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(stable_pool::WASM)
}

pub fn install_token_wasm(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(token_contract::WASM)
}

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(
        env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn install_stake_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

#[allow(clippy::too_many_arguments)]
pub fn install_multihop_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_multihop.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

pub fn deploy_factory_contract(e: &Env, admin: Address) -> Address {
    let factory_wasm = e.deployer().upload_contract_wasm(factory_contract::WASM);
    let salt = Bytes::new(e);
    let salt = e.crypto().sha256(&salt);

    e.deployer()
        .with_address(admin, salt)
        .deploy_v2(factory_wasm, ())
}

pub fn deploy_multihop_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
    factory: &Address,
) -> MultihopClient<'a> {
    let admin = admin.into().unwrap_or(Address::generate(env));

    let multihop = MultihopClient::new(env, &env.register(Multihop, ()));

    multihop.initialize(&admin, factory);
    multihop
}

pub fn deploy_and_mint_tokens<'a>(
    env: &'a Env,
    admin: &'a Address,
    amount: i128,
) -> token_contract::Client<'a> {
    let token = deploy_token_contract(env, admin);
    token.mint(admin, &amount);
    token
}

pub fn deploy_and_initialize_factory(env: &Env, admin: Address) -> factory_contract::Client {
    let factory_addr = deploy_factory_contract(env, admin.clone());
    let factory_client = factory_contract::Client::new(env, &factory_addr);
    let multihop_wasm_hash = install_multihop_wasm(env);
    let whitelisted_accounts = vec![env, admin.clone()];

    let lp_wasm_hash = install_lp_contract(env);
    let stable_wasm_hash = install_stable_lp_contract(env);
    let stake_wasm_hash = install_stake_wasm(env);
    let token_wasm_hash = install_token_wasm(env);

    factory_client.initialize(
        &admin.clone(),
        &multihop_wasm_hash,
        &lp_wasm_hash,
        &stable_wasm_hash,
        &stake_wasm_hash,
        &token_wasm_hash,
        &whitelisted_accounts,
        &10u32,
    );
    factory_client
}

#[allow(clippy::too_many_arguments)]
pub fn deploy_and_initialize_pool(
    env: &Env,
    factory: &factory_contract::Client,
    admin: Address,
    mut token_a: Address,
    mut token_a_amount: i128,
    mut token_b: Address,
    mut token_b_amount: i128,
    fees: Option<i64>,
    pool_type: PoolType,
) {
    if token_b < token_a {
        std::mem::swap(&mut token_a, &mut token_b);
        std::mem::swap(&mut token_a_amount, &mut token_b_amount);
    }

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
        max_allowed_spread_bps: 500,
        swap_fee_bps: fees.unwrap_or(0i64),
        max_referral_bps: 5_000,
        token_init_info,
        stake_init_info,
    };

    let amp = match pool_type {
        PoolType::Stable => Some(10u64),
        PoolType::Xyk => None,
    };

    let lp = factory.create_liquidity_pool(
        &admin.clone(),
        &lp_init_info,
        &String::from_str(env, "Pool"),
        &String::from_str(env, "PHO/XLM"),
        &pool_type,
        &amp,
        &100i64,
        &1_000,
    );

    match pool_type {
        PoolType::Xyk => {
            let lp_client = xyk_pool::Client::new(env, &lp);
            lp_client.provide_liquidity(
                &admin.clone(),
                &Some(token_a_amount),
                &None,
                &Some(token_b_amount),
                &None,
                &None::<i64>,
                &None::<u64>,
                &false,
            );
        }
        PoolType::Stable => {
            let lp_client = stable_pool::Client::new(env, &lp);
            lp_client.provide_liquidity(
                &admin.clone(),
                &token_a_amount,
                &token_b_amount,
                &None,
                &None::<u64>,
                &None::<u128>,
                &false,
            );
        }
    }
}
