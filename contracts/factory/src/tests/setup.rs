use crate::{
    contract::{Factory, FactoryClient},
    token_contract,
};
use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};
use soroban_sdk::{testutils::Address as _, vec, Address, BytesN, Env};
#[allow(clippy::too_many_arguments)]
pub mod lp_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_pool.wasm"
    );
}

pub mod stake_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake.wasm"
    );
}

pub fn install_multihop_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_multihop.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

pub fn deploy_lp_contract<'a>(env: &Env, contract_id: Address) -> lp_contract::Client<'a> {
    lp_contract::Client::new(
        env,
        &env.register_contract_wasm(Some(&contract_id), lp_contract::WASM),
    )
}

pub fn deploy_stake_contract<'a>(
    env: &Env,
    stake_contract_address: Address,
) -> stake_contract::Client<'a> {
    stake_contract::Client::new(
        env,
        &env.register_contract_wasm(Some(&stake_contract_address), stake_contract::WASM),
    )
}

pub fn deploy_stake_token_client<'a>(
    env: &Env,
    token_contract_address: Address,
) -> token_contract::Client<'a> {
    token_contract::Client::new(
        env,
        &env.register_contract_wasm(Some(&token_contract_address), token_contract::WASM),
    )
}

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(env, &env.register_stellar_asset_contract(admin.clone()))
}

pub fn install_lp_contract(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(lp_contract::WASM)
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

pub fn deploy_factory_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
) -> FactoryClient<'a> {
    let admin = admin.into().unwrap_or(Address::generate(env));
    let factory = FactoryClient::new(env, &env.register_contract(None, Factory {}));
    let multihop_wasm_hash = install_multihop_wasm(env);
    let whitelisted_accounts = vec![env, admin.clone()];

    let lp_wasm_hash = install_lp_contract(env);
    let stake_wasm_hash = install_stake_wasm(env);
    let token_wasm_hash = install_token_wasm(env);

    factory.initialize(
        &admin,
        &multihop_wasm_hash,
        &lp_wasm_hash,
        &stake_wasm_hash,
        &token_wasm_hash,
        &whitelisted_accounts,
        &10u32,
    );
    factory
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_lp_init_info(
    token_a: &crate::token_contract::Client<'_>,
    token_b: &crate::token_contract::Client<'_>,
    manager: Address,
    admin: &Address,
    fee_recipient: Address,
    min_bond: i128,
    min_reward: i128,
    max_allowed_slippage_bps: i64,
    max_allowed_spread_bps: i64,
    swap_fee_bps: i64,
    max_referral_bps: i64,
) -> LiquidityPoolInitInfo {
    let token_init_info = TokenInitInfo {
        token_a: token_a.address.clone(),
        token_b: token_b.address.clone(),
    };
    let stake_init_info = StakeInitInfo {
        min_bond,
        min_reward,
        manager,
    };

    LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: fee_recipient.clone(),
        max_allowed_slippage_bps,
        max_allowed_spread_bps,
        swap_fee_bps,
        max_referral_bps,
        token_init_info,
        stake_init_info,
    }
}
