use soroban_sdk::{contracttype, Address, Env};

use curve::Curve;

use crate::error::ContractError;

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

#[allow(dead_code)]
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

#[contracttype]
pub struct Distribution {
    /// How many shares is single point worth
    pub shares_per_point: u128,
    /// Shares which were not fully distributed on previous distributions, and should be redistributed
    pub shares_leftover: u64,
    /// Total rewards distributed by this contract.
    pub distributed_total: u128,
    /// Total rewards not yet withdrawn.
    pub withdrawable_total: u128,
    /// The manager of this distribution
    pub manager: Address,
    /// Max bonus for staking after 60 days
    pub max_bonus_bps: u64,
    /// Bonus per staking day
    pub bonus_per_day_bps: u64,
}

pub fn save_distribution(env: &Env, asset: &Address, distribution: &Distribution) {
    env.storage().persistent().set(asset, distribution);
}
