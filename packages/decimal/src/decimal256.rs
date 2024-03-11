// A lot of this code is taken from the cosmwasm-std crate, which is licensed under the Apache
// License 2.0 - https://github.com/CosmWasm/cosmwasm.

use soroban_sdk::{Bytes, Env, String, I256};

use core::{
    cmp::{Ordering, PartialEq, PartialOrd},
    fmt,
    ops::{Add, Div, Mul, Sub},
    str::FromStr,
};

extern crate alloc;

#[allow(dead_code)]
#[derive(Debug)]
enum Error {
    DivideByZero,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Decimal256(I256);

#[allow(dead_code)]
impl Decimal256 {
    // Number of decimal places
    pub const DECIMAL_PLACES: i32 = 18;

    // Function to get DECIMAL_FRACTIONAL
    pub fn decimal_fractional(env: &Env) -> I256 {
        I256::from_i128(env, 1_000_000_000_000_000_000i128) // 1*10**18
    }

    // Function to get DECIMAL_FRACTIONAL_SQUARED
    pub fn decimal_fractional_squared(env: &Env) -> I256 {
        I256::from_i128(env, 1_000_000_000_000_000_000_000_000_000_000_000_000i128)
        // (1*10**18)**2 = 1*10**36
    }

    // Function to get MAX value
    pub fn max(env: &Env) -> Self {
        Self(I256::from_i128(env, (1i128 << 127) - 1))
    }

    // Function to get MIN value
    pub fn min(env: &Env) -> Self {
        Self(I256::from_i128(env, -1i128 << 127))
    }

    // pub fn new(value: I256) -> Self {
    //     Decimal256(value)
    // }

    // pub const fn raw(value: I256) -> Self {
    //     Self(value)
    // }

    // /// Create a 1.0 Decimal256
    // #[inline]
    // pub const fn one() -> Self {
    //     Self(Self::DECIMAL_FRACTIONAL)
    // }

    // /// Create a 0.0 Decimal256
    // #[inline]
    // pub const fn zero() -> Self {
    //     Self(I256::from_i32(&Env::default(), 0i32))
    // }

    // /// Convert x% into Decimal256
    // pub fn percent(x: i64) -> Self {
    //     Self(I256::from_i128(&Env::default(), (x as i128) * 10_000_000_000_000_000))
    // }

    // /// Convert permille (x/1000) into Decimal256
    // pub fn permille(x: i64) -> Self {
    //     Self(I256::from_i128(&Env::default(), (x as i128) * 1_000_000_000_000_000))
    // }

    // /// Convert basis points (x/10000) into Decimal256
    // pub fn bps(x: i64) -> Self {
    //     Self(I256::from_i128(&Env::default(), (x as i128) * 100_000_000_000_000))
    // }

    // /// The number of decimal places. This is a constant value for now
    // /// but this could potentially change as the type evolves.
    // ///
    // /// See also [`Decimal256::atomics()`].
    // #[must_use]
    // #[inline]
    // pub const fn decimal_places(&self) -> i32 {
    //     Self::DECIMAL_PLACES
    // }

    // #[inline]
    // fn numerator(&self) -> I256 {
    //     self.0
    // }

    // #[inline]
    // fn denominator(&self) -> I256 {
    //     Self::DECIMAL_FRACTIONAL
    // }

    // #[must_use]
    // pub const fn is_zero(&self) -> bool {
    //     self.0 == I256::from_i32(&Env::default(), 0i32)
    // }

    // /// A decimal is an integer of atomic units plus a number that specifies the
    // /// position of the decimal dot. So any decimal can be expressed as two numbers.
    // ///
    // /// ## Examples
    // ///
    // /// ```
    // /// use decimal::Decimal256;
    // /// // Value with whole and fractional part
    // /// let a = Decimal256::percent(123);
    // /// assert_eq!(a.decimal_places(), 18);
    // /// assert_eq!(a.atomics(), 1230000000000000000);
    // ///
    // /// // Smallest possible value
    // /// let b = Decimal256::new(1);
    // /// assert_eq!(b.decimal_places(), 18);
    // /// assert_eq!(b.atomics(), 1);
    // /// ```
    // #[must_use]
    // #[inline]
    // pub const fn atomics(&self) -> I256 {
    //     self.0
    // }

    // /// Creates a decimal from a number of atomic units and the number
    // /// of decimal places. The inputs will be converted internally to form
    // /// a decimal with 18 decimal places. So the input 1234 and 3 will create
    // /// the decimal 1.234.
    // ///
    // /// Using 18 decimal places is slightly more efficient than other values
    // /// as no internal conversion is necessary.
    // ///
    // /// ## Examples
    // ///
    // /// ```
    // /// use decimal::Decimal256;
    // /// use soroban_sdk::{String, Env};
    // ///
    // /// let e = Env::default();
    // /// let a = Decimal256::from_atomics(1234, 3);
    // /// assert_eq!(a.to_string(&e), String::from_slice(&e, "1.234"));
    // ///
    // /// let a = Decimal256::from_atomics(1234, 0);
    // /// assert_eq!(a.to_string(&e), String::from_slice(&e, "1234"));
    // ///
    // /// let a = Decimal256::from_atomics(1, 18);
    // /// assert_eq!(a.to_string(&e), String::from_slice(&e, "0.000000000000000001"));
    // /// ```
    // pub fn from_atomics(atomics: impl Into<I256>, decimal_places: i32) -> Self {
    //     let atomics: I256 = atomics.into();
    //     const TEN: I256 = I256::from_be_bytes(&Env::default(), &Bytes::from_slice(&Env::default(), &[
    //         0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //         0, 0, 10,
    //     ]));
    //     match decimal_places.cmp(&Self::DECIMAL_PLACES) {
    //         Ordering::Less => {
    //             let digits = Self::DECIMAL_PLACES - decimal_places;
    //             let factor: I256 = TEN.pow(digits as u32);
    //             Self(atomics * factor)
    //         }
    //         Ordering::Equal => Self(atomics),
    //         Ordering::Greater => {
    //             let digits = decimal_places - Self::DECIMAL_PLACES;
    //             let factor = TEN.pow(digits as u32);
    //             // Since factor cannot be zero, the division is safe.
    //             Self(atomics / factor)
    //         }
    //     }
    // }

    // /// Raises a value to the power of `exp`, panicking if an overflow occurs.
    // pub fn pow(self, exp: u32) -> Self {
    //     // This uses the exponentiation by squaring algorithm:
    //     // https://en.wikipedia.org/wiki/Exponentiation_by_squaring#Basic_method

    //     fn inner(mut x: Decimal256, mut n: u32) -> Decimal256 {
    //         if n == 0 {
    //             return Decimal256::one();
    //         }

    //         let mut y = Decimal256::one();

    //         while n > 1 {
    //             if n % 2 == 0 {
    //                 x = x * x; // Regular multiplication
    //                 n /= 2;
    //             } else {
    //                 y = x * y; // Regular multiplication
    //                 x = x * x; // Regular multiplication
    //                 n = (n - 1) / 2;
    //             }
    //         }

    //         x * y
    //     }

    //     inner(self, exp)
    // }

    // /// Returns the multiplicative inverse `1/d` for decimal `d`.
    // ///
    // /// If `d` is zero, none is returned.
    // pub fn inv(&self) -> Option<Self> {
    //     if self.is_zero() {
    //         None
    //     } else {
    //         // Let self be p/q with p = self.0 and q = DECIMAL_FRACTIONAL.
    //         // Now we calculate the inverse a/b = q/p such that b = DECIMAL_FRACTIONAL. Then
    //         // `a = DECIMAL_FRACTIONAL*DECIMAL_FRACTIONAL / self.0`.
    //         Some(Decimal256(Self::DECIMAL_FRACTIONAL_SQUARED / self.0))
    //     }
    // }

    // /// Returns the ratio (numerator / denominator) as a Decimal256
    // pub fn from_ratio(numerator: impl Into<I256>, denominator: impl Into<I256>) -> Self {
    //     match Decimal256::checked_from_ratio(numerator, denominator) {
    //         Ok(ratio) => ratio,
    //         Err(Error::DivideByZero) => panic!("Denominator must not be zero"),
    //     }
    // }

    // pub fn to_I256_with_precision(&self, precision: impl Into<i32>) -> I256 {
    //     let value = self.atomics();
    //     let precision = precision.into();

    //     let divisor = 10I256.pow((self.decimal_places() - precision) as u32);
    //     value / divisor
    // }

    // fn multiply_ratio(&self, numerator: Decimal256, denominator: Decimal256) -> Decimal256 {
    //     Decimal256::from_ratio(self.atomics() * numerator.atomics(), denominator.atomics())
    // }

    // /// Returns the ratio (numerator / denominator) as a Decimal256
    // fn checked_from_ratio(
    //     numerator: impl Into<I256>,
    //     denominator: impl Into<I256>,
    // ) -> Result<Self, Error> {
    //     let numerator = numerator.into();
    //     let denominator = denominator.into();

    //     // If denominator is zero, panic.
    //     if denominator == 0 {
    //         return Err(Error::DivideByZero);
    //     }

    //     // Convert numerator and denominator to BigInt.
    //     // unwrap since I256 is always convertible to BigInt
    //     // let numerator = numerator.to_bigint().unwrap();
    //     // let denominator = denominator.to_bigint().unwrap();
    //     // let decimal_fractional = Self::DECIMAL_FRACTIONAL.to_bigint().unwrap();

    //     // Compute the ratio: (numerator * DECIMAL_FRACTIONAL) / denominator
    //     let ratio = (numerator * Self::DECIMAL_FRACTIONAL) / denominator;

    //     // Convert back to I256. If conversion fails, panic.
    //     // let ratio = ratio.to_I256().ok_or(Error::Overflow)?;

    //     // Construct and return the Decimal256.
    //     Ok(Decimal256(ratio))
    // }

    // pub fn abs(&self) -> Self {
    //     if self.0 < 0 {
    //         Decimal256(-self.0)
    //     } else {
    //         *self
    //     }
    // }

    // pub fn to_string(&self, env: &Env) -> String {
    //     String::from_str(env, alloc::format!("{}", self).as_str())
    // }

    // pub const fn abs_diff(self, other: Self) -> Self {
    //     Self(self.0.abs_diff(other.0) as I256)
    // }
}

// impl Add for Decimal256 {
//     type Output = Self;
//
//     fn add(self, other: Self) -> Self {
//         Decimal256(self.0 + other.0)
//     }
// }
// impl Sub for Decimal256 {
//     type Output = Self;
//
//     fn sub(self, other: Self) -> Self {
//         Decimal256(self.0 - other.0)
//     }
// }
//
// impl Mul for Decimal256 {
//     type Output = Self;
//
//     #[allow(clippy::suspicious_arithmetic_impl)]
//     fn mul(self, other: Self) -> Self {
//         // Decimal256s are fractions. We can multiply two decimals a and b
//         // via
//         //       (a.numerator() * b.numerator()) / (a.denominator() * b.denominator())
//         //     = (a.numerator() * b.numerator()) / a.denominator() / b.denominator()
//
//         // let self_numerator = self.numerator().to_bigint().unwrap();
//         // let other_numerator = other.numerator().to_bigint().unwrap();
//
//         // Compute the product of the numerators and divide by DECIMAL_FRACTIONAL
//         let result = (self.numerator() * other.numerator()) / Self::DECIMAL_FRACTIONAL;
//
//         // Convert the result back to I256, and panic on overflow
//         // let result = result
//         //     .to_I256()
//         //     .unwrap_or_else(|| panic!("attempt to multiply with overflow"));
//
//         // Return a new Decimal256
//         Decimal256(result)
//     }
// }
//
// impl Div for Decimal256 {
//     type Output = Self;
//
//     fn div(self, rhs: Self) -> Self {
//         match Decimal256::checked_from_ratio(self.numerator(), rhs.numerator()) {
//             Ok(ratio) => ratio,
//             Err(Error::DivideByZero) => panic!("Division failed - denominator must not be zero"),
//         }
//     }
// }
//
// impl Mul<I256> for Decimal256 {
//     type Output = I256;
//
//     fn mul(self, rhs: I256) -> Self::Output {
//         rhs * self
//     }
// }
//
// impl Div<I256> for Decimal256 {
//     type Output = Self;
//
//     fn div(self, rhs: I256) -> Self::Output {
//         Decimal256(self.0 / rhs)
//     }
// }
//
// impl Mul<Decimal256> for I256 {
//     type Output = Self;
//
//     #[allow(clippy::suspicious_arithmetic_impl)]
//     fn mul(self, rhs: Decimal256) -> Self::Output {
//         // 0*a and b*0 is always 0
//         if self == 0I256 || rhs.is_zero() {
//             return 0I256;
//         }
//         self * rhs.0 / Decimal256::DECIMAL_FRACTIONAL
//     }
// }
//
// impl FromStr for Decimal256 {
//     type Err = ();
//
//     fn from_str(input: &str) -> Result<Self, Self::Err> {
//         let mut parts_iter = input.split('.');
//
//         let whole_part = parts_iter.next().expect("Unexpected input format");
//         let whole: I256 = whole_part.parse().expect("Error parsing whole");
//         let mut atomics = whole * Self::DECIMAL_FRACTIONAL;
//
//         if let Some(fractional_part) = parts_iter.next() {
//             let fractional: I256 = fractional_part.parse().expect("Error parsing fractional");
//             let exp = Self::DECIMAL_PLACES - fractional_part.len() as i32;
//             assert!(exp >= 0, "There must be at least one fractional digit");
//             let fractional_factor = 10I256.pow(exp as u32);
//             atomics += fractional * fractional_factor;
//         }
//
//         assert!(parts_iter.next().is_none(), "Unexpected number of dots");
//
//         Ok(Decimal256(atomics))
//     }
// }
//
// impl fmt::Display for Decimal256 {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let whole = self.0 / Self::DECIMAL_FRACTIONAL;
//         let fractional = self.0 % Self::DECIMAL_FRACTIONAL;
//
//         if fractional == 0 {
//             write!(f, "{}", whole)
//         } else {
//             let fractional_string = alloc::format!(
//                 "{:0>padding$}",
//                 fractional,
//                 padding = Self::DECIMAL_PLACES as usize
//             );
//             f.write_fmt(format_args!(
//                 "{}.{}",
//                 whole,
//                 fractional_string.trim_end_matches('0')
//             ))
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn decimal_new() {
        let expected = 300i128;
        assert_eq!(Decimal256::new(expected).0, expected);
    }

    #[test]
    fn decimal_raw() {
        let value = 300i128;
        assert_eq!(Decimal256::raw(value).0, value);
    }

    #[test]
    fn decimal_one() {
        let value = Decimal256::one();
        assert_eq!(value.0, Decimal256::DECIMAL_FRACTIONAL);
    }

    #[test]
    fn decimal_zero() {
        let value = Decimal256::zero();
        assert_eq!(value.0, 0i128);
    }

    #[test]
    fn decimal_percent() {
        let value = Decimal256::percent(50);
        assert_eq!(value.0, Decimal256::DECIMAL_FRACTIONAL / 2i128);
    }

    #[test]
    fn decimal_from_atomics_works() {
        let one = Decimal256::one();
        let two = one + one;

        assert_eq!(Decimal256::from_atomics(1i128, 0), one);
        assert_eq!(Decimal256::from_atomics(10i128, 1), one);
        assert_eq!(Decimal256::from_atomics(100i128, 2), one);
        assert_eq!(Decimal256::from_atomics(1000i128, 3), one);
        assert_eq!(Decimal256::from_atomics(1000000000000000000i128, 18), one);
        assert_eq!(Decimal256::from_atomics(10000000000000000000i128, 19), one);
        assert_eq!(Decimal256::from_atomics(100000000000000000000i128, 20), one);

        assert_eq!(Decimal256::from_atomics(2i128, 0), two);
        assert_eq!(Decimal256::from_atomics(20i128, 1), two);
        assert_eq!(Decimal256::from_atomics(200i128, 2), two);
        assert_eq!(Decimal256::from_atomics(2000i128, 3), two);
        assert_eq!(Decimal256::from_atomics(2000000000000000000i128, 18), two);
        assert_eq!(Decimal256::from_atomics(20000000000000000000i128, 19), two);
        assert_eq!(Decimal256::from_atomics(200000000000000000000i128, 20), two);

        // Cuts decimal digits (20 provided but only 18 can be stored)
        assert_eq!(
            Decimal256::from_atomics(4321i128, 20),
            Decimal256::from_str("0.000000000000000043").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(6789i128, 20),
            Decimal256::from_str("0.000000000000000067").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(i128::MAX, 38),
            Decimal256::from_str("1.701411834604692317").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(i128::MAX, 39),
            Decimal256::from_str("0.170141183460469231").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(i128::MAX, 45),
            Decimal256::from_str("0.000000170141183460").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(i128::MAX, 51),
            Decimal256::from_str("0.000000000000170141").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(i128::MAX, 56),
            Decimal256::from_str("0.000000000000000001").unwrap()
        );
    }

    #[test]
    fn decimal_from_ratio_works() {
        // 1.0
        assert_eq!(Decimal256::from_ratio(1i128, 1i128), Decimal256::one());
        assert_eq!(Decimal256::from_ratio(53i128, 53i128), Decimal256::one());
        assert_eq!(Decimal256::from_ratio(125i128, 125i128), Decimal256::one());

        // 1.5
        assert_eq!(
            Decimal256::from_ratio(3i128, 2i128),
            Decimal256::percent(150)
        );
        assert_eq!(
            Decimal256::from_ratio(150i128, 100i128),
            Decimal256::percent(150)
        );
        assert_eq!(
            Decimal256::from_ratio(333i128, 222i128),
            Decimal256::percent(150)
        );

        // 0.125
        assert_eq!(
            Decimal256::from_ratio(1i64, 8i64),
            Decimal256::permille(125)
        );
        assert_eq!(
            Decimal256::from_ratio(125i64, 1000i64),
            Decimal256::permille(125)
        );

        // 1/3 (result floored)
        assert_eq!(
            Decimal256::from_ratio(1i128, 3i128),
            Decimal256(333_333_333_333_333_333i128)
        );

        // 2/3 (result floored)
        assert_eq!(
            Decimal256::from_ratio(2i128, 3i128),
            Decimal256(666_666_666_666_666_666i128)
        );

        // large inputs
        assert_eq!(Decimal256::from_ratio(0i128, i128::MAX), Decimal256::zero());
        // assert_eq!(Decimal256::from_ratio(i128::MAX, i128::MAX), Decimal256::one());

        // due to limited possibilities - we're only allowed to use i128 as input - maximum
        // number this implementation supports without overflow is u128 / DECIMAL_FRACTIONAL
        // 340282366920938463463374607431768211455 / 10^18 is approximately 340282366920938.
        assert_eq!(
            Decimal256::from_ratio(340282366920938i128, 340282366920938i128),
            Decimal256::one()
        );
        // This works because of similar orders of magnitude
        assert_eq!(
            Decimal256::from_ratio(34028236692093900000i128, 34028236692093900000i128),
            Decimal256::one()
        );
        assert_eq!(
            Decimal256::from_ratio(34028236692093900000i128, 1i128),
            Decimal256::new(34028236692093900000i128 * Decimal256::DECIMAL_FRACTIONAL)
        );
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn decimal_from_ratio_panics_for_zero_denominator() {
        Decimal256::from_ratio(1i128, 0i128);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn decimal_from_ratio_panics_for_mul_overflow() {
        Decimal256::from_ratio(i128::MAX, 1i128);
    }

    #[test]
    fn decimal_decimal_places_works() {
        let zero = Decimal256::zero();
        let one = Decimal256::one();
        let half = Decimal256::percent(50);
        let two = Decimal256::percent(200);
        let max = Decimal256::MAX;

        assert_eq!(zero.decimal_places(), 18);
        assert_eq!(one.decimal_places(), 18);
        assert_eq!(half.decimal_places(), 18);
        assert_eq!(two.decimal_places(), 18);
        assert_eq!(max.decimal_places(), 18);
    }

    #[test]
    fn decimal_from_str_works() {
        // Integers
        assert_eq!(Decimal256::from_str("0").unwrap(), Decimal256::percent(0));
        assert_eq!(Decimal256::from_str("1").unwrap(), Decimal256::percent(100));
        assert_eq!(Decimal256::from_str("5").unwrap(), Decimal256::percent(500));
        assert_eq!(
            Decimal256::from_str("42").unwrap(),
            Decimal256::percent(4200)
        );
        assert_eq!(Decimal256::from_str("000").unwrap(), Decimal256::percent(0));
        assert_eq!(
            Decimal256::from_str("001").unwrap(),
            Decimal256::percent(100)
        );
        assert_eq!(
            Decimal256::from_str("005").unwrap(),
            Decimal256::percent(500)
        );
        assert_eq!(
            Decimal256::from_str("0042").unwrap(),
            Decimal256::percent(4200)
        );

        // Decimal256s
        assert_eq!(
            Decimal256::from_str("1.0").unwrap(),
            Decimal256::percent(100)
        );
        assert_eq!(
            Decimal256::from_str("1.5").unwrap(),
            Decimal256::percent(150)
        );
        assert_eq!(
            Decimal256::from_str("0.5").unwrap(),
            Decimal256::percent(50)
        );
        assert_eq!(
            Decimal256::from_str("0.123").unwrap(),
            Decimal256::permille(123)
        );

        assert_eq!(
            Decimal256::from_str("40.00").unwrap(),
            Decimal256::percent(4000)
        );
        assert_eq!(
            Decimal256::from_str("04.00").unwrap(),
            Decimal256::percent(400)
        );
        assert_eq!(
            Decimal256::from_str("00.40").unwrap(),
            Decimal256::percent(40)
        );
        assert_eq!(
            Decimal256::from_str("00.04").unwrap(),
            Decimal256::percent(4)
        );

        // Can handle DECIMAL_PLACES fractional digits
        assert_eq!(
            Decimal256::from_str("7.123456789012345678").unwrap(),
            Decimal256(7123456789012345678i128)
        );
        assert_eq!(
            Decimal256::from_str("7.999999999999999999").unwrap(),
            Decimal256(7999999999999999999i128)
        );
    }

    #[test]
    fn decimal_is_zero_works() {
        assert!(Decimal256::zero().is_zero());
        assert!(Decimal256::percent(0).is_zero());
        assert!(Decimal256::permille(0).is_zero());

        assert!(!Decimal256::one().is_zero());
        assert!(!Decimal256::percent(123).is_zero());
        assert!(!Decimal256::permille(1234).is_zero());
    }

    #[test]
    fn decimal_inv_works() {
        // d = 0
        assert_eq!(Decimal256::zero().inv(), None);

        // d == 1
        assert_eq!(Decimal256::one().inv(), Some(Decimal256::one()));

        // d > 1 exact
        assert_eq!(
            Decimal256::percent(200).inv(),
            Some(Decimal256::percent(50))
        );
        assert_eq!(
            Decimal256::percent(2_000).inv(),
            Some(Decimal256::percent(5))
        );
        assert_eq!(
            Decimal256::percent(20_000).inv(),
            Some(Decimal256::permille(5))
        );
        assert_eq!(Decimal256::percent(200_000).inv(), Some(Decimal256::bps(5)));

        // d > 1 rounded
        assert_eq!(
            Decimal256::percent(300).inv(),
            Some(Decimal256::from_ratio(1i128, 3i128))
        );
        assert_eq!(
            Decimal256::percent(600).inv(),
            Some(Decimal256::from_ratio(1i128, 6i128))
        );

        // d < 1 exact
        assert_eq!(
            Decimal256::percent(50).inv(),
            Some(Decimal256::percent(200))
        );
        assert_eq!(
            Decimal256::percent(5).inv(),
            Some(Decimal256::percent(2_000))
        );
        assert_eq!(
            Decimal256::permille(5).inv(),
            Some(Decimal256::percent(20_000))
        );
        assert_eq!(Decimal256::bps(5).inv(), Some(Decimal256::percent(200_000)));
    }

    #[test]
    fn decimal_add_works() {
        let value = Decimal256::one() + Decimal256::percent(50); // 1.5
        assert_eq!(value.0, Decimal256::DECIMAL_FRACTIONAL * 3i128 / 2i128);

        assert_eq!(
            Decimal256::percent(5) + Decimal256::percent(4),
            Decimal256::percent(9)
        );
        assert_eq!(
            Decimal256::percent(5) + Decimal256::zero(),
            Decimal256::percent(5)
        );
        assert_eq!(Decimal256::zero() + Decimal256::zero(), Decimal256::zero());
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn decimal_add_overflow_panics() {
        let _value = Decimal256::MAX + Decimal256::percent(50);
    }

    #[test]
    fn decimal_sub_works() {
        let value = Decimal256::one() - Decimal256::percent(50); // 0.5
        assert_eq!(value.0, Decimal256::DECIMAL_FRACTIONAL / 2i128);

        assert_eq!(
            Decimal256::percent(9) - Decimal256::percent(4),
            Decimal256::percent(5)
        );
        assert_eq!(
            Decimal256::percent(16) - Decimal256::zero(),
            Decimal256::percent(16)
        );
        assert_eq!(
            Decimal256::percent(16) - Decimal256::percent(16),
            Decimal256::zero()
        );
        assert_eq!(Decimal256::zero() - Decimal256::zero(), Decimal256::zero());
    }

    #[test]
    fn decimal_implements_mul() {
        let one = Decimal256::one();
        let two = one + one;
        let half = Decimal256::percent(50);

        // 1*x and x*1
        assert_eq!(one * Decimal256::percent(0), Decimal256::percent(0));
        assert_eq!(one * Decimal256::percent(1), Decimal256::percent(1));
        assert_eq!(one * Decimal256::percent(10), Decimal256::percent(10));
        assert_eq!(one * Decimal256::percent(100), Decimal256::percent(100));
        assert_eq!(one * Decimal256::percent(1000), Decimal256::percent(1000));
        // assert_eq!(one * Decimal256::MAX, Decimal256::MAX);
        assert_eq!(Decimal256::percent(0) * one, Decimal256::percent(0));
        assert_eq!(Decimal256::percent(1) * one, Decimal256::percent(1));
        assert_eq!(Decimal256::percent(10) * one, Decimal256::percent(10));
        assert_eq!(Decimal256::percent(100) * one, Decimal256::percent(100));
        assert_eq!(Decimal256::percent(1000) * one, Decimal256::percent(1000));
        // assert_eq!(Decimal256::MAX * one, Decimal256::MAX);

        // double
        assert_eq!(two * Decimal256::percent(0), Decimal256::percent(0));
        assert_eq!(two * Decimal256::percent(1), Decimal256::percent(2));
        assert_eq!(two * Decimal256::percent(10), Decimal256::percent(20));
        assert_eq!(two * Decimal256::percent(100), Decimal256::percent(200));
        assert_eq!(two * Decimal256::percent(1000), Decimal256::percent(2000));
        assert_eq!(Decimal256::percent(0) * two, Decimal256::percent(0));
        assert_eq!(Decimal256::percent(1) * two, Decimal256::percent(2));
        assert_eq!(Decimal256::percent(10) * two, Decimal256::percent(20));
        assert_eq!(Decimal256::percent(100) * two, Decimal256::percent(200));
        assert_eq!(Decimal256::percent(1000) * two, Decimal256::percent(2000));

        // half
        assert_eq!(half * Decimal256::percent(0), Decimal256::percent(0));
        assert_eq!(half * Decimal256::percent(1), Decimal256::permille(5));
        assert_eq!(half * Decimal256::percent(10), Decimal256::percent(5));
        assert_eq!(half * Decimal256::percent(100), Decimal256::percent(50));
        assert_eq!(half * Decimal256::percent(1000), Decimal256::percent(500));
        assert_eq!(Decimal256::percent(0) * half, Decimal256::percent(0));
        assert_eq!(Decimal256::percent(1) * half, Decimal256::permille(5));
        assert_eq!(Decimal256::percent(10) * half, Decimal256::percent(5));
        assert_eq!(Decimal256::percent(100) * half, Decimal256::percent(50));
        assert_eq!(Decimal256::percent(1000) * half, Decimal256::percent(500));
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn decimal_mul_overflow_panics() {
        let _value = Decimal256::MAX * Decimal256::percent(101);
    }

    #[test]
    // in this test the Decimal256 is on the right
    fn i128_decimal_multiply() {
        // a*b
        let left = 300i128;
        let right = Decimal256::one() + Decimal256::percent(50); // 1.5
        assert_eq!(left * right, 450i128);

        // a*0
        let left = 300i128;
        let right = Decimal256::zero();
        assert_eq!(left * right, 0i128);

        // 0*a
        let left = 0i128;
        let right = Decimal256::one() + Decimal256::percent(50); // 1.5
        assert_eq!(left * right, 0i128);

        assert_eq!(0i128 * Decimal256::one(), 0i128);
        assert_eq!(1i128 * Decimal256::one(), 1i128);
        assert_eq!(2i128 * Decimal256::one(), 2i128);

        assert_eq!(1i128 * Decimal256::percent(10), 0i128);
        assert_eq!(10i128 * Decimal256::percent(10), 1i128);
        assert_eq!(100i128 * Decimal256::percent(10), 10i128);

        assert_eq!(1i128 * Decimal256::percent(50), 0i128);
        assert_eq!(100i128 * Decimal256::percent(50), 50i128);
        assert_eq!(3200i128 * Decimal256::percent(50), 1600i128);
        assert_eq!(999i128 * Decimal256::percent(50), 499i128); // default rounding down

        assert_eq!(1i128 * Decimal256::percent(200), 2i128);
        assert_eq!(1000i128 * Decimal256::percent(200), 2000i128);
    }

    #[test]
    // in this test the Decimal256 is on the left
    fn decimal_i128_multiply() {
        // a*b
        let left = Decimal256::one() + Decimal256::percent(50); // 1.5
        let right = 300i128;
        assert_eq!(left * right, 450i128);

        // 0*a
        let left = Decimal256::zero();
        let right = 300i128;
        assert_eq!(left * right, 0i128);

        // a*0
        let left = Decimal256::one() + Decimal256::percent(50); // 1.5
        let right = 0i128;
        assert_eq!(left * right, 0i128);
    }

    #[test]
    fn decimal_implements_div() {
        let one = Decimal256::one();
        let two = one + one;
        let half = Decimal256::percent(50);

        // 1/x and x/1
        assert_eq!(one / Decimal256::percent(1), Decimal256::percent(10_000));
        assert_eq!(one / Decimal256::percent(10), Decimal256::percent(1_000));
        assert_eq!(one / Decimal256::percent(100), Decimal256::percent(100));
        assert_eq!(one / Decimal256::percent(1000), Decimal256::percent(10));
        assert_eq!(Decimal256::percent(0) / one, Decimal256::percent(0));
        assert_eq!(Decimal256::percent(1) / one, Decimal256::percent(1));
        assert_eq!(Decimal256::percent(10) / one, Decimal256::percent(10));
        assert_eq!(Decimal256::percent(100) / one, Decimal256::percent(100));
        assert_eq!(Decimal256::percent(1000) / one, Decimal256::percent(1000));

        // double
        assert_eq!(two / Decimal256::percent(1), Decimal256::percent(20_000));
        assert_eq!(two / Decimal256::percent(10), Decimal256::percent(2_000));
        assert_eq!(two / Decimal256::percent(100), Decimal256::percent(200));
        assert_eq!(two / Decimal256::percent(1000), Decimal256::percent(20));
        assert_eq!(Decimal256::percent(0) / two, Decimal256::percent(0));
        assert_eq!(Decimal256::percent(10) / two, Decimal256::percent(5));
        assert_eq!(Decimal256::percent(100) / two, Decimal256::percent(50));
        assert_eq!(Decimal256::percent(1000) / two, Decimal256::percent(500));

        // half
        assert_eq!(half / Decimal256::percent(1), Decimal256::percent(5_000));
        assert_eq!(half / Decimal256::percent(10), Decimal256::percent(500));
        assert_eq!(half / Decimal256::percent(100), Decimal256::percent(50));
        assert_eq!(half / Decimal256::percent(1000), Decimal256::percent(5));
        assert_eq!(Decimal256::percent(0) / half, Decimal256::percent(0));
        assert_eq!(Decimal256::percent(1) / half, Decimal256::percent(2));
        assert_eq!(Decimal256::percent(10) / half, Decimal256::percent(20));
        assert_eq!(Decimal256::percent(100) / half, Decimal256::percent(200));
        assert_eq!(Decimal256::percent(1000) / half, Decimal256::percent(2000));

        assert_eq!(
            Decimal256::percent(15) / Decimal256::percent(60),
            Decimal256::percent(25)
        );
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn decimal_div_overflow_panics() {
        let _value = Decimal256::MAX / Decimal256::percent(10);
    }

    #[test]
    #[should_panic(expected = "Division failed - denominator must not be zero")]
    fn decimal_div_by_zero_panics() {
        let _value = Decimal256::one() / Decimal256::zero();
    }

    #[test]
    fn decimal_i128_division() {
        // a/b
        let left = Decimal256::percent(150); // 1.5
        let right = 3i128;
        assert_eq!(left / right, Decimal256::percent(50));

        // 0/a
        let left = Decimal256::zero();
        let right = 300i128;
        assert_eq!(left / right, Decimal256::zero());
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn decimal_uint128_divide_by_zero() {
        let left = Decimal256::percent(150); // 1.5
        let right = 0i128;
        let _result = left / right;
    }

    #[test]
    fn decimal_pow_works() {
        assert_eq!(Decimal256::percent(200).pow(2), Decimal256::percent(400));
        assert_eq!(Decimal256::percent(100).pow(10), Decimal256::percent(100));
    }

    #[test]
    #[should_panic]
    fn decimal_pow_overflow_panics() {
        _ = Decimal256::MAX.pow(2u32);
    }

    #[test]
    fn decimal_abs_with_negative_number() {
        let decimal = Decimal256::new(128);

        assert_eq!(decimal.abs(), Decimal256(128));
    }

    #[test]
    fn decimal_abs_with_positive_number() {
        let decimal = Decimal256::new(128);

        assert_eq!(decimal.abs(), Decimal256(128));
    }

    #[test]
    fn decimal_displayed_as_string() {
        let env = Env::default();
        let decimal = Decimal256::percent(128);

        // Convert expected string to Soroban SDK String
        let expected_msg = "1.28";
        let expected_string = String::from_str(&env, expected_msg);

        // Convert decimal to String and get its byte representation
        let result_string = decimal.to_string(&env);
        let result_string_len = result_string.len() as usize;
        let mut result_bytes = alloc::vec![0u8; result_string_len];
        result_string.copy_into_slice(&mut result_bytes);

        // Get byte representation of expected string
        let expected_string_len = expected_string.len() as usize;
        let mut expected_bytes = alloc::vec![0u8; expected_string_len];
        expected_string.copy_into_slice(&mut expected_bytes);

        assert_eq!(result_bytes, expected_bytes);
    }

    #[test]
    fn decimal_fmt_without_fractional_part() {
        let value = Decimal256::from_atomics(100, 0);
        assert_eq!(format!("{}", value), "100");
    }

    #[test]
    fn decimal_fmt_fractional_part() {
        let value = Decimal256::from_atomics(123456789, 5);
        assert_eq!(format!("{}", value), "1234.56789");
    }

    #[test]
    fn decimal_fmt_fractional_part_with_trailing_zeros() {
        // 12345.6
        let value = Decimal256::from_atomics(123456, 1);
        assert_eq!(format!("{}", value), "12345.6");
    }

    #[test]
    fn decimal_fmt_only_fractional() {
        // 0.0789
        let value = Decimal256::from_atomics(789, 4);
        assert_eq!(format!("{}", value), "0.0789");
    }
}
