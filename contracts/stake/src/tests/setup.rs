use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{
    contract::{Staking, StakingClient},
    token_contract,
};

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(env, &env.register_stellar_asset_contract(admin.clone()))
}

const MIN_BOND: i128 = 1000;
const MAX_DISTRIBUTIONS: u32 = 7;
const MIN_REWARD: i128 = 1000;

#[allow(clippy::too_many_arguments)]
pub fn deploy_staking_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
    lp_token: &Address,
) -> StakingClient<'a> {
    let admin = admin.into().unwrap_or(Address::random(env));
    let staking = StakingClient::new(env, &env.register_contract(None, Staking {}));

    staking.initialize(&admin, lp_token, &MIN_BOND, &MAX_DISTRIBUTIONS, &MIN_REWARD);
    staking
}
