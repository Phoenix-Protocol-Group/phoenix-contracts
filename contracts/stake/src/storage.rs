use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

use crate::error::ContractError;
use decimal::Decimal;

const DAY_IN_SECONDS: u64 = 60 * 60 * 24;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub lp_token: Address,
    pub min_bond: i128,
    pub max_distributions: u32,
    pub min_reward: i128,
    /// Max bonus for staking after 60 days
    pub max_bonus_bps: i64,
    /// Bonus per staking day
    pub bonus_per_day_bps: i64,
}
const CONFIG: Symbol = symbol_short!("CONFIG");

pub fn get_config(env: &Env) -> Result<Config, ContractError> {
    env.storage()
        .persistent()
        .get(&CONFIG)
        .ok_or(ContractError::ConfigNotSet)
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().persistent().set(&CONFIG, &config);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stake {
    /// The amount of staked tokens
    pub stake: i128,
    /// The timestamp when the stake was made
    pub stake_timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BondingInfo {
    /// Vec of stakes sorted by stake timestamp
    pub stakes: Vec<Stake>,
    /// Last time when user has claimed rewards
    /// User can withdraw rewards as often as they want, but this parameter resets
    /// only after 24h when reward percentage is bumped
    pub last_reward_time: u64,
    /// Total amount of staked tokens plus rewards
    pub total_stake: u128,
    /// Current rewards percentage in bps
    pub current_rewards_bps: i64,
}

pub fn get_stakes(env: &Env, key: &Address) -> Result<BondingInfo, ContractError> {
    match env.storage().persistent().get(&key) {
        Some(stake) => stake,
        None => Ok(BondingInfo {
            stakes: Vec::new(env),
            last_reward_time: 0u64,
            total_stake: 0u128,
            current_rewards_bps: 0i64,
        }),
    }
}

pub fn save_stakes(env: &Env, key: &Address, bonding_info: &BondingInfo) {
    env.storage().persistent().set(key, bonding_info);
}

pub fn update_stakes_rewards(env: &Env, key: &Address) -> Result<(), ContractError> {
    let mut bonding_info = get_stakes(env, key)?;
    let current_time = env.ledger().timestamp();

    // if last_reward_time is 0, it means that user has never claimed rewards
    // otherwise check if rewards were claimed more than 24h ago
    // (-1 second is to allow rewards to be claimed exactly after 24h)
    if bonding_info.last_reward_time == 0
        || bonding_info.last_reward_time + DAY_IN_SECONDS - 1 < current_time
    {
        bonding_info.last_reward_time = current_time;
        let config = get_config(env)?;
        // if rewards are already at maximum, do nothing
        if bonding_info.current_rewards_bps >= config.max_bonus_bps {
            return Ok(());
        }
        // update rewards percentage (in bps)
        bonding_info.current_rewards_bps += config.bonus_per_day_bps;
        // calculate bonus staking points
        let reward_stake_points =
            bonding_info.total_stake as i128 * Decimal::bps(bonding_info.current_rewards_bps);
        bonding_info.total_stake += reward_stake_points as u128;

        save_stakes(env, key, &bonding_info);
    }
    Ok(())
}

pub mod utils {
    use super::*;

    use soroban_sdk::{ConversionError, TryFromVal, Val};

    #[derive(Clone, Copy)]
    #[repr(u32)]
    pub enum DataKey {
        Admin = 0,
        TotalStaked = 1,
        Distributions = 2,
    }

    impl TryFromVal<Env, DataKey> for Val {
        type Error = ConversionError;

        fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
            Ok((*v as u32).into())
        }
    }

    pub fn save_admin(e: &Env, address: &Address) {
        e.storage().persistent().set(&DataKey::Admin, address)
    }

    pub fn get_admin(e: &Env) -> Result<Address, ContractError> {
        e.storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(ContractError::FailedToGetAdminAddrFromStorage)
    }

    pub fn init_total_staked(e: &Env) {
        e.storage().persistent().set(&DataKey::TotalStaked, &0i128);
    }

    pub fn increase_total_staked(e: &Env, amount: &i128) -> Result<(), ContractError> {
        let count = get_total_staked_counter(e)?;
        e.storage()
            .persistent()
            .set(&DataKey::TotalStaked, &(count + amount));

        Ok(())
    }

    pub fn decrease_total_staked(e: &Env, amount: &i128) -> Result<(), ContractError> {
        let count = get_total_staked_counter(e)?;
        e.storage()
            .persistent()
            .set(&DataKey::TotalStaked, &(count - amount));

        Ok(())
    }

    pub fn get_total_staked_counter(env: &Env) -> Result<i128, ContractError> {
        match env.storage().persistent().get(&DataKey::TotalStaked) {
            Some(val) => val,
            None => Err(ContractError::TotalStakedCannotBeZeroOrLess),
        }
    }

    // Keep track of all distributions to be able to iterate over them
    pub fn add_distribution(e: &Env, asset: &Address) -> Result<(), ContractError> {
        let mut distributions = get_distributions(e);
        if distributions.contains(asset) {
            return Err(ContractError::DistributionAlreadyAdded);
        }
        distributions.push_back(asset.clone());
        e.storage()
            .persistent()
            .set(&DataKey::Distributions, &distributions);
        Ok(())
    }

    pub fn get_distributions(e: &Env) -> Vec<Address> {
        e.storage()
            .persistent()
            .get(&DataKey::Distributions)
            .unwrap_or_else(|| soroban_sdk::vec![e])
    }
}
