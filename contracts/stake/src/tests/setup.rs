use soroban_sdk::{testutils::Address as _, xdr::ToXdr, Address, Bytes, BytesN, Env};

use crate::{
    contract::{Staking, StakingClient},
    token_contract,
};

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(
        env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn install_stake_rewards_contract(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake_rewards.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

mod stake_mainnet {
    soroban_sdk::contractimport!(file = "../../artifacts/phoenix_stake.wasm");
}

fn install_stake_mainnet_wasm(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(stake_mainnet::WASM)
}

fn install_current_stake_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

const MIN_BOND: i128 = 1000;
const MIN_REWARD: i128 = 1000;
pub const ONE_WEEK: u64 = 604800;
pub const ONE_DAY: u64 = 86400;
pub const SIXTY_DAYS: u64 = 60 * ONE_DAY;

pub fn deploy_staking_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
    lp_token: &Address,
    manager: &Address,
    owner: &Address,
    max_complexity: &u32,
) -> StakingClient<'a> {
    let admin = admin.into().unwrap_or(Address::generate(env));
    let staking = StakingClient::new(env, &env.register_contract(None, Staking {}));
    let _stake_rewards_hash = install_stake_rewards_contract(env);

    staking.initialize(
        &admin,
        lp_token,
        &MIN_BOND,
        &MIN_REWARD,
        manager,
        owner,
        max_complexity,
    );
    staking
}

#[test]
fn upgrade_stake_contract() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let admin = Address::generate(&env);

    let mut salt = Bytes::new(&env);
    salt.append(&admin.clone().to_xdr(&env));

    let salt = env.crypto().sha256(&salt);

    let mainnet_stake_wasm = install_stake_mainnet_wasm(&env);
    let addr = env
        .deployer()
        .with_address(admin.clone(), salt)
        .deploy(mainnet_stake_wasm);

    let stake_mainnet_client = stake_mainnet::Client::new(&env, &addr);

    let lp_token_addr = Address::generate(&env);
    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    stake_mainnet_client.initialize(
        &admin,
        &lp_token_addr,
        &MIN_BOND,
        &MIN_REWARD,
        &manager,
        &owner,
        &10,
    );

    let new_stake_wasm = install_current_stake_wasm(&env);
    stake_mainnet_client.update(&new_stake_wasm);
}
