use soroban_sdk::{log, panic_with_error, Env, U256};

use crate::{error::ContractError, storage::AmplifierParameters, DECIMAL_PRECISION};
use soroban_decimal::Decimal256;

// TODO: Those parameters will be used for updating AMP function later
#[allow(dead_code)]
pub const MAX_AMP_CHANGE: u64 = 10;
#[allow(dead_code)]
pub const MIN_AMP_CHANGING_TIME: u64 = 86400;
pub const AMP_PRECISION: u64 = 100;
/// The maximum number of calculation steps for Newton's method.
const ITERATIONS: u8 = 64;

fn abs_diff(a: &Decimal256, b: &Decimal256) -> Decimal256 {
    if a < b {
        b.clone() - a.clone()
    } else {
        a.clone() - b.clone()
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
pub fn compute_d(env: &Env, amp: u128, pools: &[Decimal256]) -> Decimal256 {
    let tol = Decimal256::raw(U256::from_u128(env, 1000000000000));

    let n_coins = Decimal256::raw(U256::from_u128(env, 2000000000000000000));
    let leverage = Decimal256::from_ratio(
        env,
        U256::from_u128(env, amp),
        U256::from_u128(env, AMP_PRECISION as u128),
    )
    .mul(env, &n_coins);
    // let leverage = U256::from_u128(env, (amp / AMP_PRECISION as u128) * N_COINS_PRECISION);
    let amount_a_times_coins = pools[0].mul(env, &n_coins);
    let amount_b_times_coins = pools[1].mul(env, &n_coins);

    // because of copy trait
    let pool_idx_zero = pools[0].clone();
    let pool_idx_one = pools[1].clone();
    let sum_x = pool_idx_zero + pool_idx_one; // sum(x_i), a.k.a S
    if sum_x.eq(&Decimal256::zero(env)) {
        return Decimal256::zero(env);
    }

    let mut d_previous: Decimal256;
    let mut d: Decimal256 = sum_x.clone();

    // Newton's method to approximate D
    for _ in 0..ITERATIONS {
        let d_product = d
            .clone()
            .pow(env, 3)
            .div(env, amount_a_times_coins.mul(env, &amount_b_times_coins));
        d_previous = d.clone();
        d = calculate_step(env, &d, &leverage, &sum_x, &d_product);
        // Equality with the precision of 1e-6
        if abs_diff(&d, &d_previous) <= tol {
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
    initial_d: &Decimal256,
    leverage: &Decimal256,
    sum_x: &Decimal256,
    d_product: &Decimal256,
) -> Decimal256 {
    // (leverage * sum_x + d_product * n_coins)
    let leverage_mul = leverage.mul(env, sum_x);
    let n_coins = Decimal256::raw(U256::from_u128(env, 2000000000000000000));
    let d_p_mul = d_product.mul(env, &n_coins);

    let l_val = leverage_mul + (d_p_mul).mul(env, initial_d);

    // initial_d * (leverage - 1)
    let leverage_sub = initial_d.mul(env, &(leverage.clone() - (Decimal256::one(env))));

    // d_product * (n_coins + 1)
    let n_coins_sum = d_product.mul(env, &(n_coins + Decimal256::one(env)));

    // Calculate the final step value
    let r_val = leverage_sub + n_coins_sum;
    l_val.div(env, r_val)
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
    new_amount: Decimal256,
    xp: &[Decimal256],
    target_precision: u32,
) -> u128 {
    soroban_sdk::testutils::arbitrary::std::dbg!("START CALC_Y");
    let n_coins = Decimal256::raw(U256::from_u128(env, 2000000000000000000));
    let tol = Decimal256::raw(U256::from_u128(env, 1000000000000));

    let d = compute_d(env, amp, xp);
    let leverage =
        Decimal256::from_ratio(env, U256::from_u128(env, amp), U256::from_u128(env, 1u128))
            .mul(env, &n_coins);
    let amp_prec = Decimal256::from_ratio(
        env,
        U256::from_u128(env, AMP_PRECISION as u128),
        U256::from_u128(env, 1u128),
    );

    let c = d.clone().pow(env, 3).mul(env, &amp_prec).div(
        env,
        new_amount
            .mul(env, &(n_coins.mul(env, &n_coins)))
            .mul(env, &leverage),
    );

    let b = new_amount + (d.mul(env, &amp_prec).div(env, leverage));

    // Solve for y by approximating: y**2 + b*y = c
    let mut y_prev;
    let mut y = d.clone();
    for _ in 0..ITERATIONS {
        y_prev = y.clone();
        y = (y.clone().pow(env, 2) + c.clone())
            .div(env, y.mul(env, &n_coins) + b.clone() - d.clone());

        soroban_sdk::testutils::arbitrary::std::dbg!(
            y.clone().to_u128_with_precision(DECIMAL_PRECISION as i32),
            y_prev
                .clone()
                .to_u128_with_precision(DECIMAL_PRECISION as i32),
            tol.to_u128_with_precision(DECIMAL_PRECISION as i32),
        );
        if abs_diff(&y, &y_prev) <= tol {
            let divisor = 10u128.pow(DECIMAL_PRECISION - target_precision);
            return y.to_u128_with_precision(target_precision as i32) / divisor;
        }
    }

    soroban_sdk::testutils::arbitrary::std::dbg!();
    // Should definitely converge in 64 iterations.
    log!(&env, "Pool Stable: calc_y: y is not converging");
    panic_with_error!(&env, ContractError::CalcYErr);
}
