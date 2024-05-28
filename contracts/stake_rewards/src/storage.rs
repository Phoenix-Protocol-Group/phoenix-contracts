use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

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
    env.storage()
        .persistent()
        .get(&CONFIG)
        .expect("Stake: Config not set")
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().persistent().set(&CONFIG, &config);
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
    }

    pub fn get_distributions(e: &Env) -> Vec<Address> {
        e.storage()
            .persistent()
            .get(&DataKey::Distributions)
            .unwrap_or_else(|| soroban_sdk::vec![e])
    }
}
