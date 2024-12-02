use crate::{
    contract::{Factory, FactoryClient},
    token_contract,
};
use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};
use soroban_sdk::{testutils::Address as _, vec, xdr::ToXdr, Address, Bytes, BytesN, Env, String};
pub const ONE_DAY: u64 = 86400;

#[allow(clippy::too_many_arguments)]
pub mod lp_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_pool.wasm"
    );
}

#[allow(clippy::too_many_arguments)]
pub mod stable_lp {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_pool_stable.wasm"
    );
}

#[allow(clippy::too_many_arguments)]
pub mod stake_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake.wasm"
    );
}

#[allow(clippy::too_many_arguments)]
pub fn install_multihop_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_multihop.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

pub fn install_lp_contract(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(lp_contract::WASM)
}

pub fn install_stable_lp(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(stable_lp::WASM)
}

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

pub fn deploy_factory_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
) -> FactoryClient<'a> {
    let admin = admin.into().unwrap_or(Address::generate(env));
    let factory = FactoryClient::new(env, &env.register(Factory, ()));
    let multihop_wasm_hash = install_multihop_wasm(env);
    let whitelisted_accounts = vec![env, admin.clone()];

    let lp_wasm_hash = install_lp_contract(env);
    let stable_wasm_hash = install_stable_lp(env);
    let stake_wasm_hash = install_stake_wasm(env);
    let token_wasm_hash = install_token_wasm(env);

    factory.initialize(
        &admin,
        &multihop_wasm_hash,
        &lp_wasm_hash,
        &stable_wasm_hash,
        &stake_wasm_hash,
        &token_wasm_hash,
        &whitelisted_accounts,
        &10u32,
    );

    factory
}

pub fn generate_lp_init_info(
    token_a: Address,
    token_b: Address,
    manager: Address,
    admin: Address,
    fee_recipient: Address,
) -> LiquidityPoolInitInfo {
    let token_init_info = TokenInitInfo { token_a, token_b };

    let stake_init_info = StakeInitInfo {
        min_bond: 10,
        min_reward: 10,
        manager,
        max_complexity: 10u32,
    };

    LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: fee_recipient.clone(),
        max_allowed_slippage_bps: 5000,
        max_allowed_spread_bps: 500,
        default_slippage_bps: 2_500,
        swap_fee_bps: 0,
        max_referral_bps: 5000,
        token_init_info,
        stake_init_info,
    }
}

pub fn install_and_deploy_token_contract<'a>(
    env: &Env,
    admin: &Address,
    decimal: &u32,
    name: &String,
    symbol: &String,
) -> token_contract::Client<'a> {
    let token_wasm = install_token_wasm(env);

    let mut salt = Bytes::new(env);
    salt.append(&name.clone().to_xdr(env));
    let salt = env.crypto().sha256(&salt);
    let token_addr = env
        .deployer()
        .with_address(admin.clone(), salt)
        .deploy_v2(token_wasm, ());

    let token_client = token_contract::Client::new(env, &token_addr);

    token_client.initialize(admin, decimal, name, symbol);

    token_client
}
