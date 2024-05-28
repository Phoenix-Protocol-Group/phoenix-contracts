use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{
    contract::{StakingRewards, StakingRewardsClient},
    stake_contract, token_contract,
};

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(env, &env.register_stellar_asset_contract(admin.clone()))
}

pub fn deploy_stake_contract<'a>(env: &Env, admin: &Address) -> stake_contract::Client<'a> {
    token_contract::Client::new(env, &env.register_stellar_asset_contract(admin.clone()))
}

const MIN_BOND: i128 = 1000;
const MIN_REWARD: i128 = 1000;
pub const ONE_WEEK: u64 = 604800;
pub const ONE_DAY: u64 = 86400;
const MAX_COMPLEXITY: u32 = 10;

pub fn deploy_staking_rewards_contract<'a>(
    env: &Env,
    admin: &Address,
    lp_token: &Address,
    reward_token: &Address,
    owner: &Address,
) -> (StakingClient<'a>, StakingRewardsClient<'a>) {
    let staking = deploy_stake_contract(env, admin);
    staking.initialize(
        &admin,
        lp_token,
        &MIN_BOND,
        &MIN_REWARD,
        admin,
        owner,
        MAX_COMPLEXITY,
    );

    let staking_rewards =
        StakingRewardsClient::new(env, &env.register_contract(None, StakingRewards {}));

    staking_rewards.initialize(&admin, staking.address, reward_token, owner);
    (staking, staking_rewards)
}
