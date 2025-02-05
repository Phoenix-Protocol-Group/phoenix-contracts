use phoenix::ttl::{PERSISTENT_RENEWAL_THRESHOLD, PERSISTENT_TARGET_TTL};
use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

pub const ADMIN: Symbol = symbol_short!("ADMIN");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    // Address of the staking contract that this reward distribution contract is
    // connected to. It can not be changed
    pub staking_contract: Address,
    // Token that is being distributed through this contract
    pub reward_token: Address,
    // Maximum complexity of the reward distribution curve; the bigger, the more resources it uses
    pub max_complexity: u32,
    // Minimum reward amount to be distributed
    pub min_reward: i128,
    // Security precaution - if bond is too small, don't count it towards the bonding power
    pub min_bond: i128,
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

pub mod utils {
    use crate::error::ContractError;

    use super::*;

    use phoenix::ttl::{INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL};
    use soroban_sdk::{log, panic_with_error, ConversionError, TryFromVal, Val};

    #[derive(Clone, Copy)]
    #[repr(u32)]
    pub enum DataKey {
        Initialized = 0,
        Admin = 1,
    }

    impl TryFromVal<Env, DataKey> for Val {
        type Error = ConversionError;

        fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
            Ok((*v as u32).into())
        }
    }

    pub fn is_initialized(e: &Env) -> bool {
        e.storage()
            .persistent()
            .get(&DataKey::Initialized)
            .unwrap_or(false)
    }

    pub fn set_initialized(e: &Env) {
        e.storage().persistent().set(&DataKey::Initialized, &true);
        e.storage().persistent().extend_ttl(
            &DataKey::Initialized,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );
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
            log!(e, "Stake Rewards: Admin not set");
            panic_with_error!(&e, ContractError::AdminNotSet)
        })
    }
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
