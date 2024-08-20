use soroban_decimal::Decimal;
use soroban_sdk::{contracttype, Env};
use soroban_sdk::{log, panic_with_error, Address, ConversionError, Map, TryFromVal, Val, Vec};

use crate::storage::{BondingInfo, Config};
use phoenix::ttl::{
    BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT,
    PERSISTENT_LIFETIME_THRESHOLD,
};

const SECONDS_PER_DAY: u64 = 24 * 60 * 60;
const SECONDS_PER_YEAR: u64 = 365 * SECONDS_PER_DAY;

pub fn calc_power(
    config: &Config,
    stakes: i128,
    multiplier: Decimal,
    token_per_power: i32,
) -> i128 {
    if stakes < config.min_bond {
        0
    } else {
        stakes * multiplier / token_per_power as i128
    }
}

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
    e.storage()
        .instance()
        .extend_ttl(PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

pub fn get_reward_history(e: &Env, reward_token: &Address) -> Map<u64, u128> {
    let reward_history = e
        .storage()
        .instance()
        .get(&DistributionDataKey::RewardHistory(reward_token.clone()))
        .unwrap();
    e.storage()
        .instance()
        .extend_ttl(PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);

    reward_history
}

pub fn save_total_staked_history(e: &Env, total_staked_history: Map<u64, u128>) {
    e.storage().persistent().set(
        &DistributionDataKey::TotalStakedHistory,
        &total_staked_history,
    );
    e.storage()
        .instance()
        .extend_ttl(PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

pub fn get_total_staked_history(e: &Env) -> Map<u64, u128> {
    let total_staked_history = e
        .storage()
        .instance()
        .get(&DistributionDataKey::TotalStakedHistory)
        .unwrap();
    e.storage()
        .instance()
        .extend_ttl(PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);

    total_staked_history
}

fn find_closest_day(keys: &Vec<u64>, target_day: u64) -> Option<u64> {
    match keys.binary_search(&target_day) {
        Ok(index) => keys.get(index), // Exact match
        Err(index) => {
            if index == 0 {
                None // No smaller key exists
            } else {
                keys.get(index - 1) // Closest smaller key
            }
        }
    }
}

pub fn calculate_pending_rewards(
    env: &Env,
    reward_token: &Address,
    user_info: &BondingInfo,
) -> i128 {
    let current_time = env.ledger().timestamp();
    let last_reward_day = user_info.last_reward_time / SECONDS_PER_DAY;
    let current_day = current_time / SECONDS_PER_DAY;

    // Load reward history and total staked history from storage
    let reward_history = get_reward_history(env, reward_token);
    let total_staked_history = get_total_staked_history(env);

    // Get the keys from the reward history map (which are the days)
    let reward_keys = reward_history.keys();

    let mut pending_rewards: i128 = 0;

    if let Some(start_day) = find_closest_day(&reward_keys, current_day) {
        for day in last_reward_day..=start_day {
            if let (Some(daily_reward), Some(total_staked)) =
                (reward_history.get(day), total_staked_history.get(day))
            {
                if total_staked > 0 {
                    // Calculate the user's share of the total staked amount
                    let user_share = user_info.total_stake as u128 * daily_reward / total_staked;

                    // Calculate multiplier based on the age of each stake
                    for stake in user_info.stakes.iter() {
                        let stake_age_days =
                            (day * SECONDS_PER_DAY - stake.stake_timestamp) / SECONDS_PER_DAY;
                        let multiplier = if stake_age_days >= 60 {
                            Decimal::one()
                        } else {
                            Decimal::from_ratio(stake_age_days, 60)
                        };

                        // Apply the multiplier and accumulate the rewards
                        let adjusted_reward = user_share as i128 * multiplier;
                        pending_rewards += adjusted_reward;
                    }
                }
            }
        }
    }

    pending_rewards
}
