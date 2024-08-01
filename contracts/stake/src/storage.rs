use soroban_sdk::{
    contract, contractimpl, contractmeta, contracttype, log, symbol_short, vec, Address, BytesN,
    Env, IntoVal, String, Symbol, Val, Vec,
};

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
    env.storage()
        .persistent()
        .get(&CONFIG)
        .expect("Stake: Config not set")
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().persistent().set(&CONFIG, &config);
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
    match env.storage().persistent().get::<_, BondingInfo>(key) {
        Some(stake) => stake,
        None => BondingInfo {
            stakes: Vec::new(env),
            reward_debt: 0u128,
            last_reward_time: 0u64,
            total_stake: 0i128,
        },
    }
}

pub fn save_stakes(env: &Env, key: &Address, bonding_info: &BondingInfo) {
    env.storage().persistent().set(key, bonding_info);
}

pub mod utils {
    use crate::error::ContractError;

    use super::*;

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
    }

    pub fn save_admin(e: &Env, address: &Address) {
        e.storage().instance().set(&DataKey::Admin, address)
    }

    pub fn get_admin(e: &Env) -> Address {
        e.storage().instance().get(&DataKey::Admin).unwrap()
    }

    pub fn init_total_staked(e: &Env) {
        e.storage().instance().set(&DataKey::TotalStaked, &0i128);
    }

    pub fn increase_total_staked(e: &Env, amount: &i128) {
        let count = get_total_staked_counter(e);
        e.storage()
            .instance()
            .set(&DataKey::TotalStaked, &(count + amount));
    }

    pub fn decrease_total_staked(e: &Env, amount: &i128) {
        let count = get_total_staked_counter(e);
        e.storage()
            .instance()
            .set(&DataKey::TotalStaked, &(count - amount));
    }

    pub fn get_total_staked_counter(env: &Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalStaked).unwrap()
    }

    // Keep track of all distributions to be able to iterate over them
    pub fn add_distribution(e: &Env, asset: &Address, stake_rewards: &Address) {
        let mut distributions = get_distributions(e);
        for (old_asset, _) in distributions.clone() {
            if &old_asset == asset {
                log!(&e, "Stake: Add distribution: Distribution already added");
                panic_with_error!(&e, ContractError::DistributionExists);
            }
        }
        distributions.push_back((asset.clone(), stake_rewards.clone()));
        e.storage()
            .persistent()
            .set(&DataKey::Distributions, &distributions);
    }

    pub fn get_distributions(e: &Env) -> Vec<(Address, Address)> {
        e.storage()
            .persistent()
            .get(&DataKey::Distributions)
            .unwrap_or_else(|| soroban_sdk::vec![e])
    }

    pub fn get_stake_rewards(e: &Env) -> BytesN<32> {
        e.storage()
            .persistent()
            .get(&DataKey::StakeRewards)
            .unwrap()
    }

    pub fn set_stake_rewards(e: &Env, hash: &BytesN<32>) {
        e.storage().persistent().set(&DataKey::StakeRewards, hash);
    }

    pub fn find_stake_rewards_by_asset(e: &Env, asset: &Address) -> Option<Address> {
        let distributions = get_distributions(e);
        for (stored_asset, stake_rewards) in distributions.iter() {
            if &stored_asset == asset {
                return Some(stake_rewards);
            }
        }
        None
    }
}
