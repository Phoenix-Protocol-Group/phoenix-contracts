// A lot of this code is taken from the cosmwasm-std crate, which is licensed under the Apache
// License 2.0 - https://github.com/CosmWasm/cosmwasm.

#![no_std]

use core::{
    cmp::{PartialEq, PartialOrd},
    ops::{Add, Div, Mul, Sub},
};

extern crate alloc;

#[derive(Debug)]
enum Error {
    Overflow,
    DivideByZero,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Decimal(i128);

#[allow(dead_code)]
impl Decimal {
    const DECIMAL_FRACTIONAL: i128 = 1_000_000_000_000_000_000i128; // 1*10**18
    const DECIMAL_FRACTIONAL_SQUARED: i128 = 1_000_000_000_000_000_000_000_000_000_000_000_000i128; // (1*10**18)**2 = 1*10**36
    /// The number of decimal places. Since decimal types are fixed-point rather than
    /// floating-point, this is a constant.
    pub const DECIMAL_PLACES: u32 = 18;
    /// The largest value that can be represented by this decimal type.
    pub const MAX: Self = Self(i128::MAX);
    /// The smallest value that can be represented by this decimal type.
    pub const MIN: Self = Self(i128::MIN);

    pub fn new(value: i128) -> Self {
        Decimal(value)
    }

    pub const fn raw(value: i128) -> Self {
        Self(value)
    }

    /// Create a 1.0 Decimal
    #[inline]
    pub const fn one() -> Self {
        Self(Self::DECIMAL_FRACTIONAL)
    }

    /// Create a 0.0 Decimal
    #[inline]
    pub const fn zero() -> Self {
        Self(0i128)
    }

    /// Convert x% into Decimal
    pub fn percent(x: i64) -> Self {
        Self((x as i128) * 10_000_000_000_000_000)
    }

    /// Convert permille (x/1000) into Decimal
    pub fn permille(x: i64) -> Self {
        Self((x as i128) * 1_000_000_000_000_000)
    }

    /// Convert basis points (x/10000) into Decimal
    pub fn bps(x: i64) -> Self {
        Self((x as i128) * 100_000_000_000_000)
    }

    /// The number of decimal places. This is a constant value for now
    /// but this could potentially change as the type evolves.
    ///
    /// See also [`Decimal::atomics()`].
    #[must_use]
    #[inline]
    pub const fn decimal_places(&self) -> u32 {
        Self::DECIMAL_PLACES
    }

    #[inline]
    fn numerator(&self) -> i128 {
        self.0
    }

    #[inline]
    fn denominator(&self) -> i128 {
        Self::DECIMAL_FRACTIONAL
    }

    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.0 == 0i128
    }

    /// A decimal is an integer of atomic units plus a number that specifies the
    /// position of the decimal dot. So any decimal can be expressed as two numbers.
    ///
    /// ## Examples
    ///
    /// ```
    /// use decimal::Decimal;
    /// // Value with whole and fractional part
    /// let a = Decimal::percent(123);
    /// assert_eq!(a.decimal_places(), 18);
    /// assert_eq!(a.atomics(), 1230000000000000000);
    ///
    /// // Smallest possible value
    /// let b = Decimal::new(1);
    /// assert_eq!(b.decimal_places(), 18);
    /// assert_eq!(b.atomics(), 1);
    /// ```
    #[must_use]
    #[inline]
    pub const fn atomics(&self) -> i128 {
        self.0
    }

    /// Returns the multiplicative inverse `1/d` for decimal `d`.
    ///
    /// If `d` is zero, none is returned.
    pub fn inv(&self) -> Option<Self> {
        if self.is_zero() {
            None
        } else {
            // Let self be p/q with p = self.0 and q = DECIMAL_FRACTIONAL.
            // Now we calculate the inverse a/b = q/p such that b = DECIMAL_FRACTIONAL. Then
            // `a = DECIMAL_FRACTIONAL*DECIMAL_FRACTIONAL / self.0`.
            Some(Decimal(Self::DECIMAL_FRACTIONAL_SQUARED / self.0))
        }
    }

    /// Returns the ratio (numerator / denominator) as a Decimal
    pub fn from_ratio(numerator: impl Into<i128>, denominator: impl Into<i128>) -> Self {
        match Decimal::checked_from_ratio(numerator, denominator) {
            Ok(ratio) => ratio,
            Err(Error::DivideByZero) => panic!("Denominator must not be zero"),
            Err(Error::Overflow) => panic!("Multiplication overflow"),
        }
    }

    /// Returns the ratio (numerator / denominator) as a Decimal
    fn checked_from_ratio(
        numerator: impl Into<i128>,
        denominator: impl Into<i128>,
    ) -> Result<Self, Error> {
        let numerator = numerator.into();
        let denominator = denominator.into();

        // If denominator is zero, panic.
        if denominator == 0 {
            return Err(Error::DivideByZero);
        }

        // Convert numerator and denominator to BigInt.
        // unwrap since i128 is always convertible to BigInt
        // let numerator = numerator.to_bigint().unwrap();
        // let denominator = denominator.to_bigint().unwrap();
        // let decimal_fractional = Self::DECIMAL_FRACTIONAL.to_bigint().unwrap();

        // Compute the ratio: (numerator * DECIMAL_FRACTIONAL) / denominator
        let ratio = (numerator * Self::DECIMAL_FRACTIONAL) / denominator;

        // Convert back to i128. If conversion fails, panic.
        // let ratio = ratio.to_i128().ok_or(Error::Overflow)?;

        // Construct and return the Decimal.
        Ok(Decimal(ratio))
    }

    pub fn abs(&self) -> Self {
        if self.0 < 0 {
            Decimal(-self.0)
        } else {
            *self
        }
    }
}

impl Add for Decimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Decimal(self.0 + other.0)
    }
}
impl Sub for Decimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Decimal(self.0 - other.0)
    }
}

impl Mul for Decimal {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, other: Self) -> Self {
        // Decimals are fractions. We can multiply two decimals a and b
        // via
        //       (a.numerator() * b.numerator()) / (a.denominator() * b.denominator())
        //     = (a.numerator() * b.numerator()) / a.denominator() / b.denominator()

        // let self_numerator = self.numerator().to_bigint().unwrap();
        // let other_numerator = other.numerator().to_bigint().unwrap();

        // Compute the product of the numerators and divide by DECIMAL_FRACTIONAL
        let result = (self.numerator() * other.numerator()) / Self::DECIMAL_FRACTIONAL;

        // Convert the result back to i128, and panic on overflow
        // let result = result
        //     .to_i128()
        //     .unwrap_or_else(|| panic!("attempt to multiply with overflow"));

        // Return a new Decimal
        Decimal(result)
    }
}

impl Div for Decimal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        match Decimal::checked_from_ratio(self.numerator(), rhs.numerator()) {
            Ok(ratio) => ratio,
            Err(Error::DivideByZero) => panic!("Division failed - denominator must not be zero"),
            Err(Error::Overflow) => panic!("Division failed - multiplication overflow"),
        }
    }
}

impl Mul<i128> for Decimal {
    type Output = i128;

    fn mul(self, rhs: i128) -> Self::Output {
        rhs * self
    }
}

impl Div<i128> for Decimal {
    type Output = Self;

    fn div(self, rhs: i128) -> Self::Output {
        Decimal(self.0 / rhs)
    }
}

impl Mul<Decimal> for i128 {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Decimal) -> Self::Output {
        // 0*a and b*0 is always 0
        if self == 0i128 || rhs.is_zero() {
            return 0i128;
        }
        self * rhs.0 / Decimal::DECIMAL_FRACTIONAL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_new() {
        let expected = 300i128;
        assert_eq!(Decimal::new(expected).0, expected);
    }

    #[test]
    fn decimal_raw() {
        let value = 300i128;
        assert_eq!(Decimal::raw(value).0, value);
    }

    #[test]
    fn decimal_one() {
        let value = Decimal::one();
        assert_eq!(value.0, Decimal::DECIMAL_FRACTIONAL);
    }

    #[test]
    fn decimal_zero() {
        let value = Decimal::zero();
        assert_eq!(value.0, 0i128);
    }

    #[test]
    fn decimal_percent() {
        let value = Decimal::percent(50);
        assert_eq!(value.0, Decimal::DECIMAL_FRACTIONAL / 2i128);
    }

    #[test]
    fn decimal_from_ratio_works() {
        // 1.0
        assert_eq!(Decimal::from_ratio(1i128, 1i128), Decimal::one());
        assert_eq!(Decimal::from_ratio(53i128, 53i128), Decimal::one());
        assert_eq!(Decimal::from_ratio(125i128, 125i128), Decimal::one());

        // 1.5
        assert_eq!(Decimal::from_ratio(3i128, 2i128), Decimal::percent(150));
        assert_eq!(Decimal::from_ratio(150i128, 100i128), Decimal::percent(150));
        assert_eq!(Decimal::from_ratio(333i128, 222i128), Decimal::percent(150));

        // 0.125
        assert_eq!(Decimal::from_ratio(1i64, 8i64), Decimal::permille(125));
        assert_eq!(Decimal::from_ratio(125i64, 1000i64), Decimal::permille(125));

        // 1/3 (result floored)
        assert_eq!(
            Decimal::from_ratio(1i128, 3i128),
            Decimal(333_333_333_333_333_333i128)
        );

        // 2/3 (result floored)
        assert_eq!(
            Decimal::from_ratio(2i128, 3i128),
            Decimal(666_666_666_666_666_666i128)
        );

        // large inputs
        assert_eq!(Decimal::from_ratio(0i128, i128::MAX), Decimal::zero());
        // assert_eq!(Decimal::from_ratio(i128::MAX, i128::MAX), Decimal::one());

        // due to limited possibilities - we're only allowed to use i128 as input - maximum
        // number this implementation supports without overflow is u128 / DECIMAL_FRACTIONAL
        // 340282366920938463463374607431768211455 / 10^18 is approximately 340282366920938.
        assert_eq!(
            Decimal::from_ratio(340282366920938i128, 340282366920938i128),
            Decimal::one()
        );
        // This works because of similar orders of magnitude
        assert_eq!(
            Decimal::from_ratio(34028236692093900000i128, 34028236692093900000i128),
            Decimal::one()
        );
        assert_eq!(
            Decimal::from_ratio(34028236692093900000i128, 1i128),
            Decimal::new(34028236692093900000i128 * Decimal::DECIMAL_FRACTIONAL)
        );
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn decimal_from_ratio_panics_for_zero_denominator() {
        Decimal::from_ratio(1i128, 0i128);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn decimal_from_ratio_panics_for_mul_overflow() {
        Decimal::from_ratio(i128::MAX, 1i128);
    }

    #[test]
    fn decimal_decimal_places_works() {
        let zero = Decimal::zero();
        let one = Decimal::one();
        let half = Decimal::percent(50);
        let two = Decimal::percent(200);
        let max = Decimal::MAX;

        assert_eq!(zero.decimal_places(), 18);
        assert_eq!(one.decimal_places(), 18);
        assert_eq!(half.decimal_places(), 18);
        assert_eq!(two.decimal_places(), 18);
        assert_eq!(max.decimal_places(), 18);
    }

    #[test]
    fn decimal_is_zero_works() {
        assert!(Decimal::zero().is_zero());
        assert!(Decimal::percent(0).is_zero());
        assert!(Decimal::permille(0).is_zero());

        assert!(!Decimal::one().is_zero());
        assert!(!Decimal::percent(123).is_zero());
        assert!(!Decimal::permille(1234).is_zero());
    }

    #[test]
    fn decimal_inv_works() {
        // d = 0
        assert_eq!(Decimal::zero().inv(), None);

        // d == 1
        assert_eq!(Decimal::one().inv(), Some(Decimal::one()));

        // d > 1 exact
        assert_eq!(Decimal::percent(200).inv(), Some(Decimal::percent(50)));
        assert_eq!(Decimal::percent(2_000).inv(), Some(Decimal::percent(5)));
        assert_eq!(Decimal::percent(20_000).inv(), Some(Decimal::permille(5)));
        assert_eq!(Decimal::percent(200_000).inv(), Some(Decimal::bps(5)));

        // d > 1 rounded
        assert_eq!(
            Decimal::percent(300).inv(),
            Some(Decimal::from_ratio(1i128, 3i128))
        );
        assert_eq!(
            Decimal::percent(600).inv(),
            Some(Decimal::from_ratio(1i128, 6i128))
        );

        // d < 1 exact
        assert_eq!(Decimal::percent(50).inv(), Some(Decimal::percent(200)));
        assert_eq!(Decimal::percent(5).inv(), Some(Decimal::percent(2_000)));
        assert_eq!(Decimal::permille(5).inv(), Some(Decimal::percent(20_000)));
        assert_eq!(Decimal::bps(5).inv(), Some(Decimal::percent(200_000)));
    }

    #[test]
    fn decimal_add_works() {
        let value = Decimal::one() + Decimal::percent(50); // 1.5
        assert_eq!(value.0, Decimal::DECIMAL_FRACTIONAL * 3i128 / 2i128);

        assert_eq!(
            Decimal::percent(5) + Decimal::percent(4),
            Decimal::percent(9)
        );
        assert_eq!(Decimal::percent(5) + Decimal::zero(), Decimal::percent(5));
        assert_eq!(Decimal::zero() + Decimal::zero(), Decimal::zero());
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn decimal_add_overflow_panics() {
        let _value = Decimal::MAX + Decimal::percent(50);
    }

    #[test]
    fn decimal_sub_works() {
        let value = Decimal::one() - Decimal::percent(50); // 0.5
        assert_eq!(value.0, Decimal::DECIMAL_FRACTIONAL / 2i128);

        assert_eq!(
            Decimal::percent(9) - Decimal::percent(4),
            Decimal::percent(5)
        );
        assert_eq!(Decimal::percent(16) - Decimal::zero(), Decimal::percent(16));
        assert_eq!(Decimal::percent(16) - Decimal::percent(16), Decimal::zero());
        assert_eq!(Decimal::zero() - Decimal::zero(), Decimal::zero());
    }

    #[test]
    fn decimal_implements_mul() {
        let one = Decimal::one();
        let two = one + one;
        let half = Decimal::percent(50);

        // 1*x and x*1
        assert_eq!(one * Decimal::percent(0), Decimal::percent(0));
        assert_eq!(one * Decimal::percent(1), Decimal::percent(1));
        assert_eq!(one * Decimal::percent(10), Decimal::percent(10));
        assert_eq!(one * Decimal::percent(100), Decimal::percent(100));
        assert_eq!(one * Decimal::percent(1000), Decimal::percent(1000));
        // assert_eq!(one * Decimal::MAX, Decimal::MAX);
        assert_eq!(Decimal::percent(0) * one, Decimal::percent(0));
        assert_eq!(Decimal::percent(1) * one, Decimal::percent(1));
        assert_eq!(Decimal::percent(10) * one, Decimal::percent(10));
        assert_eq!(Decimal::percent(100) * one, Decimal::percent(100));
        assert_eq!(Decimal::percent(1000) * one, Decimal::percent(1000));
        // assert_eq!(Decimal::MAX * one, Decimal::MAX);

        // double
        assert_eq!(two * Decimal::percent(0), Decimal::percent(0));
        assert_eq!(two * Decimal::percent(1), Decimal::percent(2));
        assert_eq!(two * Decimal::percent(10), Decimal::percent(20));
        assert_eq!(two * Decimal::percent(100), Decimal::percent(200));
        assert_eq!(two * Decimal::percent(1000), Decimal::percent(2000));
        assert_eq!(Decimal::percent(0) * two, Decimal::percent(0));
        assert_eq!(Decimal::percent(1) * two, Decimal::percent(2));
        assert_eq!(Decimal::percent(10) * two, Decimal::percent(20));
        assert_eq!(Decimal::percent(100) * two, Decimal::percent(200));
        assert_eq!(Decimal::percent(1000) * two, Decimal::percent(2000));

        // half
        assert_eq!(half * Decimal::percent(0), Decimal::percent(0));
        assert_eq!(half * Decimal::percent(1), Decimal::permille(5));
        assert_eq!(half * Decimal::percent(10), Decimal::percent(5));
        assert_eq!(half * Decimal::percent(100), Decimal::percent(50));
        assert_eq!(half * Decimal::percent(1000), Decimal::percent(500));
        assert_eq!(Decimal::percent(0) * half, Decimal::percent(0));
        assert_eq!(Decimal::percent(1) * half, Decimal::permille(5));
        assert_eq!(Decimal::percent(10) * half, Decimal::percent(5));
        assert_eq!(Decimal::percent(100) * half, Decimal::percent(50));
        assert_eq!(Decimal::percent(1000) * half, Decimal::percent(500));
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn decimal_mul_overflow_panics() {
        let _value = Decimal::MAX * Decimal::percent(101);
    }

    #[test]
    // in this test the Decimal is on the right
    fn i128_decimal_multiply() {
        // a*b
        let left = 300i128;
        let right = Decimal::one() + Decimal::percent(50); // 1.5
        assert_eq!(left * right, 450i128);

        // a*0
        let left = 300i128;
        let right = Decimal::zero();
        assert_eq!(left * right, 0i128);

        // 0*a
        let left = 0i128;
        let right = Decimal::one() + Decimal::percent(50); // 1.5
        assert_eq!(left * right, 0i128);

        assert_eq!(0i128 * Decimal::one(), 0i128);
        assert_eq!(1i128 * Decimal::one(), 1i128);
        assert_eq!(2i128 * Decimal::one(), 2i128);

        assert_eq!(1i128 * Decimal::percent(10), 0i128);
        assert_eq!(10i128 * Decimal::percent(10), 1i128);
        assert_eq!(100i128 * Decimal::percent(10), 10i128);

        assert_eq!(1i128 * Decimal::percent(50), 0i128);
        assert_eq!(100i128 * Decimal::percent(50), 50i128);
        assert_eq!(3200i128 * Decimal::percent(50), 1600i128);
        assert_eq!(999i128 * Decimal::percent(50), 499i128); // default rounding down

        assert_eq!(1i128 * Decimal::percent(200), 2i128);
        assert_eq!(1000i128 * Decimal::percent(200), 2000i128);
    }

    #[test]
    // in this test the Decimal is on the left
    fn decimal_i128_multiply() {
        // a*b
        let left = Decimal::one() + Decimal::percent(50); // 1.5
        let right = 300i128;
        assert_eq!(left * right, 450i128);

        // 0*a
        let left = Decimal::zero();
        let right = 300i128;
        assert_eq!(left * right, 0i128);

        // a*0
        let left = Decimal::one() + Decimal::percent(50); // 1.5
        let right = 0i128;
        assert_eq!(left * right, 0i128);
    }

    #[test]
    fn decimal_implements_div() {
        let one = Decimal::one();
        let two = one + one;
        let half = Decimal::percent(50);

        // 1/x and x/1
        assert_eq!(one / Decimal::percent(1), Decimal::percent(10_000));
        assert_eq!(one / Decimal::percent(10), Decimal::percent(1_000));
        assert_eq!(one / Decimal::percent(100), Decimal::percent(100));
        assert_eq!(one / Decimal::percent(1000), Decimal::percent(10));
        assert_eq!(Decimal::percent(0) / one, Decimal::percent(0));
        assert_eq!(Decimal::percent(1) / one, Decimal::percent(1));
        assert_eq!(Decimal::percent(10) / one, Decimal::percent(10));
        assert_eq!(Decimal::percent(100) / one, Decimal::percent(100));
        assert_eq!(Decimal::percent(1000) / one, Decimal::percent(1000));

        // double
        assert_eq!(two / Decimal::percent(1), Decimal::percent(20_000));
        assert_eq!(two / Decimal::percent(10), Decimal::percent(2_000));
        assert_eq!(two / Decimal::percent(100), Decimal::percent(200));
        assert_eq!(two / Decimal::percent(1000), Decimal::percent(20));
        assert_eq!(Decimal::percent(0) / two, Decimal::percent(0));
        assert_eq!(Decimal::percent(10) / two, Decimal::percent(5));
        assert_eq!(Decimal::percent(100) / two, Decimal::percent(50));
        assert_eq!(Decimal::percent(1000) / two, Decimal::percent(500));

        // half
        assert_eq!(half / Decimal::percent(1), Decimal::percent(5_000));
        assert_eq!(half / Decimal::percent(10), Decimal::percent(500));
        assert_eq!(half / Decimal::percent(100), Decimal::percent(50));
        assert_eq!(half / Decimal::percent(1000), Decimal::percent(5));
        assert_eq!(Decimal::percent(0) / half, Decimal::percent(0));
        assert_eq!(Decimal::percent(1) / half, Decimal::percent(2));
        assert_eq!(Decimal::percent(10) / half, Decimal::percent(20));
        assert_eq!(Decimal::percent(100) / half, Decimal::percent(200));
        assert_eq!(Decimal::percent(1000) / half, Decimal::percent(2000));

        assert_eq!(
            Decimal::percent(15) / Decimal::percent(60),
            Decimal::percent(25)
        );
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn decimal_div_overflow_panics() {
        let _value = Decimal::MAX / Decimal::percent(10);
    }

    #[test]
    #[should_panic(expected = "Division failed - denominator must not be zero")]
    fn decimal_div_by_zero_panics() {
        let _value = Decimal::one() / Decimal::zero();
    }

    #[test]
    fn decimal_i128_division() {
        // a/b
        let left = Decimal::percent(150); // 1.5
        let right = 3i128;
        assert_eq!(left / right, Decimal::percent(50));

        // 0/a
        let left = Decimal::zero();
        let right = 300i128;
        assert_eq!(left / right, Decimal::zero());
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn decimal_uint128_divide_by_zero() {
        let left = Decimal::percent(150); // 1.5
        let right = 0i128;
        let _result = left / right;
    }
}
