use soroban_sdk::{Address, Env};

use crate::{
    contract::{StakingRewards, StakingRewardsClient},
    stake_contract, token_contract,
};

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(env, &env.register_stellar_asset_contract(admin.clone()))
}

fn deploy_stake_contract<'a>(env: &Env) -> stake_contract::Client<'a> {
    stake_contract::Client::new(env, &env.register_contract_wasm(None, stake_contract::WASM))
}

const MIN_BOND: i128 = 1000;
const MIN_REWARD: i128 = 1000;
const MAX_COMPLEXITY: u32 = 10;

pub fn deploy_staking_rewards_contract<'a>(
    env: &Env,
    admin: &Address,
    lp_token: &Address,
    reward_token: &Address,
) -> (stake_contract::Client<'a>, StakingRewardsClient<'a>) {
    let staking_rewards =
        StakingRewardsClient::new(env, &env.register_contract(None, StakingRewards {}));

    staking_rewards.initialize(admin, reward_token, &MAX_COMPLEXITY, &MIN_REWARD, &MIN_BOND);

    let staking = deploy_stake_contract(env);
    staking.initialize(
        admin,
        lp_token,
        &staking_rewards.address.clone(),
        &MIN_BOND,
        &MIN_REWARD,
        admin,
        admin,
        &MAX_COMPLEXITY,
    );
    (staking, staking_rewards)
}
