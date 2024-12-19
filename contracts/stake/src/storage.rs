use phoenix::ttl::{PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};
use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

use crate::stake_rewards_contract;
pub const ADMIN: Symbol = symbol_short!("ADMIN");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub lp_token: Address,
    pub min_bond: i128,
    pub min_reward: i128,
    // Address of a user, allowed to create distribution flows
    pub manager: Address,
    // Address of the factory contract that initialized this pool and stake contract
    pub owner: Address,
    // Maximum complexity for the reward distribution curve
    pub max_complexity: u32,
}
const CONFIG: Symbol = symbol_short!("CONFIG");

pub fn get_config(env: &Env) -> Config {
    let config = env
        .storage()
        .persistent()
        .get(&CONFIG)
        .expect("Stake: Config not set");
    env.storage().persistent().extend_ttl(
        &CONFIG,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );

    config
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().persistent().set(&CONFIG, &config);
    env.storage().persistent().extend_ttl(
        &CONFIG,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Default)]
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
    /// Total amount of staked tokens
    pub total_stake: i128,
}

pub fn get_stakes(env: &Env, key: &Address) -> BondingInfo {
    let bonding_info = match env.storage().persistent().get::<_, BondingInfo>(key) {
        Some(stake) => stake,
        None => BondingInfo {
            stakes: Vec::new(env),
            reward_debt: 0u128,
            last_reward_time: 0u64,
            total_stake: 0i128,
        },
    };
    env.storage().persistent().has(&key).then(|| {
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    });

    bonding_info
}

pub fn save_stakes(env: &Env, key: &Address, bonding_info: &BondingInfo) {
    env.storage().persistent().set(key, bonding_info);
    env.storage().persistent().extend_ttl(
        &key,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub mod utils {
    use crate::error::ContractError;

    use super::*;

    use phoenix::ttl::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
    use soroban_sdk::{log, panic_with_error, ConversionError, TryFromVal, Val};

    #[derive(Clone, Copy)]
    #[repr(u32)]
    pub enum DataKey {
        Admin = 0,
        TotalStaked = 1,
        Distributions = 2,
        Initialized = 3,
        StakeRewards = 4,
    }

    impl TryFromVal<Env, DataKey> for Val {
        type Error = ConversionError;

        fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
            Ok((*v as u32).into())
        }
    }

    pub fn is_initialized(e: &Env) -> bool {
        e.storage()
            .instance()
            .get(&DataKey::Initialized)
            .unwrap_or(false)
    }

    pub fn set_initialized(e: &Env) {
        e.storage().instance().set(&DataKey::Initialized, &true);
        e.storage()
            .instance()
            .extend_ttl(PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    }

    pub fn save_admin_old(e: &Env, address: &Address) {
        e.storage().persistent().set(&DataKey::Admin, address);
        e.storage().persistent().extend_ttl(
            &DataKey::Admin,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn _save_admin(e: &Env, address: &Address) {
        e.storage().instance().set(&ADMIN, &address);
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
    }

    pub fn get_admin_old(e: &Env) -> Address {
        let admin = e.storage().persistent().get(&DataKey::Admin).unwrap();
        e.storage().persistent().extend_ttl(
            &DataKey::Admin,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        admin
    }

    pub fn _get_admin(e: &Env) -> Address {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        e.storage().instance().get(&ADMIN).unwrap_or_else(|| {
            log!(e, "Stake: Admin not set");
            panic_with_error!(&e, ContractError::AdminNotSet)
        })
    }

    pub fn init_total_staked(e: &Env) {
        e.storage().persistent().set(&DataKey::TotalStaked, &0i128);
        e.storage().persistent().extend_ttl(
            &DataKey::TotalStaked,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn increase_total_staked(e: &Env, amount: &i128) {
        let count = get_total_staked_counter(e);
        e.storage()
            .persistent()
            .set(&DataKey::TotalStaked, &(count + amount));

        e.storage().persistent().extend_ttl(
            &DataKey::TotalStaked,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn decrease_total_staked(e: &Env, amount: &i128) {
        let count = get_total_staked_counter(e);
        e.storage()
            .persistent()
            .set(&DataKey::TotalStaked, &(count - amount));

        e.storage().persistent().extend_ttl(
            &DataKey::TotalStaked,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn get_total_staked_counter(env: &Env) -> i128 {
        let total_staked = env
            .storage()
            .persistent()
            .get(&DataKey::TotalStaked)
            .unwrap();
        env.storage().persistent().extend_ttl(
            &DataKey::TotalStaked,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        total_staked
    }

    // Keep track of all distributions to be able to iterate over them
    pub fn add_distribution(e: &Env, asset: &Address) {
        let mut distributions = get_distributions(e);
        for old_asset in distributions.clone() {
            if &old_asset == asset {
                log!(&e, "Stake: Add distribution: Distribution already added");
                panic_with_error!(&e, ContractError::DistributionExists);
            }
        }
        distributions.push_back(asset.clone());
        e.storage()
            .persistent()
            .set(&DataKey::Distributions, &distributions);
        e.storage().persistent().extend_ttl(
            &DataKey::Distributions,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn get_distributions(e: &Env) -> Vec<Address> {
        let distributions = e
            .storage()
            .persistent()
            .get(&DataKey::Distributions)
            .unwrap_or_else(|| soroban_sdk::vec![e]);
        e.storage()
            .persistent()
            .has(&DataKey::Distributions)
            .then(|| {
                e.storage().persistent().extend_ttl(
                    &DataKey::Distributions,
                    PERSISTENT_LIFETIME_THRESHOLD,
                    PERSISTENT_BUMP_AMOUNT,
                )
            });

        distributions
    }
}

// Implement `From` trait for conversion between `BondingInfo` structs
impl From<BondingInfo> for stake_rewards_contract::BondingInfo {
    fn from(info: BondingInfo) -> Self {
        let mut stakes = Vec::new(info.stakes.env());
        for stake in info.stakes.iter() {
            stakes.push_back(stake.into());
        }
        stake_rewards_contract::BondingInfo {
            stakes,
            reward_debt: info.reward_debt,
            last_reward_time: info.last_reward_time,
            total_stake: info.total_stake,
        }
    }
}

impl From<stake_rewards_contract::BondingInfo> for BondingInfo {
    fn from(info: stake_rewards_contract::BondingInfo) -> Self {
        let mut stakes = Vec::new(info.stakes.env());
        for stake in info.stakes.iter() {
            stakes.push_back(stake.into());
        }
        BondingInfo {
            stakes,
            reward_debt: info.reward_debt,
            last_reward_time: info.last_reward_time,
            total_stake: info.total_stake,
        }
    }
}

// Implement `From` trait for conversion between `Stake` structs
impl From<Stake> for stake_rewards_contract::Stake {
    fn from(stake: Stake) -> Self {
        stake_rewards_contract::Stake {
            stake: stake.stake,
            stake_timestamp: stake.stake_timestamp,
        }
    }
}

impl From<stake_rewards_contract::Stake> for Stake {
    fn from(stake: stake_rewards_contract::Stake) -> Self {
        Stake {
            stake: stake.stake,
            stake_timestamp: stake.stake_timestamp,
        }
    }
}
