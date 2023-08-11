use soroban_sdk::{contracttype, Address, Env, Vec};

use curve::Curve;

use crate::{error::ContractError, storage::get_stakes};

// one reward distribution curve over one denom
pub fn save_reward_curve(env: &Env, asset: &Address, distribution_curve: &Curve) {
    env.storage().persistent().set(&asset, distribution_curve);
}

#[allow(dead_code)]
pub fn get_reward_curve(env: &Env, asset: &Address) -> Result<Curve, ContractError> {
    match env.storage().persistent().get(asset) {
        Some(reward_curve) => Ok(reward_curve),
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

impl Distribution {
    pub fn calculate_rewards_power(
        &self,
        env: &Env,
        staker: &Address,
    ) -> Result<u128, ContractError> {
        let bonding_info = get_stakes(env, staker)?;
        let mut total_staked = 0;
        for stake in bonding_info.stakes {
            total_staked += stake.stake;
        }
        Ok(total_staked as u128 / self.shares_per_point)
    }
}

pub fn save_distribution(env: &Env, asset: &Address, distribution: &Distribution) {
    env.storage().persistent().set(asset, distribution);
}

pub fn get_distribution(env: &Env, asset: &Address) -> Result<Distribution, ContractError> {
    match env.storage().persistent().get(asset) {
        Some(distribution) => Ok(distribution),
        None => Err(ContractError::NoRewardsForThisAsset),
    }
}

#[contracttype]
pub struct WithdrawAdjustment {
    /// Represents a correction to the reward points for the user. This can be positive or negative.
    /// A positive value indicates that the user should receive additional points (e.g., from a bonus or an error correction),
    /// while a negative value signifies a reduction (e.g., due to a penalty or an adjustment for past over-allocations).
    pub shared_correction: i128,
    /// Represents the total amount of rewards that the user has withdrawn so far.
    /// This value ensures that a user doesn't withdraw more than they are owed and is used to
    /// calculate the net rewards a user can withdraw at any given time.
    pub withdrawn_rewards: u128,
}

/// Save the withdraw adjustment for a user for a given asset using the user's address as the key
/// and asset's address as the subkey.
#[allow(dead_code)]
pub fn save_withdraw_adjustment(
    env: &Env,
    user: &Address,
    adjustments: &Vec<(Address, WithdrawAdjustment)>,
) {
    env.storage().persistent().set(user, adjustments);
}
