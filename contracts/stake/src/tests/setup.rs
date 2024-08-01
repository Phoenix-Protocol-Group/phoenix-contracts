use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

use crate::{
    contract::{Staking, StakingClient},
    token_contract,
};

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(env, &env.register_stellar_asset_contract(admin.clone()))
}

#[allow(clippy::too_many_arguments)]
pub fn install_stake_rewards_contract(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake_rewards.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

const MIN_BOND: i128 = 1000;
const MIN_REWARD: i128 = 1000;
pub const ONE_WEEK: u64 = 604800;
pub const ONE_DAY: u64 = 86400;

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
    let stake_rewards_hash = install_stake_rewards_contract(env);

    staking.initialize(
        &admin,
        lp_token,
        &stake_rewards_hash,
        &MIN_BOND,
        &MIN_REWARD,
        manager,
        owner,
        max_complexity,
    );
    staking
}
