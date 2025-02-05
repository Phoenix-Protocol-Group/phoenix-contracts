use soroban_decimal::Decimal;
use soroban_sdk::{contracttype, log, panic_with_error, Address, Env, Map};

use crate::{error::ContractError, storage::BondingInfo};
use phoenix::ttl::{PERSISTENT_RENEWAL_THRESHOLD, PERSISTENT_TARGET_TTL};

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
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
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
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
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
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
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
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
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
                        let user_share = (stake.stake as u128)
                            .checked_mul(daily_reward)
                            .and_then(|product| product.checked_div(total_staked))
                            .unwrap_or_else(|| {
                                log!(&env, "Pool Stable: Math error in user share calculation");
                                panic_with_error!(&env, ContractError::ContractMathError);
                            });
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
                        pending_rewards = pending_rewards
                            .checked_add(adjusted_reward)
                            .unwrap_or_else(|| {
                                log!(&env, "Pool Stable: overflow occured");
                                panic_with_error!(&env, ContractError::ContractMathError);
                            });
                    }
                }
            }
        }
    }

    pending_rewards
}
