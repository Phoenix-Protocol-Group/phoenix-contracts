use soroban_sdk::{Address, Env};

use crate::{
    contract::{StakingRewards, StakingRewardsClient},
    token_contract,
};

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(
        env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    )
}

const MIN_BOND: i128 = 1000;
const MIN_REWARD: i128 = 1000;
const MAX_COMPLEXITY: u32 = 10;

pub fn deploy_staking_rewards_contract<'a>(
    env: &Env,
    admin: &Address,
    reward_token: &Address,
    staking_contract: &Address,
) -> StakingRewardsClient<'a> {
    let staking_rewards = StakingRewardsClient::new(
        env,
        &env.register(
            StakingRewards,
            (
                admin,
                staking_contract,
                reward_token,
                &MAX_COMPLEXITY,
                &MIN_REWARD,
                &MIN_BOND,
            ),
        ),
    );

    staking_rewards
}
