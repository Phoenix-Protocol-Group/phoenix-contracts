use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{
    contract::{Staking, StakingClient},
    token_contract,
};

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(env, &env.register_stellar_asset_contract(admin.clone()))
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
    let staking = StakingClient::new(env, &env.register_contract(None, Staking {}));

    staking.initialize(
        &admin,
        lp_token,
        &MIN_BOND,
        &MIN_REWARD,
        manager,
        owner,
        &max_complexity,
    );
    staking
}
