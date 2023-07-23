use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};

use crate::error::ContractError;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub lp_token: Address,
    pub token_per_power: u128,
    pub min_bond: i128,
    pub max_distributions: u32,
}
const CONFIG: Symbol = Symbol::short("CONFIG");

pub fn get_config(env: &Env) -> Result<Config, ContractError> {
    match env.storage().get(&CONFIG) {
        Some(config) => config.map_err(|_| ContractError::FailedToLoadFromStorage),
        None => Err(ContractError::ConfigNotSet),
    }
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().set(&CONFIG, &config);
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
}

pub fn get_stakes(env: &Env, key: &Address) -> Result<BondingInfo, ContractError> {
    match env.storage().get(&key) {
        Some(stake) => stake.map_err(|_| ContractError::FailedToLoadFromStorage),
        None => Ok(BondingInfo {
            stakes: Vec::new(env),
        }),
    }
}

pub fn save_stakes(env: &Env, key: &Address, bonding_info: &BondingInfo) {
    env.storage().set(key, bonding_info);
}

// pub fn total_rewards_power(&self, storage: &dyn Storage, cfg: &Config, staker: &Addr) -> StdResult<Uint128> {
//     let mut power = Uint128::zero();
//     let bonding_info = STAKE.load(storage, staker)?.unwrap_or_default();
//     for stake in bonding_info.stakes.iter() {
//         let multiplier = self.rewards_multiplier(stake.stake_timestamp);
//         power += calc_power(cfg, stake.stake, multiplier);
//     }
//     Ok(power)
// }
//
// pub fn rewards_multiplier(&self, stake_timestamp: u64) -> Decimal {
//     let days_staked = (env::block_time() - stake_timestamp) / (24 * 60 * 60);
//     let increase = Decimal::percent(0.5) * Decimal::from(days_staked);
//     let capped_increase = std::cmp::min(increase, Decimal::percent(30));
//     Decimal::one() + capped_increase
// }
//
// // Then in your execute_distribute_rewards function:
// let total_rewards = distribution.total_rewards_power(deps.storage, &cfg);

pub mod utils {
    use super::*;

    use soroban_sdk::{ConversionError, RawVal, TryFromVal};

    #[derive(Clone, Copy)]
    #[repr(u32)]
    pub enum DataKey {
        Admin = 0,
    }

    impl TryFromVal<Env, DataKey> for RawVal {
        type Error = ConversionError;

        fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
            Ok((*v as u32).into())
        }
    }

    pub fn save_admin(e: &Env, address: &Address) {
        e.storage().set(&DataKey::Admin, address)
    }

    pub fn get_admin(e: &Env) -> Result<Address, ContractError> {
        e.storage()
            .get_unchecked(&DataKey::Admin)
            .map_err(|_| ContractError::FailedToLoadFromStorage)
    }
}
