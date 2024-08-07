use soroban_sdk::{log, panic_with_error, Env, U256};

use crate::{error::ContractError, storage::AmplifierParameters, DECIMAL_PRECISION};

// TODO: Those parameters will be used for updating AMP function later
#[allow(dead_code)]
pub const MAX_AMP_CHANGE: u64 = 10;
#[allow(dead_code)]
pub const MIN_AMP_CHANGING_TIME: u64 = 86400;
pub const AMP_PRECISION: u64 = 100;

/// The maximum number of calculation steps for Newton's method.
const ITERATIONS: u8 = 64;
/// N = 2
const N_COINS: u128 = 2;
/// N = 2 with DECIMAL_PRECISION (18) digits
const N_COINS_PRECISION: u128 = 2000000000000000000;
/// 1*10**18
const DECIMAL_FRACTIONAL: u128 = 1_000_000_000_000_000_000;
/// 1e-6
const TOL: u128 = 1000000000000;

pub fn scale_value(atomics: u128, decimal_places: u32, target_decimal_places: u32) -> u128 {
    const TEN: u128 = 10;

    if decimal_places < target_decimal_places {
        let factor = TEN.pow(target_decimal_places - decimal_places);
        atomics
            .checked_mul(factor)
            .expect("Multiplication overflow")
    } else {
        let factor = TEN.pow(decimal_places - target_decimal_places);
        atomics.checked_div(factor).expect("Division overflow")
    }
}

fn abs_diff(a: &U256, b: &U256) -> U256 {
    if a < b {
        b.sub(a)
    } else {
        a.sub(b)
    }
}

/// Compute the current pool amplification coefficient (AMP).
pub(crate) fn compute_current_amp(env: &Env, amp_params: &AmplifierParameters) -> u64 {
    let block_time = env.ledger().timestamp();
    if block_time < amp_params.next_amp_time {
        let elapsed_time: u128 = block_time.saturating_sub(amp_params.init_amp_time).into();
        let time_range = amp_params
            .next_amp_time
            .saturating_sub(amp_params.init_amp_time);
        let init_amp = amp_params.init_amp as u128;
        let next_amp = amp_params.next_amp as u128;

        if next_amp > init_amp {
            let amp_range = next_amp - init_amp;
            let res = init_amp + (amp_range * elapsed_time) / time_range as u128;
            res as u64
        } else {
            let amp_range = init_amp - next_amp;
            let res = init_amp - (amp_range * elapsed_time) / time_range as u128;
            res as u64
        }
    } else {
        amp_params.next_amp
    }
}

/// Computes the stableswap invariant (D).
///
/// * **Equation**
///
/// A * sum(x_i) * n**n + D = A * D * n**n + D**(n+1) / (n**n * prod(x_i))
pub fn compute_d(env: &Env, amp: u128, pools: &[u128]) -> U256 {
    let leverage = U256::from_u128(env, (amp / AMP_PRECISION as u128) * N_COINS_PRECISION);
    let amount_a_times_coins = pools[0] * N_COINS;
    let amount_b_times_coins = pools[1] * N_COINS;

    let sum_x = U256::from_u128(env, pools[0] + pools[1]); // sum(x_i), a.k.a S
    let zero = U256::from_u128(env, 0u128);
    if sum_x == zero {
        return zero;
    }

    let mut d_previous: U256;
    let mut d: U256 = sum_x.clone();

    // Newton's method to approximate D
    for _ in 0..ITERATIONS {
        let d_product = d.pow(3).div(
            &(U256::from_u128(env, amount_a_times_coins)
                .mul(&U256::from_u128(env, amount_b_times_coins))),
        );
        soroban_sdk::testutils::arbitrary::std::dbg!();
        d_previous = d.clone();
        d = calculate_step(env, &d, &leverage, &sum_x, &d_product);
        // Equality with the precision of 1e-6
        if abs_diff(&d, &d_previous) <= U256::from_u128(env, TOL) {
            return d;
        }
    }

    log!(
        &env,
        "Pool Stable: compute_d: Newton method for D failed to converge"
    );
    panic_with_error!(&env, ContractError::NewtonMethodFailed);
}

/// Helper function used to calculate the D invariant as a last step in the `compute_d` public function.
///
/// * **Equation**:
///
/// d = (leverage * sum_x + d_product * n_coins) * initial_d / ((leverage - 1) * initial_d + (n_coins + 1) * d_product)
fn calculate_step(
    env: &Env,
    initial_d: &U256,
    leverage: &U256,
    sum_x: &U256,
    d_product: &U256,
) -> U256 {
    // (leverage * sum_x + d_product * n_coins)
    let leverage_mul = leverage.mul(sum_x);
    let d_p_mul = d_product.mul(&U256::from_u128(env, N_COINS));

    let l_val = leverage_mul.add(&d_p_mul).mul(initial_d);

    // (leverage - 1) * initial_d
    let leverage_sub = initial_d.mul(&leverage.sub(&U256::from_u128(env, 1)));

    // (n_coins + 1) * d_product
    let n_coins_sum = d_product.mul(&(U256::from_u128(env, 3)));

    // Calculate the final step value
    let r_val = leverage_sub.add(&n_coins_sum);

    l_val.div(&r_val)
}

/// Compute the swap amount `y` in proportion to `x`.
///
/// * **Solve for y**
///
/// y**2 + y * (sum' - (A*n**n - 1) * D / (A * n**n)) = D ** (n + 1) / (n ** (2 * n) * prod' * A)
///
/// y**2 + b*y = c
pub(crate) fn calc_y(
    env: &Env,
    amp: u128,
    new_amount: u128,
    xp: &[u128],
    target_precision: u32,
) -> u128 {
    let n_coins = U256::from_u128(env, N_COINS);
    let new_amount = U256::from_u128(env, new_amount);

    let d = compute_d(env, amp, xp);
    let leverage = U256::from_u128(env, amp * DECIMAL_FRACTIONAL * N_COINS);
    let amp_prec = U256::from_u128(env, AMP_PRECISION as u128 * DECIMAL_FRACTIONAL);

    let c = d
        .pow(3)
        .mul(&amp_prec)
        .div(&new_amount.mul(&n_coins.mul(&n_coins)).mul(&leverage));

    let b = new_amount.add(&d.mul(&amp_prec).div(&leverage));

    // Solve for y by approximating: y**2 + b*y = c
    let mut y_prev;
    let mut y = d.clone();
    for _ in 0..ITERATIONS {
        y_prev = y.clone();
        y = (y.pow(2).add(&c)).div(&(y.mul(&n_coins).add(&b).sub(&d)));
        if abs_diff(&y, &y_prev) <= U256::from_u128(env, TOL) {
            let divisor = 10u128.pow(DECIMAL_PRECISION - target_precision);
            return y
                .to_u128()
                .expect("Pool stable: calc_y: conversion to u128 failed")
                / divisor;
        }
    }

    // Should definitely converge in 64 iterations.
    log!(&env, "Pool Stable: calc_y: y is not converging");
    panic_with_error!(&env, ContractError::CalcYErr);
}
