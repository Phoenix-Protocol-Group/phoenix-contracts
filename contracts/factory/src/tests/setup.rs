use crate::{
    contract::{Factory, FactoryClient},
    token_contract,
};
use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};
use soroban_sdk::{
    testutils::{arbitrary::std, Address as _},
    vec, Address, BytesN, Env, String,
};
pub const ONE_DAY: u64 = 86400;
const TOKEN_WASM: &[u8] =
    include_bytes!("../../../../target/wasm32-unknown-unknown/release/soroban_token_contract.wasm");

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "upgrade")]
pub mod old_factory {
    soroban_sdk::contractimport!(file = "../../.artifacts_sdk_update/old_phoenix_factory.wasm");
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "upgrade")]
pub fn old_lp_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(file = "../../.artifacts_sdk_update/old_phoenix_pool.wasm");
    env.deployer().upload_contract_wasm(WASM)
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "upgrade")]
pub fn old_stake_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(file = "../../.artifacts_sdk_update/old_phoenix_stake.wasm");
    env.deployer().upload_contract_wasm(WASM)
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "upgrade")]
pub fn install_latest_factory(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_factory.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

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
    admin: Address,
    decimal: u32,
    name: String,
    symbol: String,
) -> token_contract::Client<'a> {
    let token_addr = env.register(TOKEN_WASM, (admin, decimal, name, symbol));
    let token_client = token_contract::Client::new(env, &token_addr);

    token_client
}

#[test]
#[allow(deprecated)]
#[cfg(feature = "upgrade")]
fn update_factory() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();
    let admin = Address::generate(&env);

    let factory_addr = env.register_contract_wasm(None, old_factory::WASM);
    let old_factory_client = old_factory::Client::new(&env, &factory_addr);

    old_factory_client.initialize(
        &admin.clone(),
        &install_multihop_wasm(&env),
        &old_lp_wasm(&env),
        &install_stable_lp(&env),
        &old_stake_wasm(&env),
        &install_token_wasm(&env),
        &vec![
            &env,
            admin.clone(),
            Address::generate(&env),
            Address::generate(&env),
        ],
        &7u32,
    );

    assert_eq!(old_factory_client.get_admin(), admin.clone());

    let latest_factory_wasm = install_latest_factory(&env);
    let stable_wasm = install_stable_lp(&env);

    old_factory_client.update(&latest_factory_wasm, &stable_wasm);

    let latest_factory_client = FactoryClient::new(&env, &factory_addr);

    assert_eq!(latest_factory_client.get_admin(), admin.clone());

    latest_factory_client.update_wasm_hashes(
        &Some(install_lp_contract(&env)),
        &Some(install_stake_wasm(&env)),
        &Some(install_token_wasm(&env)),
    );
}
