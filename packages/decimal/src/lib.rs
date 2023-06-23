// A lot of this code is taken from the cosmwasm-std crate, which is licensed under the Apache
// License 2.0 - https://github.com/CosmWasm/cosmwasm.

#![no_std]
use core::{
    cmp::{PartialEq, PartialOrd},
    ops::Mul,
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Decimal(i128);

#[allow(dead_code)]
impl Decimal {
    const DECIMAL_FRACTIONAL: i128 = 1_000_000_000_000_000_000i128; // 1*10**18
    const DECIMAL_FRACTIONAL_SQUARED: i128 = 1_000_000_000_000_000_000_000_000_000_000_000_000i128; // (1*10**18)**2 = 1*10**36
    /// The number of decimal places. Since decimal types are fixed-point rather than
    /// floating-point, this is a constant.
    pub const DECIMAL_PLACES: u32 = 18;

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
    pub fn percent(x: u64) -> Self {
        Self((x as i128) * 10_000_000_000_000_000)
    }

    /// Convert basis points (x/10000) into Decimal
    pub fn bps(x: u64) -> Self {
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
    fn inv(&self) -> Option<Self> {
        if self.is_zero() {
            None
        } else {
            // Let self be p/q with p = self.0 and q = DECIMAL_FRACTIONAL.
            // Now we calculate the inverse a/b = q/p such that b = DECIMAL_FRACTIONAL. Then
            // `a = DECIMAL_FRACTIONAL*DECIMAL_FRACTIONAL / self.0`.
            Some(Decimal(Self::DECIMAL_FRACTIONAL_SQUARED / self.0))
        }
    }

    /// Returns the ratio (numerator / denominator) as a Decimal.
    /// Panics if the denominator is zero or if the operation would cause an overflow.
    pub fn from_ratio(numerator: i128, denominator: i128) -> Self {
        // Check if the denominator is zero.
        if denominator == 0 {
            panic!("Denominator cannot be zero");
        }

        // Check if the operation would cause an overflow.
        if numerator.checked_mul(Self::DECIMAL_FRACTIONAL).is_none() {
            panic!("Overflow error");
        }

        // Calculate the ratio.
        let ratio = numerator * Self::DECIMAL_FRACTIONAL / denominator;

        Decimal(ratio)
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

impl Mul<i128> for Decimal {
    type Output = i128;

    fn mul(self, rhs: i128) -> Self::Output {
        rhs * self
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

        assert_eq!(Decimal::from_ratio(0i128, i128::MAX), Decimal::zero());
    }

    #[test]
    fn multiplying_i128() {
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
}
