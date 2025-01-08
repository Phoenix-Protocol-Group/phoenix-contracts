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

pub fn scale_value(
    env: &Env,
    atomics: u128,
    decimal_places: u32,
    target_decimal_places: u32,
) -> u128 {
    let ten = U256::from_u128(env, 10);
    let atomics = U256::from_u128(env, atomics);

    let scaled_value = if decimal_places < target_decimal_places {
        let power = target_decimal_places - decimal_places;
        let factor = ten.pow(power);
        atomics.mul(&factor)
    } else {
        let power = decimal_places - target_decimal_places;
        let factor = ten.pow(power);
        atomics.div(&factor)
    };

    scaled_value
        .to_u128()
        .expect("Value doesn't fit into u128!")
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
    let amount_a_times_coins = U256::from_u128(env, pools[0]).mul(&U256::from_u128(env, N_COINS));
    let amount_b_times_coins = U256::from_u128(env, pools[1]).mul(&U256::from_u128(env, N_COINS));
    let sum_x = U256::from_u128(env, pools[0] + pools[1]); // sum(x_i), a.k.a S
    let zero = U256::from_u128(env, 0u128);
    if sum_x == zero {
        return zero;
    }

    let mut d_previous: U256;
    let mut d: U256 = sum_x.clone();

    // Newton's method to approximate D
    for _ in 0..ITERATIONS {
        let a_times_b_product = amount_a_times_coins.mul(&amount_b_times_coins);
        let d_product = d.pow(3).div(&a_times_b_product);
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

/// Compute the swap amount `y` in proportion to `x` using a partially-reordered formula.
///
/// Original stable-swap equation:
/// ```text
/// y² + y * (sum' - (A*n^n - 1) * D / (A * n^n)) = D^(n+1) / (n^(2n) * prod' * A)
///
/// => y² + b·y = c
/// ```
/// We chunk up multiplications/divisions to avoid overflow when computing `d³ * amp_prec`.
pub(crate) fn calc_y(
    env: &Env,
    amp: u128,
    new_amount_u128: u128,
    xp: &[u128],
    target_precision: u32,
) -> u128 {
    // number of coins in the pool, e.g. 2 for a two-coin stableswap.
    let coins_count = U256::from_u128(env, N_COINS);

    // convert `new_amount_u128` to U256 for big math.
    let new_u256_amount = U256::from_u128(env, new_amount_u128);

    // compute the stableswap invariant D.
    let invariant_d = compute_d(env, amp, xp);

    let amp_precision_factor = U256::from_u128(env, (AMP_PRECISION as u128) * DECIMAL_FRACTIONAL);

    // compute "leverage" = amp * DECIMAL_FRACTIONAL * n_coins.
    let leverage = U256::from_u128(env, amp)
        .mul(&U256::from_u128(env, DECIMAL_FRACTIONAL))
        .mul(&coins_count);

    // ------------------------------------------------------------------
    // Now we compute:
    //   c = (D^3 * amp_precision_factor) / (new_amount * n_coins^2 * leverage)
    // but we do it in multiple steps to prevent overflow.
    // ------------------------------------------------------------------

    // Step A: D²
    let invariant_sq = invariant_d.mul(&invariant_d);

    // Step B: multiply coins_count by itself => n_coins^2, then times new_u256_amount.
    // denominator_chunk1 = n_coins^2 * new_amount
    let coins_count_sq = coins_count.mul(&coins_count);
    let denominator_chunk1 = coins_count_sq.mul(&new_u256_amount);

    // Step C: partial factor => (D² / (n_coins^2 * new_amount))
    let temp_factor1 = invariant_sq.div(&denominator_chunk1);

    // Step D: multiply by D => (D³ / (n_coins^2 * new_amount))
    let temp_factor2 = temp_factor1.mul(&invariant_d);

    // Step E: multiply by amp_precision_factor => (D³ * amp_prec) / (n_coins^2 * new_amount)
    let temp_factor3 = temp_factor2.mul(&amp_precision_factor);

    // Step F: finally divide by leverage =>
    //   c = (D³ * amp_prec) / (n_coins^2 * new_amount * leverage)
    let constant_c = temp_factor3.div(&leverage);

    // ------------------------------------------------------------------
    // b = new_amount + (D * amp_precision_factor / leverage)
    // ------------------------------------------------------------------
    let coefficient_b = {
        let scaled_d = invariant_d.mul(&amp_precision_factor).div(&leverage);
        new_u256_amount.add(&scaled_d)
    };

    // ------------------------------------------------------------------
    // Solve for y in the equation:
    //   y² + b·y = c  ==>  y = (y² + c) / (n_coins·y + b - D)
    // Using iteration (Newton-like).
    // ------------------------------------------------------------------
    let mut y_guess = invariant_d.clone();
    for _ in 0..ITERATIONS {
        let y_prev = y_guess.clone();

        // Numerator = y² + c
        let numerator = y_guess.pow(2).add(&constant_c);

        // Denominator = n_coins·y + b - D
        let denominator = coins_count
            .mul(&y_guess)
            .add(&coefficient_b)
            .sub(&invariant_d);

        // Next approximation for y
        y_guess = numerator.div(&denominator);

        // Check convergence
        if abs_diff(&y_guess, &y_prev) <= U256::from_u128(env, TOL) {
            // Scale down from DECIMAL_PRECISION to `target_precision`.
            let divisor = 10u128.pow(DECIMAL_PRECISION - target_precision);
            return y_guess
                .to_u128()
                .expect("calc_y: final y doesn't fit in u128!")
                / divisor;
        }
    }

    // If not converged in 64 iterations, we treat that as an error.
    log!(env, "calc_y: not converging in 64 iterations!");
    panic_with_error!(env, ContractError::CalcYErr);
}
