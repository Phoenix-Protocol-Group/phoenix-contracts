use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
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
    env.storage()
        .persistent()
        .get(&CONFIG)
        .expect("Stake: Config not set")
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().persistent().set(&CONFIG, &config);
}

pub mod utils {
    use super::*;

    use soroban_sdk::{ConversionError, TryFromVal, Val};

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
    }

    pub fn save_admin(e: &Env, address: &Address) {
        e.storage().persistent().set(&DataKey::Admin, address)
    }

    pub fn get_admin(e: &Env) -> Address {
        e.storage().persistent().get(&DataKey::Admin).unwrap()
    }
}

// Struct 1:1 from stake contract
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Stake {
    pub stake: i128,
    pub stake_timestamp: u64,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BondingInfo {
    pub stakes: Vec<Stake>,
    pub reward_debt: u128,
    pub last_reward_time: u64,
    pub total_stake: i128,
}
