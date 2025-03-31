use phoenix::ttl::{PERSISTENT_RENEWAL_THRESHOLD, PERSISTENT_TARGET_TTL};
use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

#[allow(dead_code)]
pub const ADMIN: Symbol = symbol_short!("ADMIN");
pub const STAKE_KEY: Symbol = symbol_short!("STAKE");
pub(crate) const PENDING_ADMIN: Symbol = symbol_short!("p_admin");

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
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    config
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().persistent().set(&CONFIG, &config);
    env.storage().persistent().extend_ttl(
        &CONFIG,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
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
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );
    });

    bonding_info
}

pub fn save_stakes(env: &Env, key: &Address, bonding_info: &BondingInfo) {
    env.storage().persistent().set(key, bonding_info);
    env.storage().persistent().extend_ttl(
        &key,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );
}

pub mod utils {
    use crate::error::ContractError;

    use super::*;

    use phoenix::ttl::{INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL};
    use soroban_sdk::{log, panic_with_error, ConversionError, TryFromVal, Val};

    #[derive(Clone, Copy)]
    #[repr(u32)]
    pub enum DataKey {
        Admin = 0,
        TotalStaked = 1,
        Distributions = 2,
        Initialized = 3,  // TODO: deprecated, remove in future upgrade
        StakeRewards = 4, // maybe deprecated
    }

    impl TryFromVal<Env, DataKey> for Val {
        type Error = ConversionError;

        fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
            Ok((*v as u32).into())
        }
    }

    pub fn save_admin_old(e: &Env, address: &Address) {
        e.storage().persistent().set(&DataKey::Admin, address);
        e.storage().persistent().extend_ttl(
            &DataKey::Admin,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );
    }

    pub fn _save_admin(e: &Env, address: &Address) {
        e.storage().instance().set(&ADMIN, &address);
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
    }

    pub fn get_admin_old(e: &Env) -> Address {
        let admin = e.storage().persistent().get(&DataKey::Admin).unwrap();
        e.storage().persistent().extend_ttl(
            &DataKey::Admin,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );

        admin
    }

    pub fn _get_admin(e: &Env) -> Address {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        e.storage().instance().get(&ADMIN).unwrap_or_else(|| {
            log!(e, "Stake: Admin not set");
            panic_with_error!(&e, ContractError::AdminNotSet)
        })
    }

    pub fn init_total_staked(e: &Env) {
        e.storage().persistent().set(&DataKey::TotalStaked, &0i128);
        e.storage().persistent().extend_ttl(
            &DataKey::TotalStaked,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );
    }

    pub fn increase_total_staked(e: &Env, amount: &i128) {
        let count = get_total_staked_counter(e);
        let new_sum = count.checked_add(*amount).unwrap_or_else(|| {
            log!(&e, "Stake: Increase Total Staked: Overflow occured.");
            panic_with_error!(&e, ContractError::ContractMathError);
        });
        e.storage()
            .persistent()
            .set(&DataKey::TotalStaked, &new_sum);

        e.storage().persistent().extend_ttl(
            &DataKey::TotalStaked,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );
    }

    pub fn decrease_total_staked(e: &Env, amount: &i128) {
        let count = get_total_staked_counter(e);

        let new_diff = count.checked_sub(*amount).unwrap_or_else(|| {
            log!(&e, "Stake: Increase Total Staked: Overflow occured.");
            panic_with_error!(&e, ContractError::ContractMathError);
        });
        e.storage()
            .persistent()
            .set(&DataKey::TotalStaked, &new_diff);

        e.storage().persistent().extend_ttl(
            &DataKey::TotalStaked,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );
    }

    pub fn get_total_staked_counter(env: &Env) -> i128 {
        let total_staked = env
            .storage()
            .persistent()
            .get(&DataKey::TotalStaked)
            // or maybe .unwrap_or(0)
            .unwrap_or_else(|| {
                log!(&env, "Stake: Get Total Staked Counter: No value found");
                panic_with_error!(&env, ContractError::StakeNotFound);
            });
        env.storage().persistent().extend_ttl(
            &DataKey::TotalStaked,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );

        total_staked
    }

    // Keep track of all distributions to be able to iterate over them
    pub fn add_distribution(e: &Env, asset: &Address) {
        let mut distributions = get_distributions(e);
        if distributions.contains(asset) {
            log!(&e, "Stake: Add distribution: Distribution already added");
            panic_with_error!(&e, ContractError::DistributionExists);
        }
        distributions.push_back(asset.clone());
        e.storage()
            .persistent()
            .set(&DataKey::Distributions, &distributions);
        e.storage().persistent().extend_ttl(
            &DataKey::Distributions,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
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
                    PERSISTENT_RENEWAL_THRESHOLD,
                    PERSISTENT_TARGET_TTL,
                )
            });

        distributions
    }
}
