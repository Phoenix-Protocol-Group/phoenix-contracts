use phoenix::{
    ttl::{PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD},
    utils::{convert_i128_to_u128, convert_u128_to_i128},
};
use soroban_sdk::{contracttype, log, panic_with_error, Address, Env};

use curve::Curve;
use soroban_decimal::Decimal;

use crate::{error::ContractError, storage::BondingInfo};
use crate::{
    error::ContractError,
    storage::{get_stakes, Config},
    TOKEN_PER_POWER,
};
use phoenix::ttl::{PERSISTENT_RENEWAL_THRESHOLD, PERSISTENT_TARGET_TTL};

/// How much points is the worth of single token in rewards distribution.
/// The scaling is performed to have better precision of fixed point division.
/// This value is not actually the scaling itself, but how much bits value should be shifted
/// (for way more efficient division).
///
/// 32, to have those 32 bits, but it reduces how much tokens may be handled by this contract
/// (it is now 96-bit integer instead of 128). In original ERC2222 it is handled by 256-bit
/// calculations, but I256 is missing and it is required for this.
pub const SHARES_SHIFT: u8 = 32;

const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;

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
    env.storage().persistent().set(
        &DistributionDataKey::Curve(asset.clone()),
        distribution_curve,
    );
    env.storage().persistent().extend_ttl(
        &DistributionDataKey::Curve(asset),
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn get_reward_curve(env: &Env, asset: &Address) -> Option<Curve> {
    let result = env
        .storage()
        .persistent()
        .get(&DistributionDataKey::Curve(asset.clone()));

    env.storage()
        .persistent()
        .has(&DistributionDataKey::Curve(asset.clone()))
        .then(|| {
            env.storage().persistent().extend_ttl(
                &DistributionDataKey::Curve(asset.clone()),
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            )
        });

    result
}

#[contracttype]
#[derive(Debug, Default, Clone)]
pub struct Distribution {
    /// How many shares is single point worth
    pub shares_per_point: u128,
    /// Shares which were not fully distributed on previous distributions, and should be redistributed
    pub shares_leftover: u64,
    /// Total rewards distributed by this contract.
    pub distributed_total: u128,
    /// Total rewards not yet withdrawn.
    pub withdrawable_total: u128,
    /// Max bonus for staking after 60 days
    pub max_bonus_bps: u64,
    /// Bonus per staking day
    pub bonus_per_day_bps: u64,
}

pub fn save_distribution(env: &Env, asset: &Address, distribution: &Distribution) {
    env.storage().persistent().set(
        &DistributionDataKey::Distribution(asset.clone()),
        distribution,
    );

    env.storage().persistent().extend_ttl(
        &DistributionDataKey::Distribution(asset.clone()),
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    )
}

pub fn get_distribution(env: &Env, asset: &Address) -> Distribution {
    let distribution = env
        .storage()
        .persistent()
        .get(&DistributionDataKey::Distribution(asset.clone()))
        .unwrap();

    env.storage()
        .persistent()
        .has(&DistributionDataKey::Distribution(asset.clone()))
        .then(|| {
            env.storage().persistent().extend_ttl(
                &DistributionDataKey::Distribution(asset.clone()),
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            )
        });

    distribution
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
    let diff = new_rewards_power
        .checked_sub(old_rewards_power)
        .unwrap_or_else(|| {
            log!(&env, "Stake: Update Rewards: Underflow occured.");
            panic_with_error!(&env, ContractError::ContractMathError);
        });
    // Apply the points correction with the calculated difference.
    let ppw = distribution.shares_per_point;
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
    withdraw_adjustment.shares_correction = (shares_per_point as i128)
        .checked_mul(diff)
        .and_then(|result| shares_correction.checked_sub(result))
        .unwrap_or_else(|| {
            log!(
                &env,
                "Stake: Apply Points Correction: Underflow/Overflow occured."
            );
            panic_with_error!(&env, ContractError::ContractMathError);
        });
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
    env.storage().persistent().extend_ttl(
        &DistributionDataKey::WithdrawAdjustment(WithdrawAdjustmentKey {
            user: user.clone(),
            asset: distribution.clone(),
        }),
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn get_withdraw_adjustment(
    env: &Env,
    user: &Address,
    distribution: &Address,
) -> WithdrawAdjustment {
    let result = env
        .storage()
        .persistent()
        .get(&DistributionDataKey::WithdrawAdjustment(
            WithdrawAdjustmentKey {
                user: user.clone(),
                asset: distribution.clone(),
            },
        ))
        .unwrap_or_default();

    env.storage()
        .persistent()
        .has(&DistributionDataKey::WithdrawAdjustment(
            WithdrawAdjustmentKey {
                user: user.clone(),
                asset: distribution.clone(),
            },
        ))
        .then(|| {
            env.storage().persistent().extend_ttl(
                &DistributionDataKey::WithdrawAdjustment(WithdrawAdjustmentKey {
                    user: user.clone(),
                    asset: distribution.clone(),
                }),
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        });

    result
}

pub fn withdrawable_rewards(
    env: &Env,
    owner: &Address,
    distribution: &Distribution,
    adjustment: &WithdrawAdjustment,
    config: &Config,
) -> u128 {
    let ppw = distribution.shares_per_point;

    let stakes: i128 = get_stakes(env, owner).total_stake;
    // Decimal::one() represents the standart multiplier per token
    // 1_000 represents the contsant token per power. TODO: make it configurable
    let points = calc_power(config, stakes as i128, Decimal::one(), TOKEN_PER_POWER);
    let points = convert_u128_to_i128(
        ppw.checked_mul(convert_i128_to_u128(points))
            .unwrap_or_else(|| {
                log!(&env, "Stake: Withdrawable Rewards: Overflow occured.");
                panic_with_error!(&env, ContractError::ContractMathError);
            }),
    );

    let correction = adjustment.shares_correction;
    let points = points.checked_add(correction).unwrap_or_else(|| {
        log!(&env, "Stake: Withdrawable Rewards: Underflow occured.");
        panic_with_error!(&env, ContractError::ContractMathError);
    });
    let amount = points.checked_shr(SHARES_SHIFT.into()).unwrap_or_else(|| {
        log!(&env, "Stake Withdrawable Rewards: Underflow occured.");
        panic_with_error!(&env, ContractError::ContractMathError);
    });
    convert_i128_to_u128(amount)
        .checked_sub(adjustment.withdrawn_rewards)
        .unwrap_or_else(|| {
            log!(&env, "Stake: Withdrawable Rewards: Underflow occured.");
            panic_with_error!(&env, ContractError::ContractMathError);
        })
}

pub fn calculate_annualized_payout(reward_curve: Option<Curve>, now: u64) -> Decimal {
    match reward_curve {
        Some(c) => {
            // look at the last timestamp in the rewards curve and extrapolate
            match c.end() {
                Some(last_timestamp) => {
                    if last_timestamp <= now {
                        return Decimal::zero();
                    }
                    let time_diff = last_timestamp - now;
                    if time_diff >= SECONDS_PER_YEAR {
                        // if the last timestamp is more than a year in the future,
                        // we can just calculate the rewards for the whole year directly

                        // formula: `(locked_now - locked_end)`
                        Decimal::from_atomics(
                            (c.value(now) - c.value(now + SECONDS_PER_YEAR)) as i128,
                            0,
                        )
                    } else {
                        // if the last timestamp is less than a year in the future,
                        // we want to extrapolate the rewards for the whole year

                        // formula: `(locked_now - locked_end) / time_diff * SECONDS_PER_YEAR`
                        // `locked_now - locked_end` are the tokens freed up over the `time_diff`.
                        // Dividing by that diff, gives us the rate of tokens per second,
                        // which is then extrapolated to a whole year.
                        // Because of the constraints put on `c` when setting it,
                        // we know that `locked_end` is always 0, so we don't need to subtract it.
                        Decimal::from_ratio(
                            (c.value(now) * SECONDS_PER_YEAR as u128) as i128,
                            time_diff,
                        )
                    }
                }
                None => {
                    // this case should only happen if the reward curve is freshly initialized
                    // (i.e. no rewards have been scheduled yet)
                    Decimal::zero()
                }
            }
        }
        None => Decimal::zero(),
    }
}

pub fn calc_power(
    config: &Config,
    stakes: i128,
    multiplier: Decimal,
    token_per_power: i32,
) -> i128 {
    if stakes < config.min_bond {
        0
    } else {
        stakes * multiplier / token_per_power as i128
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use curve::SaturatingLinear;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn update_rewards_should_return_early_if_old_power_is_same_as_new_power() {
        let env = Env::default();
        let user = Address::generate(&env);
        let asset = Address::generate(&env);
        let mut distribution = Distribution::default();

        let old_rewards_power = 100;
        let new_rewards_power = 100;

        // it's only enough not to panic as the inner method call to apply_points_correction calls get_withdraw_adjustment
        // this would trigger InternalError otherwise
        update_rewards(
            &env,
            &user,
            &asset,
            &mut distribution,
            old_rewards_power,
            new_rewards_power,
        );
    }

    #[test]
    fn calculate_annualized_payout_should_return_zero_when_last_timestamp_in_the_past() {
        let reward_curve = Some(Curve::SaturatingLinear(SaturatingLinear {
            min_x: 15,
            min_y: 1,
            max_x: 60,
            max_y: 120,
        }));
        let result = calculate_annualized_payout(reward_curve, 121);
        assert_eq!(result, Decimal::zero());
    }

    #[test]
    fn calculate_annualized_payout_extrapolating_an_year() {
        let reward_curve = Some(Curve::SaturatingLinear(SaturatingLinear {
            min_x: 15,
            min_y: 1,
            max_x: SECONDS_PER_YEAR + 60,
            max_y: (SECONDS_PER_YEAR + 120) as u128,
        }));
        // we take the last timestamp in the curve and extrapolate the rewards for a year
        let result = calculate_annualized_payout(reward_curve, SECONDS_PER_YEAR + 1);
        // a bit weird assertion, but we're testing the extrapolation with a large number
        assert_eq!(
            result,
            Decimal::new(16_856_291_324_745_762_711_864_406_779_661)
        );
    }

    #[test]
    fn calculate_annualized_payout_should_return_zero_no_end_in_curve() {
        let reward_curve = Some(Curve::Constant(10));
        let result = calculate_annualized_payout(reward_curve, 121);
        assert_eq!(result, Decimal::zero());
    }

    #[test]
    fn calculate_annualized_payout_should_return_zero_no_curve() {
        let reward_curve = None::<Curve>;
        let result = calculate_annualized_payout(reward_curve, 121);
        assert_eq!(result, Decimal::zero());
    }
}
