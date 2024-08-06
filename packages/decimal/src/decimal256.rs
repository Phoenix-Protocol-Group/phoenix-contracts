// A lot of this code is taken from the cosmwasm-std crate, which is licensed under the Apache
// License 2.0 - https://github.com/CosmWasm/cosmwasm.

use soroban_sdk::{Env, U256};

use core::{
    cmp::{Ordering, PartialEq, PartialOrd},
    ops::{Add, Sub},
};

extern crate alloc;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
enum Error {
    DivideByZero,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Decimal256(U256);

#[allow(dead_code)]
impl Decimal256 {
    const DECIMAL_PLACES: i32 = 18;

    pub fn new(env: &Env, value: u128) -> Self {
        Decimal256(U256::from_u128(env, value))
    }

    pub fn raw(value: U256) -> Self {
        Self(value)
    }

    pub fn decimal_fractional(env: &Env) -> U256 {
        U256::from_u128(env, 1_000_000_000_000_000_000u128) // 1*10**18
    }

    pub fn one(env: &Env) -> Self {
        Self(U256::from_u128(env, 1_000_000_000_000_000_000))
    }

    pub fn zero(env: &Env) -> Self {
        Self(U256::from_u128(env, 0))
    }

    pub fn max(env: &Env) -> Self {
        Decimal256(U256::from_parts(
            env,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
        ))
    }

    pub fn percent(env: &Env, x: u64) -> Self {
        Self(U256::from_u128(env, (x as u128) * 10_000_000_000_000_000))
    }

    pub fn permille(env: &Env, x: u64) -> Self {
        Self(U256::from_u128(env, (x as u128) * 1_000_000_000_000_000))
    }

    pub fn bps(env: &Env, x: u64) -> Self {
        Self(U256::from_u128(env, (x as u128) * 100_000_000_000_000))
    }

    pub fn decimal_places(&self) -> i32 {
        Self::DECIMAL_PLACES
    }

    fn numerator(&self) -> U256 {
        self.0.clone()
    }

    fn denominator(&self, env: &Env) -> U256 {
        U256::from_u128(env, 1_000_000_000_000_000_000)
    }

    pub fn is_zero(&self, env: &Env) -> bool {
        self.0 == U256::from_u128(env, 0)
    }

    pub fn atomics(&self) -> Option<u128> {
        self.0.to_u128()
    }

    // TODO: Allow for `decimal_places` larger than 38
    pub fn from_atomics(env: &Env, atomics: u128, decimal_places: i32) -> Self {
        let ten: U256 = U256::from_u128(env, 10u128);
        let atomics = U256::from_u128(env, atomics);
        match decimal_places.cmp(&Self::DECIMAL_PLACES) {
            Ordering::Less => {
                let digits = Self::DECIMAL_PLACES - decimal_places;
                let factor = ten.pow(digits as u32);
                Self(atomics.mul(&factor))
            }
            Ordering::Equal => Self(atomics),
            Ordering::Greater => {
                let digits = decimal_places - Self::DECIMAL_PLACES;
                let factor = ten.pow(digits as u32);
                Self(atomics.div(&factor))
            }
        }
    }

    pub fn pow(self, env: &Env, exp: u32) -> Self {
        fn inner(env: &Env, mut x: Decimal256, mut n: u32) -> Decimal256 {
            if n == 0 {
                return Decimal256::one(env);
            }

            let mut y = Decimal256::one(env);

            while n > 1 {
                if n % 2 == 0 {
                    x = x.clone().mul(env, &x);
                    n /= 2;
                } else {
                    y = x.clone().mul(env, &y);
                    x = x.clone().mul(env, &x);
                    n = (n - 1) / 2;
                }
            }

            x.mul(env, &y)
        }

        inner(env, self, exp)
    }

    pub fn inv(&self, env: &Env) -> Option<Self> {
        if self.is_zero(env) {
            None
        } else {
            let fractional_squared =
                U256::from_u128(env, 1_000_000_000_000_000_000_000_000_000_000_000_000);
            Some(Decimal256(fractional_squared.div(&self.0)))
        }
    }

    pub fn from_ratio(env: &Env, numerator: impl Into<U256>, denominator: impl Into<U256>) -> Self {
        match Decimal256::checked_from_ratio(env, numerator, denominator) {
            Ok(ratio) => ratio,
            Err(Error::DivideByZero) => panic!("Denominator must not be zero"),
        }
    }

    pub fn to_u128_with_precision(&self, precision: impl Into<i32>) -> u128 {
        let value = self.atomics().unwrap();
        let precision = precision.into();

        let divisor = 10u128.pow((self.decimal_places() - precision) as u32);
        value / divisor
    }

    fn multiply_ratio(
        &self,
        env: &Env,
        numerator: Decimal256,
        denominator: Decimal256,
    ) -> Decimal256 {
        Decimal256::from_ratio(env, self.0.mul(&numerator.0), denominator.0)
    }

    fn checked_from_ratio(
        env: &Env,
        numerator: impl Into<U256>,
        denominator: impl Into<U256>,
    ) -> Result<Self, Error> {
        let numerator = numerator.into();
        let denominator = denominator.into();

        if denominator == U256::from_u128(env, 0) {
            return Err(Error::DivideByZero);
        }

        let ratio = numerator
            .mul(&U256::from_u128(env, 1_000_000_000_000_000_000))
            .div(&denominator);
        Ok(Decimal256(ratio))
    }

    pub fn abs_diff(self, env: &Env, other: Self) -> Self {
        let diff = self
            .0
            .to_u128()
            .unwrap()
            .abs_diff(other.0.to_u128().unwrap());
        Self(U256::from_u128(env, diff))
    }

    pub fn div_by_u256(&self, rhs: U256) -> Self {
        Decimal256(self.0.div(&rhs))
    }
}

impl Add for Decimal256 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Decimal256(self.0.add(&other.0))
    }
}

impl Sub for Decimal256 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Decimal256(self.0.sub(&other.0))
    }
}

impl Decimal256 {
    pub fn mul(&self, env: &Env, other: &Self) -> Self {
        let result = self
            .numerator()
            .mul(&other.numerator())
            .div(&U256::from_u128(env, 1_000_000_000_000_000_000));
        Decimal256(result)
    }

    pub fn mul_u128(&self, env: &Env, other: u128) -> U256 {
        if self == &Decimal256::zero(env) || other == 0u128 {
            return U256::from_u128(env, 0u128);
        }
        let other = U256::from_u128(env, other);
        other
            .mul(&self.0)
            .div(&U256::from_u128(env, 1_000_000_000_000_000_000))
    }

    #[allow(dead_code)]
    pub fn div(&self, env: &Env, rhs: Self) -> Self {
        match Decimal256::checked_from_ratio(env, self.numerator(), rhs.numerator()) {
            Ok(ratio) => ratio,
            Err(Error::DivideByZero) => panic!("Division failed - denominator must not be zero"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal256_new() {
        let env = Env::default();
        let expected = 300u128;
        assert_eq!(
            Decimal256::new(&env, expected).0.to_u128().unwrap(),
            expected
        );
    }

    #[test]
    fn decimal256_raw() {
        let env = Env::default();
        let value = 300u128;
        assert_eq!(
            Decimal256::raw(U256::from_u128(&env, value))
                .0
                .to_u128()
                .unwrap(),
            value
        );
    }

    #[test]
    fn decimal256_one() {
        let env = Env::default();
        let value = Decimal256::one(&env);
        assert_eq!(value.0.to_u128().unwrap(), 1_000_000_000_000_000_000);
    }

    #[test]
    fn decimal256_zero() {
        let env = Env::default();
        let value = Decimal256::zero(&env);
        assert_eq!(value.0.to_u128().unwrap(), 0);
    }

    #[test]
    fn decimal256_percent() {
        let env = Env::default();
        let value = Decimal256::percent(&env, 50);
        assert_eq!(value.0.to_u128().unwrap(), 500_000_000_000_000_000);
    }

    #[test]
    fn decimal256_from_atomics_works() {
        let env = Env::default();
        let one = Decimal256::one(&env);
        let two = Decimal256::new(&env, 2 * 1_000_000_000_000_000_000);

        assert_eq!(Decimal256::from_atomics(&env, 1, 0), one);
        assert_eq!(Decimal256::from_atomics(&env, 10, 1), one);
        assert_eq!(Decimal256::from_atomics(&env, 100, 2), one);
        assert_eq!(Decimal256::from_atomics(&env, 1000, 3), one);
        assert_eq!(
            Decimal256::from_atomics(&env, 1_000_000_000_000_000_000, 18),
            one
        );
        assert_eq!(
            Decimal256::from_atomics(&env, 10_000_000_000_000_000_000, 19),
            one
        );
        assert_eq!(
            Decimal256::from_atomics(&env, 100_000_000_000_000_000_000, 20),
            one
        );

        assert_eq!(Decimal256::from_atomics(&env, 2, 0), two);
        assert_eq!(Decimal256::from_atomics(&env, 20, 1), two);
        assert_eq!(Decimal256::from_atomics(&env, 200, 2), two);
        assert_eq!(Decimal256::from_atomics(&env, 2000, 3), two);
        assert_eq!(
            Decimal256::from_atomics(&env, 2_000_000_000_000_000_000, 18),
            two
        );
        assert_eq!(
            Decimal256::from_atomics(&env, 20_000_000_000_000_000_000, 19),
            two
        );
        assert_eq!(
            Decimal256::from_atomics(&env, 200_000_000_000_000_000_000, 20),
            two
        );

        // Cuts decimal digits (20 provided but only 18 can be stored)
        assert_eq!(
            Decimal256::from_atomics(&env, 4321, 20),
            Decimal256::from_ratio(
                &env,
                U256::from_u128(&env, 43),
                U256::from_u128(&env, 1000000000000000000)
            ),
        );
        assert_eq!(
            Decimal256::from_atomics(&env, 6789, 20),
            Decimal256::from_ratio(
                &env,
                U256::from_u128(&env, 67),
                U256::from_u128(&env, 1000000000000000000)
            ),
        );
        assert_eq!(
            // 340282366920938463463374607431768211455 / 10000000000000000000 (10^19) = 3.40282366920938463463374607431768211455
            Decimal256::from_atomics(&env, u128::MAX, 37),
            Decimal256::from_ratio(
                &env,
                U256::from_u128(&env, 340282366920938463463374607431768211455),
                U256::from_u128(&env, 10000000000000000000000000000000000000)
            ),
        );
        assert_eq!(
            // 340282366920938463463374607431768211455 / 100000000000000000000 (10^20) = 3.40282366920938463463374607431768211455
            Decimal256::from_atomics(&env, u128::MAX, 38),
            Decimal256::from_ratio(
                &env,
                U256::from_u128(&env, 340282366920938463463374607431768211455),
                U256::from_u128(&env, 100000000000000000000000000000000000000)
            ),
        );
        // TODO: we can handle up to 38 `decimal_places` as input in `from_atomics`:w
        //assert_eq!(
        //    // 340282366920938463463374607431768211455 / 1000000000000000000000 (10^21) = 340282366920.938463463374607432
        //    Decimal256::from_atomics(&env, u128::MAX, 39),
        //    Decimal256::from_ratio(
        //        &env,
        //        U256::from_u128(&env, 340282366920938463463374607432),
        //        U256::from_u128(&env, 1000000000000000000)
        //    )
        //);
        //assert_eq!(
        //    // 340282366920938463463374607431768211455 / 1000000000000000000000000000 (10^27) = ?
        //    Decimal256::from_atomics(&env, u128::MAX, 45),
        //    Decimal256::from_ratio(
        //        &env,
        //        U256::from_u128(&env, 67),
        //        U256::from_u128(&env, 1000000000000000000)
        //    ),
        //);
        //assert_eq!(
        //    // 340282366920938463463374607431768211455 / 1000000000000000000000000000000000 (10^33) = ?
        //    Decimal256::from_atomics(&env, u128::MAX, 51),
        //    Decimal256::from_ratio(
        //        &env,
        //        U256::from_u128(&env, 67),
        //        U256::from_u128(&env, 1000000000000000000)
        //    ),
        //);
        //assert_eq!(
        //    Decimal256::from_atomics(&env, u128::MAX, 56),
        //    Decimal256::from_ratio(
        //        &env,
        //        U256::from_u128(&env, 67),
        //        U256::from_u128(&env, 1000000000000000000)
        //    ),
        //);
    }

    #[test]
    fn decimal256_from_ratio_works() {
        let env = Env::default();

        // 1.0
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 1), U256::from_u128(&env, 1)),
            Decimal256::one(&env)
        );
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 53), U256::from_u128(&env, 53)),
            Decimal256::one(&env)
        );
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 125), U256::from_u128(&env, 125)),
            Decimal256::one(&env)
        );

        // 1.5
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 3), U256::from_u128(&env, 2)),
            Decimal256::percent(&env, 150)
        );
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 150), U256::from_u128(&env, 100)),
            Decimal256::percent(&env, 150)
        );
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 333), U256::from_u128(&env, 222)),
            Decimal256::percent(&env, 150)
        );

        // 0.125
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 1), U256::from_u128(&env, 8)),
            Decimal256::permille(&env, 125)
        );
        assert_eq!(
            Decimal256::from_ratio(
                &env,
                U256::from_u128(&env, 125),
                U256::from_u128(&env, 1000)
            ),
            Decimal256::permille(&env, 125)
        );

        // 1/3 (result floored)
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 1), U256::from_u128(&env, 3)),
            Decimal256(U256::from_u128(&env, 333_333_333_333_333_333))
        );

        // 2/3 (result floored)
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 2), U256::from_u128(&env, 3)),
            Decimal256(U256::from_u128(&env, 666_666_666_666_666_666))
        );

        // large inputs
        assert_eq!(
            Decimal256::from_ratio(
                &env,
                U256::from_u128(&env, 0),
                U256::from_u128(&env, u128::MAX)
            ),
            Decimal256::zero(&env)
        );
        assert_eq!(
            Decimal256::from_ratio(
                &env,
                U256::from_u128(&env, u128::MAX),
                U256::from_u128(&env, u128::MAX)
            ),
            Decimal256::one(&env)
        );

        // due to limited possibilities - we're only allowed to use i128 as input - maximum
        // number this implementation supports without overflow is u128 / decimal256_FRACTIONAL
        // 340282366920938463463374607431768211455 / 10^18 is approximately 340282366920938.
        assert_eq!(
            Decimal256::from_ratio(
                &env,
                U256::from_u128(&env, 340282366920938),
                U256::from_u128(&env, 340282366920938)
            ),
            Decimal256::one(&env)
        );
        // This works because of similar orders of magnitude
        assert_eq!(
            Decimal256::from_ratio(
                &env,
                U256::from_u128(&env, 34028236692093900000),
                U256::from_u128(&env, 34028236692093900000)
            ),
            Decimal256::one(&env)
        );
        assert_eq!(
            Decimal256::from_ratio(
                &env,
                U256::from_u128(&env, 34028236692093900000),
                U256::from_u128(&env, 1)
            ),
            Decimal256::new(&env, 34028236692093900000 * 1_000_000_000_000_000_000)
        );
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn decimal256_from_ratio_panics_for_zero_denominator() {
        let env = Env::default();
        Decimal256::from_ratio(&env, U256::from_u128(&env, 1), U256::from_u128(&env, 0));
    }

    #[ignore = "FIXME: Why is U256 not panicking?"]
    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn decimal256_from_ratio_panics_for_mul_overflow() {
        let env = Env::default();
        Decimal256::from_ratio(
            &env,
            U256::from_u128(&env, u128::MAX),
            U256::from_u128(&env, 1),
        );
    }

    #[test]
    fn decimal256_decimal256_places_works() {
        let env = Env::default();
        let zero = Decimal256::zero(&env);
        let one = Decimal256::one(&env);
        let half = Decimal256::percent(&env, 50);
        let two = Decimal256::new(&env, 2 * 1_000_000_000_000_000_000);
        let max = Decimal256::max(&env);

        assert_eq!(zero.decimal_places(), 18);
        assert_eq!(one.decimal_places(), 18);
        assert_eq!(half.decimal_places(), 18);
        assert_eq!(two.decimal_places(), 18);
        assert_eq!(max.decimal_places(), 18);
    }

    #[test]
    fn decimal256_from_fraction_compared_to_percent() {
        let env = Env::default();

        // Integers
        assert_eq!(Decimal256::zero(&env), Decimal256::percent(&env, 0));
        assert_eq!(Decimal256::one(&env), Decimal256::percent(&env, 100));
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 5), U256::from_u128(&env, 1)),
            Decimal256::percent(&env, 500)
        );
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 42), U256::from_u128(&env, 1)),
            Decimal256::percent(&env, 4200)
        );
    }

    #[test]
    fn decimal256_is_zero_works() {
        let env = Env::default();
        assert!(Decimal256::zero(&env).is_zero(&env));
        assert!(Decimal256::percent(&env, 0).is_zero(&env));
        assert!(Decimal256::permille(&env, 0).is_zero(&env));

        assert!(!Decimal256::one(&env).is_zero(&env));
        assert!(!Decimal256::percent(&env, 123).is_zero(&env));
        assert!(!Decimal256::permille(&env, 1234).is_zero(&env));
    }

    #[test]
    fn decimal256_inv_works() {
        let env = Env::default();

        // d = 0
        assert_eq!(Decimal256::zero(&env).inv(&env), None);

        // d == 1
        assert_eq!(Decimal256::one(&env).inv(&env), Some(Decimal256::one(&env)));

        // d > 1 exact
        assert_eq!(
            Decimal256::percent(&env, 200).inv(&env),
            Some(Decimal256::percent(&env, 50))
        );
        assert_eq!(
            Decimal256::percent(&env, 2_000).inv(&env),
            Some(Decimal256::percent(&env, 5))
        );
        assert_eq!(
            Decimal256::percent(&env, 20_000).inv(&env),
            Some(Decimal256::permille(&env, 5))
        );
        assert_eq!(
            Decimal256::percent(&env, 200_000).inv(&env),
            Some(Decimal256::bps(&env, 5))
        );

        // d > 1 rounded
        assert_eq!(
            Decimal256::percent(&env, 300).inv(&env),
            Some(Decimal256::from_ratio(
                &env,
                U256::from_u128(&env, 1),
                U256::from_u128(&env, 3)
            ))
        );
        assert_eq!(
            Decimal256::percent(&env, 600).inv(&env),
            Some(Decimal256::from_ratio(
                &env,
                U256::from_u128(&env, 1),
                U256::from_u128(&env, 6)
            ))
        );

        // d < 1 exact
        assert_eq!(
            Decimal256::percent(&env, 50).inv(&env),
            Some(Decimal256::percent(&env, 200))
        );
        assert_eq!(
            Decimal256::percent(&env, 5).inv(&env),
            Some(Decimal256::percent(&env, 2_000))
        );
        assert_eq!(
            Decimal256::permille(&env, 5).inv(&env),
            Some(Decimal256::percent(&env, 20_000))
        );
        assert_eq!(
            Decimal256::bps(&env, 5).inv(&env),
            Some(Decimal256::percent(&env, 200_000))
        );
    }

    #[test]
    fn decimal256_add_works() {
        let env = Env::default();

        let value = Decimal256::one(&env) + Decimal256::percent(&env, 50); // 1.5
        assert_eq!(value.0.to_u128().unwrap(), 1_500_000_000_000_000_000);

        assert_eq!(
            Decimal256::percent(&env, 5) + Decimal256::percent(&env, 4),
            Decimal256::percent(&env, 9)
        );
        assert_eq!(
            Decimal256::percent(&env, 5) + Decimal256::zero(&env),
            Decimal256::percent(&env, 5)
        );
        assert_eq!(
            Decimal256::zero(&env) + Decimal256::zero(&env),
            Decimal256::zero(&env)
        );
    }

    #[test]
    // #[should_panic(expected = "attempt to add with overflow")]
    // FIXME: Add proper panics
    #[should_panic(expected = "Error(Object, ArithDomain)")]
    fn decimal256_add_overflow_panics() {
        let env = Env::default();
        let _ = Decimal256::max(&env) + Decimal256::percent(&env, 50);
    }

    #[test]
    fn decimal256_sub_works() {
        let env = Env::default();

        let value = Decimal256::one(&env) - Decimal256::percent(&env, 50); // 0.5
        assert_eq!(value.0.to_u128().unwrap(), 500_000_000_000_000_000);

        assert_eq!(
            Decimal256::percent(&env, 9) - Decimal256::percent(&env, 4),
            Decimal256::percent(&env, 5)
        );
        assert_eq!(
            Decimal256::percent(&env, 16) - Decimal256::zero(&env),
            Decimal256::percent(&env, 16)
        );
        assert_eq!(
            Decimal256::percent(&env, 16) - Decimal256::percent(&env, 16),
            Decimal256::zero(&env)
        );
        assert_eq!(
            Decimal256::zero(&env) - Decimal256::zero(&env),
            Decimal256::zero(&env)
        );
    }

    #[test]
    fn decimal256_implements_mul() {
        let env = Env::default();
        let one = Decimal256::one(&env);
        let two = Decimal256::new(&env, 2 * 1_000_000_000_000_000_000);
        let half = Decimal256::percent(&env, 50);

        // 1*x and x*1
        assert_eq!(
            one.clone().mul(&env, &Decimal256::percent(&env, 0)),
            Decimal256::percent(&env, 0)
        );
        assert_eq!(
            one.clone().mul(&env, &Decimal256::percent(&env, 1)),
            Decimal256::percent(&env, 1)
        );
        assert_eq!(
            one.clone().mul(&env, &Decimal256::percent(&env, 10)),
            Decimal256::percent(&env, 10)
        );
        assert_eq!(
            one.clone().mul(&env, &Decimal256::percent(&env, 100)),
            Decimal256::percent(&env, 100)
        );
        assert_eq!(
            one.clone().mul(&env, &Decimal256::percent(&env, 1000)),
            Decimal256::percent(&env, 1000)
        );
        assert_eq!(
            Decimal256::percent(&env, 0).mul(&env, &one),
            Decimal256::percent(&env, 0)
        );
        assert_eq!(
            Decimal256::percent(&env, 1).mul(&env, &one),
            Decimal256::percent(&env, 1)
        );
        assert_eq!(
            Decimal256::percent(&env, 10).mul(&env, &one),
            Decimal256::percent(&env, 10)
        );
        assert_eq!(
            Decimal256::percent(&env, 100).mul(&env, &one),
            Decimal256::percent(&env, 100)
        );
        assert_eq!(
            Decimal256::percent(&env, 1000).mul(&env, &one),
            Decimal256::percent(&env, 1000)
        );

        // double
        assert_eq!(
            two.clone().mul(&env, &Decimal256::percent(&env, 0)),
            Decimal256::percent(&env, 0)
        );
        assert_eq!(
            two.clone().mul(&env, &Decimal256::percent(&env, 1)),
            Decimal256::percent(&env, 2)
        );
        assert_eq!(
            two.clone().mul(&env, &Decimal256::percent(&env, 10)),
            Decimal256::percent(&env, 20)
        );
        assert_eq!(
            two.clone().mul(&env, &Decimal256::percent(&env, 100)),
            Decimal256::percent(&env, 200)
        );
        assert_eq!(
            two.clone().mul(&env, &Decimal256::percent(&env, 1000)),
            Decimal256::percent(&env, 2000)
        );
        assert_eq!(
            Decimal256::percent(&env, 0).mul(&env, &two),
            Decimal256::percent(&env, 0)
        );
        assert_eq!(
            Decimal256::percent(&env, 1).mul(&env, &two),
            Decimal256::percent(&env, 2)
        );
        assert_eq!(
            Decimal256::percent(&env, 10).mul(&env, &two),
            Decimal256::percent(&env, 20)
        );
        assert_eq!(
            Decimal256::percent(&env, 100).mul(&env, &two),
            Decimal256::percent(&env, 200)
        );
        assert_eq!(
            Decimal256::percent(&env, 1000).mul(&env, &two),
            Decimal256::percent(&env, 2000)
        );

        // half
        assert_eq!(
            half.clone().mul(&env, &Decimal256::percent(&env, 0)),
            Decimal256::percent(&env, 0)
        );
        assert_eq!(
            half.clone().mul(&env, &Decimal256::percent(&env, 1)),
            Decimal256::permille(&env, 5)
        );
        assert_eq!(
            half.clone().mul(&env, &Decimal256::percent(&env, 10)),
            Decimal256::percent(&env, 5)
        );
        assert_eq!(
            half.clone().mul(&env, &Decimal256::percent(&env, 100)),
            Decimal256::percent(&env, 50)
        );
        assert_eq!(
            half.clone().mul(&env, &Decimal256::percent(&env, 1000)),
            Decimal256::percent(&env, 500)
        );
        assert_eq!(
            Decimal256::percent(&env, 0).mul(&env, &half),
            Decimal256::percent(&env, 0)
        );
        assert_eq!(
            Decimal256::percent(&env, 1).mul(&env, &half),
            Decimal256::permille(&env, 5)
        );
        assert_eq!(
            Decimal256::percent(&env, 10).mul(&env, &half),
            Decimal256::percent(&env, 5)
        );
        assert_eq!(
            Decimal256::percent(&env, 100).mul(&env, &half),
            Decimal256::percent(&env, 50)
        );
        assert_eq!(
            Decimal256::percent(&env, 1000).mul(&env, &half),
            Decimal256::percent(&env, 500)
        );
    }

    #[test]
    // #[should_panic(expected = "attempt to multiply with overflow")]
    // FIXME: Add proper panics
    #[should_panic(expected = "Error(Object, ArithDomain)")]
    fn decimal256_mul_overflow_panics() {
        let env = Env::default();
        let _value = Decimal256::max(&env).mul(&env, &Decimal256::percent(&env, 101));
    }

    #[test]
    fn u128_decimal256_multiply() {
        let env = Env::default();

        // a*b
        let left =
            Decimal256::from_ratio(&env, U256::from_u128(&env, 300), U256::from_u128(&env, 1));
        let right = Decimal256::one(&env) + Decimal256::percent(&env, 50); // 1.5
        assert_eq!(
            left.mul(&env, &right),
            Decimal256::from_ratio(&env, U256::from_u128(&env, 450), U256::from_u128(&env, 1)),
        );

        // a*0
        let left =
            Decimal256::from_ratio(&env, U256::from_u128(&env, 300), U256::from_u128(&env, 1));
        let right = Decimal256::zero(&env);
        assert_eq!(left.mul(&env, &right), Decimal256::zero(&env));

        //// 0*a
        let left = Decimal256::zero(&env);
        let right = Decimal256::one(&env) + Decimal256::percent(&env, 50); // 1.5
        assert_eq!(left.mul(&env, &right), Decimal256::zero(&env));

        assert_eq!(
            Decimal256::zero(&env).mul(&env, &Decimal256::one(&env)),
            Decimal256::zero(&env)
        );
        assert_eq!(
            Decimal256::one(&env).mul(&env, &Decimal256::one(&env)),
            Decimal256::one(&env)
        );
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 2), U256::from_u128(&env, 1))
                .mul(&env, &Decimal256::one(&env)),
            Decimal256::from_ratio(&env, U256::from_u128(&env, 2), U256::from_u128(&env, 1))
        );

        // 1 * %0.1
        assert_eq!(
            Decimal256::one(&env,).mul(&env, &Decimal256::percent(&env, 10)),
            Decimal256::from_ratio(&env, U256::from_u128(&env, 1), U256::from_u128(&env, 10))
        );

        // 10 * %0.1
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 10), U256::from_u128(&env, 1))
                .mul(&env, &Decimal256::percent(&env, 10)),
            Decimal256::one(&env)
        );

        // 100 * %0.1
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 100), U256::from_u128(&env, 1))
                .mul(&env, &Decimal256::percent(&env, 10)),
            Decimal256::from_ratio(&env, U256::from_u128(&env, 10), U256::from_u128(&env, 1))
        );

        // 1 * %0.5
        assert_eq!(
            Decimal256::one(&env).mul(&env, &Decimal256::percent(&env, 50)),
            Decimal256::from_ratio(&env, U256::from_u128(&env, 1), U256::from_u128(&env, 2))
        );

        // 100 * %0.5
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 100), U256::from_u128(&env, 1))
                .mul(&env, &Decimal256::percent(&env, 50)),
            Decimal256::from_ratio(&env, U256::from_u128(&env, 50), U256::from_u128(&env, 1))
        );

        // 3200 * %0.5
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 3_200), U256::from_u128(&env, 1))
                .mul(&env, &Decimal256::percent(&env, 50)),
            Decimal256::from_ratio(&env, U256::from_u128(&env, 1_600), U256::from_u128(&env, 1))
        );

        // 999 * %0.5
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 999), U256::from_u128(&env, 1))
                .mul(&env, &Decimal256::percent(&env, 50)),
            Decimal256::from_ratio(&env, U256::from_u128(&env, 4995), U256::from_u128(&env, 10))
        );

        // 1 * %2
        assert_eq!(
            Decimal256::one(&env).mul(&env, &Decimal256::percent(&env, 200)),
            Decimal256::from_ratio(&env, U256::from_u128(&env, 2), U256::from_u128(&env, 1))
        );

        // 1_000 * %2
        assert_eq!(
            Decimal256::from_ratio(&env, U256::from_u128(&env, 1_000), U256::from_u128(&env, 1))
                .mul(&env, &Decimal256::percent(&env, 200)),
            Decimal256::from_ratio(&env, U256::from_u128(&env, 2_000), U256::from_u128(&env, 1))
        );
    }

    // in this test the Decimal256 is on the left
    #[test]
    fn decimal256_multiplication() {
        let env = Env::default();

        // a*b
        let left = Decimal256::one(&env) + Decimal256::percent(&env, 50); // 1.5
        let right =
            Decimal256::from_ratio(&env, U256::from_u128(&env, 300), U256::from_u128(&env, 1));

        assert_eq!(
            left.mul(&env, &right),
            Decimal256::from_ratio(&env, U256::from_u128(&env, 450), U256::from_u128(&env, 1))
        );

        // 0*a
        let left = Decimal256::zero(&env);
        let right = Decimal256::one(&env) + Decimal256::percent(&env, 50); // 1.5
        assert_eq!(left.mul(&env, &right), Decimal256::zero(&env));

        // a*0
        let left = Decimal256::one(&env) + Decimal256::percent(&env, 50); // 1.5
        let right = Decimal256::zero(&env);
        assert_eq!(left.mul(&env, &right), Decimal256::zero(&env));
    }

    #[test]
    fn decimal256_implements_div() {
        let env = Env::default();
        let one = Decimal256::one(&env);
        let two = Decimal256::new(&env, 2 * 1_000_000_000_000_000_000);
        let half = Decimal256::percent(&env, 50);

        // 1/x and x/1
        // 1 / %0.01
        assert_eq!(
            one.div(&env, Decimal256::percent(&env, 1)),
            Decimal256::percent(&env, 10_000)
        );

        // 1 / %0.1
        assert_eq!(
            one.div(&env, Decimal256::percent(&env, 10)),
            Decimal256::percent(&env, 1_000)
        );

        // 1 / %1
        assert_eq!(
            one.div(&env, Decimal256::percent(&env, 100)),
            Decimal256::one(&env)
        );

        // 1 / %10
        assert_eq!(
            one.div(&env, Decimal256::percent(&env, 1_000)),
            Decimal256::percent(&env, 10)
        );

        // %0 / 1
        assert_eq!(
            Decimal256::percent(&env, 0).div(&env, one.clone()),
            Decimal256::percent(&env, 0)
        );

        // %0.01 / 1
        assert_eq!(
            Decimal256::percent(&env, 1).div(&env, one.clone()),
            Decimal256::percent(&env, 1)
        );

        // %0.1 / 1
        assert_eq!(
            Decimal256::percent(&env, 10).div(&env, one.clone()),
            Decimal256::percent(&env, 10)
        );

        // %1 / 1
        assert_eq!(
            Decimal256::percent(&env, 100).div(&env, one.clone()),
            Decimal256::percent(&env, 100)
        );

        // %100 / 1
        assert_eq!(
            Decimal256::percent(&env, 1_000).div(&env, one.clone()),
            Decimal256::percent(&env, 1_000)
        );

        // 2/x and x/2
        // 2 / %0.01
        assert_eq!(
            two.div(&env, Decimal256::percent(&env, 1)),
            Decimal256::percent(&env, 20_000)
        );

        // 2 / %0.1
        assert_eq!(
            two.div(&env, Decimal256::percent(&env, 10)),
            Decimal256::percent(&env, 2_000)
        );

        // 2 / %1
        assert_eq!(
            two.div(&env, Decimal256::percent(&env, 100)),
            Decimal256::percent(&env, 200)
        );

        // 2 / %10
        assert_eq!(
            two.div(&env, Decimal256::percent(&env, 1_000)),
            Decimal256::percent(&env, 20)
        );

        // %0 / 2
        assert_eq!(
            Decimal256::percent(&env, 0).div(&env, two.clone()),
            Decimal256::percent(&env, 0)
        );

        // %0.1 / 2
        assert_eq!(
            Decimal256::percent(&env, 10).div(&env, two.clone()),
            Decimal256::percent(&env, 5)
        );

        // %1 / 2
        assert_eq!(
            Decimal256::percent(&env, 100).div(&env, two.clone()),
            Decimal256::percent(&env, 50)
        );

        // %10 / 2
        assert_eq!(
            Decimal256::percent(&env, 1_000).div(&env, two.clone()),
            Decimal256::percent(&env, 500)
        );

        // half/x and x/half
        // half / %0.01
        assert_eq!(
            half.div(&env, Decimal256::percent(&env, 1)),
            Decimal256::percent(&env, 5_000)
        );

        // half / %0.1
        assert_eq!(
            half.div(&env, Decimal256::percent(&env, 10)),
            Decimal256::percent(&env, 500)
        );

        // half / %1
        assert_eq!(
            half.div(&env, Decimal256::percent(&env, 100)),
            Decimal256::percent(&env, 50)
        );

        // half / %10
        assert_eq!(
            half.div(&env, Decimal256::percent(&env, 1_000)),
            Decimal256::percent(&env, 5)
        );

        // %0 / half
        assert_eq!(
            Decimal256::percent(&env, 0).div(&env, half.clone()),
            Decimal256::percent(&env, 0)
        );

        // %0.01 / half
        assert_eq!(
            Decimal256::percent(&env, 1).div(&env, half.clone()),
            Decimal256::percent(&env, 2)
        );

        // %0.1 / half
        assert_eq!(
            Decimal256::percent(&env, 10).div(&env, half.clone()),
            Decimal256::percent(&env, 20)
        );

        // %1 / half
        assert_eq!(
            Decimal256::percent(&env, 100).div(&env, half.clone()),
            Decimal256::percent(&env, 200)
        );

        // %10 / half
        assert_eq!(
            Decimal256::percent(&env, 1_000).div(&env, half.clone()),
            Decimal256::percent(&env, 2_000)
        );

        // %0.15 / %0.6
        assert_eq!(
            Decimal256::percent(&env, 15).div(&env, Decimal256::percent(&env, 60)),
            Decimal256::percent(&env, 25)
        );
    }

    #[test]
    #[should_panic(expected = "overflow has occured")]
    fn decimal256_div_overflow_panics() {
        let env = Env::default();
        let _value = Decimal256(U256::from_parts(
            &env,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
        ))
        .div(&env, Decimal256::percent(&env, 10));
    }

    #[test]
    #[should_panic(expected = "Division failed - denominator must not be zero")]
    fn decimal256_div_by_zero_panics() {
        let env = Env::default();
        let _value = Decimal256::one(&env).div(&env, Decimal256::zero(&env));
    }

    #[test]
    fn decimal256_pow_works() {
        let env = Env::default();
        assert_eq!(
            Decimal256::percent(&env, 200).pow(&env, 2),
            Decimal256::percent(&env, 400)
        );
        assert_eq!(
            Decimal256::percent(&env, 100).pow(&env, 10),
            Decimal256::percent(&env, 100)
        );
    }

    #[test]
    #[should_panic(expected = "overflow has occured")]
    fn decimal256_pow_overflow_panics() {
        let env = Env::default();
        _ = Decimal256(U256::from_parts(
            &env,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
        ))
        .pow(&env, 2u32);
    }

    #[test]
    fn test_denominator() {
        let env = Env::default();
        let decimal = Decimal256::percent(&env, 123);
        assert_eq!(
            decimal.denominator(&env),
            Decimal256::decimal_fractional(&env)
        );
    }

    #[test]
    fn test_atomics() {
        let env = Env::default();
        let decimal = Decimal256::percent(&env, 123);
        assert_eq!(decimal.atomics().unwrap(), 1230000000000000000);
    }

    #[test]
    fn test_to_i128_with_precision() {
        let env = Env::default();
        let decimal = Decimal256::percent(&env, 124);
        assert_eq!(decimal.to_u128_with_precision(1), 12);
        assert_eq!(decimal.to_u128_with_precision(2), 124);
    }

    #[test]
    fn test_multiply_ratio() {
        let env = Env::default();
        let decimal = Decimal256::percent(&env, 1);
        let numerator = Decimal256::new(&env, 2);
        let denominator = Decimal256::new(&env, 5);

        // decimal is 10_000_000_000_000_000, atomics would be same
        // numerator is 20_000_000_000_000_000, atomics would be same
        // denominator is 50_000_000_000_000_000, amount would be same
        // decimal * numerator = 200_000_000_000_000_000_000_000_000_000
        // decimal from ratio
        // numerator 200_000_000_000_000_000_000_000_000_000
        // denominator = 50_000_000_000_000_000
        // numerator * decimal256_FRACTIONAL / denominator is the result
        assert_eq!(
            decimal.multiply_ratio(&env, numerator, denominator),
            Decimal256::new(&env, 4000000000000000000000000000000000)
        );
    }

    #[test]
    fn test_abs_difference() {
        let env = Env::default();
        let a = Decimal256::new(&env, 100);
        let b = Decimal256::new(&env, 200);

        assert_eq!(
            a.clone().abs_diff(&env, b.clone()),
            Decimal256::new(&env, 100)
        );
        assert_eq!(b.clone().abs_diff(&env, a), Decimal256::new(&env, 100));
    }

    #[test]
    fn test_checked_from_ratio() {
        let env = Env::default();
        let numerator = Decimal256::new(&env, 100);
        let denominator = Decimal256::new(&env, 200);

        assert_eq!(
            Decimal256::checked_from_ratio(&env, numerator.0, denominator.0),
            Ok(Decimal256::new(&env, 500_000_000_000_000_000))
        );
    }

    #[test]
    fn test_decimal256_places() {
        let env = Env::default();
        let a = Decimal256::percent(&env, 50);

        assert_eq!(a.decimal_places(), 18);
    }

    #[test]
    fn multiply_decimal256_with_u128() {
        let env = Env::default();

        let decimal256 = Decimal256::new(&env, 100 * 1_000_000_000_000_000_000u128);
        let result = decimal256.mul_u128(&env, 10);
        assert_eq!(U256::from_u128(&env, 1_000), result);

        // `u128::MAX` is 340_282_366_920_938_463_463_374_607_431_768_211_455
        // leaving `big_decimal256` to be the exact same value
        // multiplying that number by `1_000_000` it becomes
        // `340_282_366_920_938_463_463_374_607_431_768_211_455_000_000` and then
        // dividing by `1_000_000_000_000_000_000` making it look like the expected
        let big_decimal256 = Decimal256::new(&env, u128::MAX);
        let result = big_decimal256.mul_u128(&env, 1_000_000);
        assert_eq!(
            U256::from_u128(&env, 340_282_366_920_938_463_463_374_607),
            result
        );
    }

    #[test]
    fn multiply_decimal256_with_zero_values() {
        let env = Env::default();

        let decimal256 = Decimal256::new(&env, 1_000_000_000_000_000_000u128);
        let result = decimal256.mul_u128(&env, 0);
        assert_eq!(U256::from_u128(&env, 0), result);

        let zero_decimal = Decimal256::new(&env, 0u128);
        let result = zero_decimal.mul_u128(&env, 1_000);
        assert_eq!(U256::from_u128(&env, 0), result);
    }
}
