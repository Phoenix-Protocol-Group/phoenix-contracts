use crate::{
    contract::{Staking, StakingClient},
    token_contract,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

pub const ONE_WEEK: u64 = 604800;
pub const ONE_DAY: u64 = 86400;
pub const SIXTY_DAYS: u64 = 60 * ONE_DAY;

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(
        env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    )
}

#[allow(clippy::too_many_arguments)]
pub mod latest_stake {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake.wasm"
    );
}

#[allow(dead_code)]
fn install_stake_latest_wasm(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(latest_stake::WASM)
}

const MIN_BOND: i128 = 1000;
const MIN_REWARD: i128 = 1000;

pub fn deploy_staking_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
    lp_token: &Address,
    manager: &Address,
    owner: &Address,
    max_complexity: &u32,
) -> StakingClient<'a> {
    let admin = admin.into().unwrap_or(Address::generate(env));
    let staking = StakingClient::new(
        env,
        &env.register(
            Staking,
            (
                &admin,
                lp_token,
                &MIN_BOND,
                &MIN_REWARD,
                manager,
                owner,
                max_complexity,
            ),
        ),
    );

    staking
}
