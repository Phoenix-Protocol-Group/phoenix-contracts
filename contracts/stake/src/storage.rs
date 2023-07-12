use soroban_sdk::{
    contracttype, Address, Env, Symbol,
    Map, Vec
};

use crate::error::ContractError;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub token_a: Address,
    pub share_token: Address,
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
    stake: u128,
    /// The timestamp when the stake was made
    stake_timestamp: u64,
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
        None => Err(ContractError::StakeNotFound),
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
