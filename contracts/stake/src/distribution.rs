use soroban_decimal::Decimal;
use soroban_sdk::{contracttype, Address, Env, Map};

use crate::storage::BondingInfo;
use phoenix::ttl::{PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};

const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

#[derive(Clone)]
#[contracttype]
pub enum DistributionDataKey {
    RewardHistory(Address),
    TotalStakedHistory,
}

pub fn save_reward_history(e: &Env, reward_token: &Address, reward_history: Map<u64, u128>) {
    e.storage().persistent().set(
        &DistributionDataKey::RewardHistory(reward_token.clone()),
        &reward_history,
    );
    e.storage().persistent().extend_ttl(
        &DistributionDataKey::RewardHistory(reward_token.clone()),
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn get_reward_history(e: &Env, reward_token: &Address) -> Map<u64, u128> {
    let reward_history = e
        .storage()
        .persistent()
        .get(&DistributionDataKey::RewardHistory(reward_token.clone()))
        .unwrap();
    e.storage().persistent().extend_ttl(
        &DistributionDataKey::RewardHistory(reward_token.clone()),
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );

    reward_history
}

pub fn save_total_staked_history(e: &Env, total_staked_history: Map<u64, u128>) {
    e.storage().persistent().set(
        &DistributionDataKey::TotalStakedHistory,
        &total_staked_history,
    );
    e.storage().persistent().extend_ttl(
        &DistributionDataKey::TotalStakedHistory,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn get_total_staked_history(e: &Env) -> Map<u64, u128> {
    let total_staked_history = e
        .storage()
        .persistent()
        .get(&DistributionDataKey::TotalStakedHistory)
        .unwrap();
    e.storage().persistent().extend_ttl(
        &DistributionDataKey::TotalStakedHistory,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );

    total_staked_history
}

pub fn calculate_pending_rewards(
    env: &Env,
    reward_token: &Address,
    user_info: &BondingInfo,
) -> i128 {
    let current_timestamp = env.ledger().timestamp();
    let last_reward_day = user_info.last_reward_time;

    // Load reward history and total staked history from storage
    let reward_history = get_reward_history(env, reward_token);
    let total_staked_history = get_total_staked_history(env);

    // Get the keys from the reward history map (which are the days)
    let reward_keys = reward_history.keys();

    let mut pending_rewards: i128 = 0;

    // Find the closest timestamp after last_reward_day
    if let Some(first_relevant_day) = reward_keys.iter().find(|&day| day > last_reward_day) {
        for staking_reward_day in reward_keys
            .iter()
            .skip_while(|&day| day < first_relevant_day)
            .take_while(|&day| day <= current_timestamp)
        {
            if let (Some(daily_reward), Some(total_staked)) = (
                reward_history.get(staking_reward_day),
                total_staked_history.get(staking_reward_day),
            ) {
                if total_staked > 0 {
                    // Calculate multiplier based on the age of each stake
                    for stake in user_info.stakes.iter() {
                        // Calculate the user's share of the total staked amount at the time
                        //TODO: safe math
                        let user_share = stake.stake as u128 * daily_reward / total_staked;
                        let stake_age_days = (staking_reward_day
                            .saturating_sub(stake.stake_timestamp))
                            / SECONDS_PER_DAY;
                        if stake_age_days == 0u64 {
                            continue;
                        }
                        let multiplier = if stake_age_days >= 60 {
                            Decimal::one()
                        } else {
                            Decimal::from_ratio(stake_age_days, 60)
                        };

                        // Apply the multiplier and accumulate the rewards
                        let adjusted_reward = user_share as i128 * multiplier;
                        //TODO: safe math
                        pending_rewards += adjusted_reward;
                    }
                }
            }
        }
    }

    pending_rewards
}
