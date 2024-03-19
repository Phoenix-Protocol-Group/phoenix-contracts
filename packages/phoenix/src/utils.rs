use decimal::Decimal;
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

pub fn is_approx_ratio(a: Decimal, b: Decimal, tolerance: Decimal) -> bool {
    let diff = (a - b).abs();
    diff <= tolerance
}

#[macro_export]
macro_rules! assert_approx_eq {
    ($a:expr, $b:expr) => {{
        let eps = 1.0e-6;
        let (a, b) = (&$a, &$b);
        assert!(
            (*a - *b).abs() < eps,
            "assertion failed: `(left !== right)` \
             (left: `{:?}`, right: `{:?}`, expect diff: `{:?}`, real diff: `{:?}`)",
            *a,
            *b,
            eps,
            (*a - *b).abs()
        );
    }};
    ($a:expr, $b:expr, $eps:expr) => {{
        let (a, b) = (&$a, &$b);
        let eps = $eps;
        assert!(
            (*a - *b).abs() < eps,
            "assertion failed: `(left !== right)` \
             (left: `{:?}`, right: `{:?}`, expect diff: `{:?}`, real diff: `{:?}`)",
            *a,
            *b,
            eps,
            (*a - *b).abs()
        );
    }};
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

#[cfg(test)]
mod tests {
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
}
