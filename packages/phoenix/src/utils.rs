use soroban_decimal::Decimal;
use soroban_sdk::{contracttype, Address};

// Validate if int value is bigger then 0
#[macro_export]
macro_rules! validate_int_parameters {
    ($($arg:expr),*) => {
        {
            $(
                let value: Option<i128> = Into::<Option<_>>::into($arg);
                if let Some(val) = value {
                    if val <= 0 {
                        panic!("value cannot be less than or equal zero")
                    }
                }
            )*
        }
    };
}

// Validate all bps to be between the range 0..10_000
#[macro_export]
macro_rules! validate_bps {
    ($($value:expr),+) => {
        const MIN_BPS: i64 = 0;
        const MAX_BPS: i64 = 10_000;
        $(
            // if $value < MIN_BPS || $value > MAX_BPS {
            //     panic!("The value {} is out of range. Must be between {} and {} bps.", $value, MIN_BPS, MAX_BPS);
            // }
            assert!((MIN_BPS..=MAX_BPS).contains(&$value), "The value {} is out of range. Must be between {} and {} bps.", $value, MIN_BPS, MAX_BPS);
        )+
    }
}

/// Validate if the current timestamp is within desired timestamp
#[macro_export]
macro_rules! ensure_not_expired {
    ($env:expr, $expiration_timestamp:expr) => {
        if $env.ledger().timestamp() >= $expiration_timestamp {
            panic!("Transaction has expired")
        }
    };
}

pub fn is_approx_ratio(a: Decimal, b: Decimal, tolerance: Decimal) -> bool {
    let diff = (a - b).abs();
    diff <= tolerance
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenInitInfo {
    pub token_a: Address,
    pub token_b: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakeInitInfo {
    pub min_bond: i128,
    pub min_reward: i128,
    pub manager: Address,
    pub max_complexity: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquidityPoolInitInfo {
    pub admin: Address,
    pub swap_fee_bps: i64,
    pub fee_recipient: Address,
    pub max_allowed_slippage_bps: i64,
    pub max_allowed_spread_bps: i64,
    pub max_referral_bps: i64,
    pub token_init_info: TokenInitInfo,
    pub stake_init_info: StakeInitInfo,
}

#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PoolType {
    Xyk = 0,
    Stable = 1,
}

#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Ledger, Env};

    use super::*;

    #[test]
    fn test_validate_int_parameters() {
        // The macro should not panic for valid parameters.
        validate_int_parameters!(1, 2, 3);
        validate_int_parameters!(1, 1, 1);
        validate_int_parameters!(1i128, 2i128, 3i128, Some(4i128), None::<i128>);
        validate_int_parameters!(None::<i128>, None::<i128>);
        validate_int_parameters!(Some(1i128), None::<i128>);
    }

    #[test]
    #[should_panic]
    fn should_panic_when_value_less_than_zero() {
        validate_int_parameters!(1, -2, 3);
    }

    #[test]
    #[should_panic]
    fn should_panic_when_first_value_equal_zero() {
        validate_int_parameters!(0, 1, 3);
    }

    #[test]
    #[should_panic]
    fn should_panic_when_last_value_equal_zero() {
        validate_int_parameters!(1, 1, 0);
    }

    #[test]
    #[should_panic]
    fn should_panic_when_some_equals_zero() {
        validate_int_parameters!(Some(0i128), None::<i128>);
    }

    #[test]
    #[should_panic]
    fn should_panic_when_some_less_than_zero() {
        validate_int_parameters!(Some(-1i128), None::<i128>);
    }

    #[test]
    fn test_assert_approx_ratio_close_values() {
        let a = Decimal::from_ratio(100, 101);
        let b = Decimal::from_ratio(100, 100);
        let tolerance = Decimal::percent(3);
        assert!(is_approx_ratio(a, b, tolerance));
    }

    #[test]
    fn test_assert_approx_ratio_equal_values() {
        let a = Decimal::from_ratio(100, 100);
        let b = Decimal::from_ratio(100, 100);
        let tolerance = Decimal::percent(3);
        assert!(is_approx_ratio(a, b, tolerance));
    }

    #[test]
    fn test_assert_approx_ratio_outside_tolerance() {
        let a = Decimal::from_ratio(100, 104);
        let b = Decimal::from_ratio(100, 100);
        let tolerance = Decimal::percent(3);
        assert!(!is_approx_ratio(a, b, tolerance));
    }

    #[test]
    #[should_panic(expected = "The value -1 is out of range. Must be between 0 and 10000 bps.")]
    fn validate_bps_below_min() {
        validate_bps!(-1, 300, 5_000, 8_534);
    }

    #[test]
    #[should_panic(expected = "The value 10001 is out of range. Must be between 0 and 10000 bps.")]
    fn validate_bps_above_max() {
        validate_bps!(100, 10_001, 31_3134, 348);
    }

    #[test]
    fn bps_valid_range() {
        validate_bps!(0, 5_000, 7_500, 10_000);
    }

    #[test]
    #[should_panic(expected = "Transaction has expired")]
    fn ensure_not_expired_should_panic_when_desired_is_after_current() {
        let env = Env::default();
        env.ledger().with_mut(|li| li.timestamp = 100);
        ensure_not_expired!(env, 99);
    }

    #[test]
    fn ensure_not_expired_should_work_fine() {
        let env = Env::default();
        env.ledger().with_mut(|li| li.timestamp = 100);
        ensure_not_expired!(env, 101);
    }
}
