use curve::Curve;
use phoenix::ttl::{
    INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT,
    PERSISTENT_LIFETIME_THRESHOLD,
};
use soroban_sdk::{
    contracttype, log, panic_with_error, symbol_short, vec, Address, ConversionError, Env, String,
    Symbol, TryFromVal, Val, Vec,
};

use crate::error::ContractError;

pub const ADMIN: Symbol = symbol_short!("ADMIN");

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

pub fn save_admin_old(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
    env.storage().persistent().extend_ttl(
        &DataKey::Admin,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn _save_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN, admin);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn get_admin_old(env: &Env) -> Address {
    let admin_addr = env
        .storage()
        .persistent()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| {
            log!(&env, "Vesting: Get admin: Critical error - No admin found");
            panic_with_error!(env, ContractError::NoAdminFound);
        });
    env.storage().persistent().extend_ttl(
        &DataKey::Admin,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );

    admin_addr
}

pub fn _get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

    env.storage().instance().get(&ADMIN).unwrap_or_else(|| {
        log!(&env, "Vesting: Admin not set");
        panic_with_error!(&env, ContractError::AdminNotFound)
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
    env.storage().persistent().extend_ttl(
        &vesting_key,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn update_vesting(env: &Env, address: &Address, index: u64, vesting_info: &VestingInfo) {
    let vesting_key = VestingInfoKey {
        recipient: address.clone(),
        index,
    };
    env.storage().persistent().set(&vesting_key, vesting_info);
    env.storage().persistent().extend_ttl(
        &vesting_key,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn get_vesting(env: &Env, recipient: &Address, index: u64) -> VestingInfo {
    let vesting_key = VestingInfoKey {
        recipient: recipient.clone(),
        index,
    };
    let vesting_info = env.storage().persistent().get(&vesting_key).unwrap_or_else(|| {
        log!(&env, "Vesting: Get vesting schedule: Critical error - No vesting schedule found for the given address");
        panic_with_error!(env, ContractError::VestingNotFoundForAddress);
    });
    env.storage().persistent().extend_ttl(
        &vesting_key,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );

    vesting_info
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
            env.storage().persistent().extend_ttl(
                &vesting_key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        } else {
            break;
        }
    }

    vestings
}

#[cfg(feature = "minter")]
pub fn save_minter(env: &Env, minter: &MinterInfo) {
    env.storage().instance().set(&DataKey::Minter, minter);
    env.storage()
        .instance()
        .extend_ttl(PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

#[cfg(feature = "minter")]
pub fn get_minter(env: &Env) -> Option<MinterInfo> {
    use phoenix::ttl::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};

    let minter_info = env.storage().instance().get(&DataKey::Minter);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

    minter_info
}

pub fn save_token_info(env: &Env, token_info: &VestingTokenInfo) {
    env.storage()
        .instance()
        .set(&DataKey::VestingTokenInfo, token_info);
    env.storage()
        .instance()
        .extend_ttl(PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

pub fn get_token_info(env: &Env) -> VestingTokenInfo {
    let vesting_token_info = env
        .storage()
        .instance()
        .get(&DataKey::VestingTokenInfo)
        .unwrap_or_else(|| {
            log!(
                &env,
                "Vesting: Get token info: Critical error - No token info found"
            );
            panic_with_error!(env, ContractError::NoTokenInfoFound);
        });
    env.storage()
        .instance()
        .extend_ttl(PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);

    vesting_token_info
}

pub fn save_max_vesting_complexity(env: &Env, max_vesting_complexity: &u32) {
    env.storage()
        .instance()
        .set(&DataKey::MaxVestingComplexity, max_vesting_complexity);
    env.storage()
        .instance()
        .extend_ttl(PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

pub fn get_max_vesting_complexity(env: &Env) -> u32 {
    let vesting_complexity = env
        .storage()
        .instance()
        .get(&DataKey::MaxVestingComplexity)
        .unwrap();
    env.storage()
        .instance()
        .extend_ttl(PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);

    vesting_complexity
}

pub fn is_initialized(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&DataKey::IsInitialized)
        .unwrap_or(false)
}

pub fn set_initialized(e: &Env) {
    e.storage().instance().set(&DataKey::IsInitialized, &true);
    e.storage()
        .instance()
        .extend_ttl(PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}
