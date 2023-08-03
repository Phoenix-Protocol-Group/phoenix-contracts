use soroban_sdk::storage::Storage;
use soroban_sdk::{contracttype, Address, Env, Symbol, TryFromVal, Vec};

use curve::Curve;

use crate::error::ContractError;
use crate::token_contract::Contract;

#[contracttype]
pub struct StorageCurve {
    pub manager: Address,
    pub start_timestamp: u64,
    pub stop_timestamp: u64,
    pub amount_to_distribute: u128,
}

// one reward distribution curve over one denom
pub fn save_reward_curve(env: &Env, asset: &Address, distribution_curve: &StorageCurve) {
    env.storage().persistent().set(&asset, distribution_curve);
}

pub fn get_reward_curve(env: &Env, asset: &Address) -> Result<Curve, ContractError> {
    match env.storage().persistent().get::<_, StorageCurve>(asset) {
        Some(reward_curve) => Ok(Curve::saturating_linear(
            (
                reward_curve.start_timestamp,
                reward_curve.amount_to_distribute,
            ),
            (reward_curve.stop_timestamp, 0),
        )),
        None => Err(ContractError::NoRewardsForThisAsset),
    }
}
