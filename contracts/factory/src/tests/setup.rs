extern crate std;

use crate::{
    contract::{Factory, FactoryClient},
    token_contract,
};
use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};
use soroban_sdk::{testutils::Address as _, vec, Address, BytesN, Env, String, Vec};
pub const ONE_DAY: u64 = 86400;
const TOKEN_WASM: &[u8] =
    include_bytes!("../../../../target/wasm32-unknown-unknown/release/soroban_token_contract.wasm");

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "upgrade")]
pub mod old_factory {
    soroban_sdk::contractimport!(file = "../../.wasm_binaries_mainnet/live_factory.wasm");
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "upgrade")]
pub fn install_old_multihop_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(file = "../../.wasm_binaries_mainnet/live_multihop.wasm");
    env.deployer().upload_contract_wasm(WASM)
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "upgrade")]
pub fn install_old_token_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(file = "../../.wasm_binaries_mainnet/live_token_contract.wasm");
    env.deployer().upload_contract_wasm(WASM)
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "upgrade")]
pub fn old_lp_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(file = "../../.wasm_binaries_mainnet/live_pho_usdc_pool.wasm");
    env.deployer().upload_contract_wasm(WASM)
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "upgrade")]
pub fn old_stake_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(file = "../../.wasm_binaries_mainnet/live_pho_usdc_stake.wasm");
    env.deployer().upload_contract_wasm(WASM)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn install_latest_factory(env: &Env) -> BytesN<32> {
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

    let multihop_wasm_hash = install_multihop_wasm(env);
    let whitelisted_accounts: Vec<Address> = vec![env, admin.clone()];

    let lp_wasm_hash = install_lp_contract(env);
    let stable_wasm_hash = install_stable_lp(env);
    let stake_wasm_hash = install_stake_wasm(env);
    let token_wasm_hash = install_token_wasm(env);

    let factory = FactoryClient::new(
        env,
        &env.register(
            Factory,
            (
                &admin,
                &multihop_wasm_hash,
                &lp_wasm_hash,
                &stable_wasm_hash,
                &stake_wasm_hash,
                &token_wasm_hash,
                whitelisted_accounts,
                &10u32,
            ),
        ),
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
    use phoenix::utils::PoolType;

    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();
    let admin = Address::generate(&env);

    let factory_addr = env.register_contract_wasm(None, old_factory::WASM);
    let old_factory_client = old_factory::Client::new(&env, &factory_addr);

    old_factory_client.initialize(
        &admin.clone(),
        &install_old_multihop_wasm(&env),
        &old_lp_wasm(&env),
        &install_stable_lp(&env),
        &old_stake_wasm(&env),
        &install_old_token_wasm(&env),
        &vec![
            &env,
            admin.clone(),
            Address::generate(&env),
            Address::generate(&env),
        ],
        &7u32,
    );

    let mut token_a = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        9,
        String::from_str(&env, "Phoenix"),
        String::from_str(&env, "PHO"),
    );
    let mut token_b = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        13,
        String::from_str(&env, "Stellar"),
        String::from_str(&env, "XLM"),
    );

    if token_b.address < token_a.address {
        std::mem::swap(&mut token_a, &mut token_b);
    }

    let factory = deploy_factory_contract(&env, Some(admin.clone()));
    assert_eq!(factory.get_admin(), admin);

    let token_init_info = old_factory::TokenInitInfo {
        token_a: token_a.address.clone(),
        token_b: token_b.address.clone(),
    };

    let stake_init_info = old_factory::StakeInitInfo {
        min_bond: 10,
        min_reward: 10,
        manager: admin.clone(),
        max_complexity: 10u32,
    };

    let lp_init_info = old_factory::LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        max_allowed_slippage_bps: 5000,
        max_allowed_spread_bps: 500,
        default_slippage_bps: 2_500,
        swap_fee_bps: 0,
        max_referral_bps: 5000,
        token_init_info,
        stake_init_info,
    };

    let pool_addr = old_factory_client.create_liquidity_pool(
        &admin.clone(),
        &lp_init_info,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "PHO/BTC"),
        &old_factory::PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );

    assert_eq!(old_factory_client.get_admin(), admin.clone());

    let latest_factory_wasm = install_latest_factory(&env);

    old_factory_client.update(&latest_factory_wasm);

    let latest_factory_client = FactoryClient::new(&env, &factory_addr);

    assert_eq!(latest_factory_client.get_admin(), admin.clone());

    latest_factory_client.update_config(
        &None,
        &Some(install_lp_contract(&env)),
        &Some(install_stake_wasm(&env)),
        &Some(install_token_wasm(&env)),
        &None,
        &None,
        &None,
    );

    let _ = latest_factory_client.get_config();

    let pool_query = latest_factory_client.query_pool_details(&pool_addr);
    assert_eq!(pool_query.pool_response.asset_a.address, token_a.address);
    assert_eq!(pool_query.pool_response.asset_b.address, token_b.address);

    let mut token1 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        9,
        String::from_str(&env, "Bitcoin"),
        String::from_str(&env, "BTC"),
    );
    let mut token2 = install_and_deploy_token_contract(
        &env,
        admin.clone(),
        13,
        String::from_str(&env, "Stellar"),
        String::from_str(&env, "XLM"),
    );

    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let second_lp_init_info = generate_lp_init_info(
        token1.address.clone(),
        token2.address.clone(),
        Address::generate(&env),
        admin.clone(),
        Address::generate(&env),
    );

    let second_pool = latest_factory_client.create_liquidity_pool(
        &admin,
        &second_lp_init_info,
        &String::from_str(&env, "SecondPool"),
        &String::from_str(&env, "BTC/XLM"),
        &PoolType::Xyk,
        &None::<u64>,
        &100i64,
        &1_000,
    );

    let pool_query = latest_factory_client.query_pool_details(&second_pool);
    assert_eq!(pool_query.pool_response.asset_a.address, token1.address);
    assert_eq!(pool_query.pool_response.asset_b.address, token2.address);

    let new_admin = Address::generate(&env);
    latest_factory_client.propose_admin(&new_admin, &None);
    latest_factory_client.accept_admin();
    assert_eq!(latest_factory_client.get_admin(), new_admin);
}
