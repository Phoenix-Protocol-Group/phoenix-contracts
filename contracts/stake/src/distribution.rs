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

    // Load the reward and total staked histories
    let reward_history = get_reward_history(env, reward_token);
    let total_staked_history = get_total_staked_history(env);

    // Filter reward history to the relevant range
    let mut relevant_reward_history = Map::new(env);
    for (day, reward) in reward_history.iter() {
        if day >= last_reward_day && day <= current_timestamp {
            relevant_reward_history.set(day, reward);
        }
    }

    let mut pending_rewards: i128 = 0;

    // Iterate over each stake
    for stake in user_info.stakes.iter() {
        let stake_start_day = stake.stake_timestamp / SECONDS_PER_DAY;

        // Iterate over the relevant reward days
        for (day, daily_reward) in relevant_reward_history.iter() {
            if day < stake_start_day || day > current_timestamp {
                continue;
            }

            // Find the total staked amount for the given day
            if let Some(total_staked) = total_staked_history.get(day) {
                if total_staked > 0 {
                    // Calculate stake age in days
                    let stake_age_days = (day - stake_start_day).min(60);

                    // Correct multiplier logic
                    let multiplier = if stake_age_days >= 60 {
                        Decimal::one()
                    } else {
                        Decimal::from_ratio(stake_age_days, 60)
                    };

                    // Calculate the user's share of the reward
                    let user_share = (stake.stake as u128) * daily_reward / total_staked;

                    // Apply multiplier and accumulate rewards
                    pending_rewards += (user_share as i128) * multiplier;
                }
            }
        }
    }

    pending_rewards
}
