use soroban_sdk::{contracttype, Address, Env, Vec};

use curve::Curve;

use crate::{error::ContractError, storage::get_stakes};

/// How much points is the worth of single token in rewards distribution.
/// The scaling is performed to have better precision of fixed point division.
/// This value is not actually the scaling itself, but how much bits value should be shifted
/// (for way more efficient division).
///
/// 32, to have those 32 bits, but it reduces how much tokens may be handled by this contract
/// (it is now 96-bit integer instead of 128). In original ERC2222 it is handled by 256-bit
/// calculations, but I256 is missing and it is required for this.
pub const SHARES_SHIFT: u8 = 32;

#[derive(Clone)]
#[contracttype]
pub enum DistributionDataKey {
    Curve(Address),
    Distribution(Address),
    WithdrawAdjustment(Address),
}

// one reward distribution curve over one denom
pub fn save_reward_curve(env: &Env, asset: Address, distribution_curve: &Curve) {
    env.storage()
        .persistent()
        .set(&DistributionDataKey::Curve(asset), distribution_curve);
}

#[allow(dead_code)]
pub fn get_reward_curve(env: &Env, asset: &Address) -> Result<Curve, ContractError> {
    match env
        .storage()
        .persistent()
        .get(&DistributionDataKey::Curve(asset.clone()))
    {
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

pub fn save_distribution(env: &Env, asset: Address, distribution: &Distribution) {
    env.storage()
        .persistent()
        .set(&DistributionDataKey::Distribution(asset), distribution);
}

pub fn get_distribution(env: &Env, asset: &Address) -> Result<Distribution, ContractError> {
    match env
        .storage()
        .persistent()
        .get(&DistributionDataKey::Distribution(asset.clone()))
    {
        Some(distribution) => Ok(distribution),
        None => Err(ContractError::NoRewardsForThisAsset),
    }
}

#[contracttype]
#[derive(Default, Clone)]
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
    user: Address,
    adjustments: &Vec<(Address, WithdrawAdjustment)>,
) {
    env.storage()
        .persistent()
        .set(&DistributionDataKey::WithdrawAdjustment(user), adjustments);
}

pub fn get_withdraw_adjustments(
    env: &Env,
    user: &Address,
) -> Result<Vec<(Address, WithdrawAdjustment)>, ContractError> {
    match env
        .storage()
        .persistent()
        .get(&DistributionDataKey::WithdrawAdjustment(user.clone()))
    {
        Some(adjustments) => Ok(adjustments),
        None => Err(ContractError::WithdrawAdjustmentMissing),
    }
}

pub fn get_withdraw_adjustment(
    withdraw_adjustments: &Vec<(Address, WithdrawAdjustment)>,
    distribution: &Address,
) -> WithdrawAdjustment {
    withdraw_adjustments
        .iter()
        .find(|(asset, _)| asset == distribution)
        .map(|(_, adjustment)| adjustment)
        .unwrap_or_default()
}

pub fn withdrawable_rewards(
    env: &Env,
    owner: &Address,
    distribution: &Distribution,
    adjustment: &WithdrawAdjustment,
) -> Result<u128, ContractError> {
    let ppw = distribution.shares_per_point;

    let points = get_stakes(env, owner)?.total_stake;
    let points = (ppw * points) as i128;

    let correction = adjustment.shared_correction;
    let points = points + correction;
    let amount = points as u128 >> SHARES_SHIFT;
    let amount = amount - adjustment.withdrawn_rewards;

    Ok(amount)
}
