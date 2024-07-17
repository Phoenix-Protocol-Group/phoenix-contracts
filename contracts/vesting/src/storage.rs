use curve::Curve;
use soroban_sdk::{
    contracttype, log, panic_with_error, vec, Address, ConversionError, Env, String, TryFromVal,
    Val, Vec,
};

use crate::error::ContractError;

impl TryFromVal<Env, DataKey> for Val {
    type Error = ConversionError;

    fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
        Ok((*v as u32).into())
    }
}

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    Admin = 1,
    Config = 2,
    Minter = 3,
    Whitelist = 4,
    VestingTokenInfo = 5,
    MaxVestingComplexity = 6,
    IsInitialized = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingTokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub address: Address,
}

// This structure is used as an argument during the vesting account creation
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingSchedule {
    pub recipient: Address,
    pub curve: Curve,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingInfo {
    // the total amount of tokens left to be distributed
    // it's updated during each claim
    pub balance: u128,
    pub recipient: Address,
    pub schedule: Curve,
}

#[cfg(feature = "minter")]
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinterInfo {
    pub address: Address,
    pub mint_capacity: u128,
}

#[cfg(feature = "minter")]
impl MinterInfo {
    pub fn get_curve(&self) -> Curve {
        Curve::Constant(self.mint_capacity)
    }
}

pub fn save_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .persistent()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| {
            log!(&env, "Vesting: Get admin: Critical error - No admin found");
            panic_with_error!(env, ContractError::NoAdminFound);
        })
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingInfoKey {
    pub recipient: Address,
    pub index: u64,
}

pub fn save_vesting(env: &Env, address: &Address, vesting_info: &VestingInfo) {
    let mut index = 0u64;
    let mut vesting_key = VestingInfoKey {
        recipient: address.clone(),
        index,
    };

    // Find the next available index
    while env.storage().persistent().has(&vesting_key) {
        index += 1;
        vesting_key = VestingInfoKey {
            recipient: address.clone(),
            index,
        };
    }

    env.storage().persistent().set(&vesting_key, vesting_info);
}

pub fn update_vesting(env: &Env, address: &Address, index: u64, vesting_info: &VestingInfo) {
    let vesting_key = VestingInfoKey {
        recipient: address.clone(),
        index,
    };
    env.storage().persistent().set(&vesting_key, vesting_info);
}

pub fn get_vesting(env: &Env, recipient: &Address, index: u64) -> VestingInfo {
    let vesting_key = VestingInfoKey {
        recipient: recipient.clone(),
        index,
    };
    env.storage().persistent().get(&vesting_key).unwrap_or_else(|| {
        log!(&env, "Vesting: Get vesting schedule: Critical error - No vesting schedule found for the given address");
        panic_with_error!(env, ContractError::VestingNotFoundForAddress);
    })
}

pub fn get_all_vestings(env: &Env, address: &Address) -> Vec<VestingInfo> {
    let mut vestings = vec![&env];
    let mut index = 0u64;

    loop {
        let vesting_key = VestingInfoKey {
            recipient: address.clone(),
            index,
        };

        if let Some(vesting_info) = env.storage().persistent().get(&vesting_key) {
            vestings.push_back(vesting_info);
            index += 1;
        } else {
            break;
        }
    }

    vestings
}

#[cfg(feature = "minter")]
pub fn save_minter(env: &Env, minter: &MinterInfo) {
    env.storage().instance().set(&DataKey::Minter, minter);
}

#[cfg(feature = "minter")]
pub fn get_minter(env: &Env) -> Option<MinterInfo> {
    env.storage().instance().get(&DataKey::Minter)
}

pub fn save_token_info(env: &Env, token_info: &VestingTokenInfo) {
    env.storage()
        .instance()
        .set(&DataKey::VestingTokenInfo, token_info);
}

pub fn get_token_info(env: &Env) -> VestingTokenInfo {
    env.storage()
        .instance()
        .get(&DataKey::VestingTokenInfo)
        .unwrap_or_else(|| {
            log!(
                &env,
                "Vesting: Get token info: Critical error - No token info found"
            );
            panic_with_error!(env, ContractError::NoTokenInfoFound);
        })
}

pub fn save_max_vesting_complexity(env: &Env, max_vesting_complexity: &u32) {
    env.storage()
        .instance()
        .set(&DataKey::MaxVestingComplexity, max_vesting_complexity);
}

pub fn get_max_vesting_complexity(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::MaxVestingComplexity)
        .unwrap()
}
pub fn is_initialized(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&DataKey::IsInitialized)
        .unwrap_or(false)
}

pub fn set_initialized(e: &Env) {
    e.storage().instance().set(&DataKey::IsInitialized, &true);
}
