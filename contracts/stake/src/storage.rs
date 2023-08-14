use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

use crate::error::ContractError;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub lp_token: Address,
    pub token_per_power: u128,
    pub min_bond: i128,
    pub max_distributions: u32,
    pub min_reward: i128,
}
const CONFIG: Symbol = symbol_short!("CONFIG");

pub fn get_config(env: &Env) -> Result<Config, ContractError> {
    env.storage()
        .instance()
        .get(&CONFIG)
        .ok_or(ContractError::ConfigNotSet)
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().instance().set(&CONFIG, &config);
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
    /// The rewards debt is a mechanism to determine how much a user has already been credited in terms of staking rewards.
    /// Whenever a user deposits or withdraws staked tokens to the pool, the rewards for the user is updated based on the
    /// accumulated rewards per share, and the difference is stored as reward debt. When claiming rewards, this reward debt
    /// is used to determine how much rewards a user can actually claim.
    pub reward_debt: u128,
    /// Last time when user has claimed rewards
    pub last_reward_time: u64,
}

pub fn get_stakes(env: &Env, key: &Address) -> Result<BondingInfo, ContractError> {
    match env.storage().instance().get(&key) {
        Some(stake) => stake,
        None => Ok(BondingInfo {
            stakes: Vec::new(env),
            reward_debt: 0u128,
            last_reward_time: 0u64,
        }),
    }
}

pub fn save_stakes(env: &Env, key: &Address, bonding_info: &BondingInfo) {
    env.storage().instance().set(key, bonding_info);
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

    use soroban_sdk::{ConversionError, TryFromVal, Val};

    #[derive(Clone, Copy)]
    #[repr(u32)]
    pub enum DataKey {
        Admin = 0,
        TotalStaked = 1,
    }

    impl TryFromVal<Env, DataKey> for Val {
        type Error = ConversionError;

        fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
            Ok((*v as u32).into())
        }
    }

    pub fn save_admin(e: &Env, address: &Address) {
        e.storage().instance().set(&DataKey::Admin, address)
    }

    pub fn get_admin(e: &Env) -> Result<Address, ContractError> {
        e.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::FailedToLoadFromStorage)
    }

    pub fn init_staked(e: &Env) {
        e.storage().persistent().set(&DataKey::TotalStaked, &0i128);
    }

    pub fn increase_staked(e: &Env, amount: &i128) {
        let count = get_total_staked_counter(e);

        match count {
            Ok(mut c) => {
                c += amount;

                e.storage().persistent().set(&DataKey::TotalStaked, &c);
            }
            Err(_) => {}
        };
    }

    // there's some annoying code duplication happening above and below this comment line
    // I know I can use something like change_staked(e: &Env, amount: i128, increase: bool)
    // but that just feels wrong for me to increase/decrease based on a boolean value
    // keeping it as is now.. for the moment

    pub fn decrease_staked(e: &Env, amount: &i128) {
        let count = get_total_staked_counter(e);

        match count {
            Ok(mut c) => {
                c -= amount;

                e.storage().persistent().set(&DataKey::TotalStaked, &c);
            }
            Err(_) => {}
        };
    }

    pub fn get_total_staked_counter(env: &Env) -> Result<i128, ContractError> {
        match env.storage().persistent().get(&DataKey::TotalStaked) {
            Some(val) => val,
            None => return Err(ContractError::TotalStakedCannotBeZeroOrLess),
        }
    }
}
