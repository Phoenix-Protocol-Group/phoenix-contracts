use soroban_sdk::{contracttype, Address, Env};

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
pub struct WithdrawAdjustmentKey {
    user: Address,
    asset: Address,
}

#[derive(Clone)]
#[contracttype]
pub enum DistributionDataKey {
    Curve(Address),
    Distribution(Address),
    WithdrawAdjustment(WithdrawAdjustmentKey),
}

// one reward distribution curve over one denom
pub fn save_reward_curve(env: &Env, asset: Address, distribution_curve: &Curve) {
    env.storage()
        .persistent()
        .set(&DistributionDataKey::Curve(asset), distribution_curve);
}

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
    /// The total amount of distribution points; required for proper reward calculation
    pub total_points: u128,
}

pub fn save_distribution(env: &Env, asset: &Address, distribution: &Distribution) {
    env.storage().persistent().set(
        &DistributionDataKey::Distribution(asset.clone()),
        distribution,
    );
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

pub fn update_rewards(
    env: &Env,
    user: &Address,
    asset: &Address,
    distribution: &mut Distribution,
    old_rewards_power: i128,
    new_rewards_power: i128,
) {
    if old_rewards_power == new_rewards_power {
        return;
    }
    let ppw = distribution.shares_per_point;
    let diff = new_rewards_power - old_rewards_power;
    apply_points_correction(env, user, asset, diff, ppw);
}

/// Applies points correction for given address.
/// `shares_per_point` is current value from `SHARES_PER_POINT` - not loaded in function, to
/// avoid multiple queries on bulk updates.
/// `diff` is the points change
fn apply_points_correction(
    env: &Env,
    user: &Address,
    asset: &Address,
    diff: i128,
    shares_per_point: u128,
) {
    let mut withdraw_adjustment = get_withdraw_adjustment(env, user, asset);
    let shares_correction = withdraw_adjustment.shares_correction;
    withdraw_adjustment.shares_correction = shares_correction - shares_per_point as i128 * diff;
    save_withdraw_adjustment(env, user, asset, &withdraw_adjustment);
}

#[contracttype]
#[derive(Debug, Default, Clone)]
pub struct WithdrawAdjustment {
    /// Represents a correction to the reward points for the user. This can be positive or negative.
    /// A positive value indicates that the user should receive additional points (e.g., from a bonus or an error correction),
    /// while a negative value signifies a reduction (e.g., due to a penalty or an adjustment for past over-allocations).
    pub shares_correction: i128,
    /// Represents the total amount of rewards that the user has withdrawn so far.
    /// This value ensures that a user doesn't withdraw more than they are owed and is used to
    /// calculate the net rewards a user can withdraw at any given time.
    pub withdrawn_rewards: u128,
}

/// Save the withdraw adjustment for a user for a given asset using the user's address as the key
/// and asset's address as the subkey.
pub fn save_withdraw_adjustment(
    env: &Env,
    user: &Address,
    distribution: &Address,
    adjustment: &WithdrawAdjustment,
) {
    env.storage().persistent().set(
        &DistributionDataKey::WithdrawAdjustment(WithdrawAdjustmentKey {
            user: user.clone(),
            asset: distribution.clone(),
        }),
        adjustment,
    );
}

pub fn get_withdraw_adjustment(
    env: &Env,
    user: &Address,
    distribution: &Address,
) -> WithdrawAdjustment {
    env.storage()
        .persistent()
        .get(&DistributionDataKey::WithdrawAdjustment(
            WithdrawAdjustmentKey {
                user: user.clone(),
                asset: distribution.clone(),
            },
        ))
        .unwrap_or_default()
}

pub fn withdrawable_rewards(
    env: &Env,
    owner: &Address,
    distribution: &Distribution,
    adjustment: &WithdrawAdjustment,
) -> Result<u128, ContractError> {
    let ppw = distribution.shares_per_point;

    let points = get_stakes(env, owner)?.virtual_stake;
    let points = (ppw * points) as i128;

    let correction = adjustment.shares_correction;
    let points = points + correction;
    let amount = points as u128 >> SHARES_SHIFT;
    let amount = amount - adjustment.withdrawn_rewards;

    Ok(amount)
}
