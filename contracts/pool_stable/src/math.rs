use soroban_sdk::Env;

use crate::storage::AmplifierParameters;

use decimal::Decimal;

// TODO: Those parameters will be used for updating AMP function later
#[allow(dead_code)]
pub const MAX_AMP: u64 = 1_000_000;
#[allow(dead_code)]
pub const MAX_AMP_CHANGE: u64 = 10;
#[allow(dead_code)]
pub const MIN_AMP_CHANGING_TIME: u64 = 86400;
pub const AMP_PRECISION: u64 = 100;

/// The maximum number of calculation steps for Newton's method.
const ITERATIONS: u8 = 64;
/// N = 2
pub const N_COINS: Decimal = Decimal::raw(2000000000000000000);
/// 1e-6
pub const TOL: Decimal = Decimal::raw(1000000000000);

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
pub fn compute_d(amp: u128, pools: &[Decimal]) -> Decimal {
    let leverage = Decimal::from_ratio(amp as i128, AMP_PRECISION) * N_COINS;
    let amount_a_times_coins = pools[0] * N_COINS;
    let amount_b_times_coins = pools[1] * N_COINS;

    let sum_x = pools[0] + pools[1]; // sum(x_i), a.k.a S
    if sum_x.is_zero() {
        return Decimal::zero();
    }

    let mut d_previous: Decimal;
    let mut d: Decimal = sum_x;

    // Newton's method to approximate D
    for _ in 0..ITERATIONS {
        let d_product = d.pow(3) / (amount_a_times_coins * amount_b_times_coins);
        d_previous = d;
        d = calculate_step(d, leverage, sum_x, d_product);
        // Equality with the precision of 1e-6
        if (d - d_previous).abs() <= TOL {
            return d;
        }
    }

    panic!("Newton method for D failed to converge");
}

/// Helper function used to calculate the D invariant as a last step in the `compute_d` public function.
///
/// * **Equation**:
///
/// d = (leverage * sum_x + d_product * n_coins) * initial_d / ((leverage - 1) * initial_d + (n_coins + 1) * d_product)
fn calculate_step(
    initial_d: Decimal,
    leverage: Decimal,
    sum_x: Decimal,
    d_product: Decimal,
) -> Decimal {
    let leverage_mul = leverage * sum_x;
    let d_p_mul = d_product * N_COINS;

    let l_val = leverage_mul + d_p_mul * initial_d;
    let leverage_sub = initial_d * (leverage - Decimal::one());
    let n_coins_sum = d_product * (N_COINS + Decimal::one());

    let r_val = leverage_sub + n_coins_sum;

    l_val / r_val
}

/// Compute the swap amount `y` in proportion to `x`.
///
/// * **Solve for y**
///
/// y**2 + y * (sum' - (A*n**n - 1) * D / (A * n**n)) = D ** (n + 1) / (n ** (2 * n) * prod' * A)
///
/// y**2 + b*y = c
pub(crate) fn calc_y(amp: u128, new_amount: Decimal, xp: &[Decimal], target_precision: u8) -> i128 {
    let d = compute_d(amp, xp);
    let leverage = Decimal::from_ratio(amp as i128, 1u8) * N_COINS;
    let amp_prec = Decimal::from_ratio(AMP_PRECISION, 1u8);

    let c = d.pow(3) * amp_prec / (new_amount * N_COINS * N_COINS * leverage);
    let b = new_amount + d * amp_prec / leverage;

    // Solve for y by approximating: y**2 + b*y = c
    let mut y_prev;
    let mut y = d;
    for _ in 0..ITERATIONS {
        y_prev = y;
        y = (y.pow(2) + c) / (y * N_COINS + b - d);
        if (y - y_prev).abs() <= TOL {
            return y.to_i128_with_precision(target_precision);
        }
    }

    // Should definitely converge in 64 iterations.
    panic!("y is not converging");
}
