use soroban_decimal::Decimal;
use soroban_sdk::{contracttype, Address, Env, Map};

use crate::storage::BondingInfo;
use phoenix::ttl::{PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};

pub const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

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

    // Load reward history and total staked history
    let reward_history = get_reward_history(env, reward_token);
    let total_staked_history = get_total_staked_history(env);

    let reward_keys = reward_history.keys();
    let mut pending_rewards: i128 = 0;

    // Find the first relevant reward day after `last_reward_day`
    if let Some(first_relevant_day) = reward_keys.iter().find(|&day| day > last_reward_day) {
        // Iterate over all relevant reward days
        for reward_day in reward_keys
            .iter()
            .skip_while(|&day| day < first_relevant_day)
            .take_while(|&day| day <= current_timestamp)
        {
            // Get the daily reward and total staked for this day
            if let (Some(daily_reward), Some(total_staked)) = (
                reward_history.get(reward_day),
                total_staked_history.get(reward_day),
            ) {
                if total_staked == 0 {
                    continue; // Skip if nothing was staked
                }

                // Process each stake for the current reward day
                for stake in user_info.stakes.iter() {
                    // Calculate the age of the stake in days
                    let stake_age_days =
                        ((reward_day - stake.stake_timestamp) / SECONDS_PER_DAY).min(60);

                    // Determine the multiplier based on the stake age
                    let multiplier = Decimal::from_ratio(stake_age_days, 60);

                    // Calculate the user's share of the rewards
                    let user_share = (stake.stake as u128) * daily_reward / total_staked;

                    // Adjust the reward using the multiplier and add to the total
                    pending_rewards += (user_share as i128) * multiplier;
                }
            }
        }
    }

    pending_rewards
}

pub fn calculate_pending_rewards_chunked(
    env: &Env,
    reward_token: &Address,
    user_info: &BondingInfo,
    chunk_size: u32,
    start_day: Option<u64>,
) -> (i128, u64) {
    let current_timestamp = env.ledger().timestamp();
    let last_claim_time = user_info.last_reward_time;

    let reward_history = get_reward_history(env, reward_token);
    let total_staked_history = get_total_staked_history(env);

    let mut reward_keys = soroban_sdk::Vec::new(env);

    for day in reward_history.keys().into_iter() {
        if day > last_claim_time && day <= current_timestamp {
            reward_keys.push_back(day);
        }
    }

    let mut pending_rewards: i128 = 0;
    let mut last_reward_day = 0u64;

    for reward_day in reward_keys
        .into_iter()
        .skip_while(|&day| day < start_day.unwrap_or(last_claim_time))
        .take(chunk_size as usize)
    {
        if let (Some(daily_reward), Some(total_staked)) = (
            reward_history.get(reward_day),
            total_staked_history.get(reward_day),
        ) {
            if total_staked > 0 {
                for stake in user_info.stakes.iter() {
                    let stake_age_days =
                        ((reward_day - stake.stake_timestamp) / SECONDS_PER_DAY).min(60);
                    let multiplier = Decimal::from_ratio(stake_age_days, 60);

                    let user_share = (stake.stake as u128) * daily_reward / total_staked;
                    pending_rewards += (user_share as i128) * multiplier;
                }
            }
            last_reward_day = reward_day;
        }
    }

    (pending_rewards, last_reward_day)
}
