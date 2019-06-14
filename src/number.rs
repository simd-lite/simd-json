// Taken from json-rust
// https://github.com/maciejhirsz/json-rust/blob/677a590e5f8680e3251ab3a8fe07fe144fa4c4f9/src/number.rs

use crate::util::grisu2;
use crate::util::print_dec;
use std::num::FpCategory;
use std::{f32, f64, fmt, ops};

/// NaN value represented in `Number` type. NaN is equal to itself.
pub const NAN: Number = Number {
    category: NAN_MASK,
    mantissa: 0,
    exponent: 0,
};

const NEGATIVE: u8 = 0;
const POSITIVE: u8 = 1;
const NAN_MASK: u8 = !1;

/// Number representation used inside `JsonValue`. You can easily convert
/// the `Number` type into native Rust number types and back, or use the
/// equality operator with another number type.
///
/// ```
/// # use json::number::Number;
/// let foo: Number = 3.14.into();
/// let bar: f64 = foo.into();
///
/// assert_eq!(foo, 3.14);
/// assert_eq!(bar, 3.14);
/// ```
///
/// More often than not you will deal with `JsonValue::Number` variant that
/// wraps around this type, instead of using the methods here directly.
#[derive(Copy, Clone, Debug)]
pub struct Number {
    // A byte describing the sign and NaN-ness of the number.
    //
    // category == 0 (NEGATIVE constant)         -> negative sign
    // category == 1 (POSITIVE constant)         -> positive sign
    // category >  1 (matches NAN_MASK constant) -> NaN
    category: u8,

    // Decimal exponent, analog to `e` notation in string form.
    exponent: i16,

    // Integer base before sing and exponent applied.
    mantissa: u64,
}

impl Number {
    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Number {
                category: 0,
                exponent: 0,
                mantissa,
            } => Some(-(*mantissa as i64)),
            Number {
                category: 1,
                exponent: 0,
                mantissa,
            } => Some(*mantissa as i64),
            _ => None,
        }
    }

    #[inline]
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Number {
                category: 1,
                exponent: 0,
                mantissa,
            } => Some(*mantissa),
            _ => None,
        }
    }

    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        match self.category {
            0 => Some(exponentiate_f64(self.mantissa as f64, self.exponent)),
            1 => Some(-exponentiate_f64(self.mantissa as f64, self.exponent)),
            _ => None,
        }
    }

    /// Construct a new `Number` from parts. This can't create a NaN value.
    ///
    /// ```
    /// # use json::number::Number;
    /// let pi = unsafe { Number::from_parts_unchecked(true, 3141592653589793, -15) };
    ///
    /// assert_eq!(pi, 3.141592653589793);
    /// ```
    ///
    /// While this method is marked unsafe, it doesn't actually perform any unsafe operations.
    /// THe goal of the 'unsafe' is to deter from using this method in favor of its safe equivalent
    /// `from_parts`, at least in context when the associated performance cost is negligible.
    #[inline]
    pub unsafe fn from_parts_unchecked(positive: bool, mantissa: u64, exponent: i16) -> Self {
        Number {
            category: positive as u8,
            exponent: exponent,
            mantissa: mantissa,
        }
    }

    /// Construct a new `Number` from parts, stripping unnecessary trailing zeroes.
    /// This can't create a NaN value.
    ///
    /// ```
    /// # use json::number::Number;
    /// let one = Number::from_parts(true, 1000, -3);
    /// let (positive, mantissa, exponent) = one.as_parts();
    ///
    /// assert_eq!(true, positive);
    /// assert_eq!(1, mantissa);
    /// assert_eq!(0, exponent);
    /// ```
    #[inline]
    pub fn from_parts(positive: bool, mut mantissa: u64, mut exponent: i16) -> Self {
        while exponent < 0 && mantissa % 10 == 0 {
            exponent += 1;
            mantissa /= 10;
        }
        unsafe { Number::from_parts_unchecked(positive, mantissa, exponent) }
    }

    /// Reverse to `from_parts` - obtain parts from an existing `Number`.
    ///
    /// ```
    /// # use json::number::Number;
    /// let pi = Number::from(3.141592653589793);
    /// let (positive, mantissa, exponent) = pi.as_parts();
    ///
    /// assert_eq!(positive, true);
    /// assert_eq!(mantissa, 3141592653589793);
    /// assert_eq!(exponent, -15);
    /// ```
    #[inline]
    pub fn as_parts(&self) -> (bool, u64, i16) {
        (self.category == POSITIVE, self.mantissa, self.exponent)
    }

    #[inline]
    pub fn is_sign_positive(&self) -> bool {
        self.category == POSITIVE
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.mantissa == 0 && !self.is_nan()
    }

    #[inline]
    pub fn is_nan(&self) -> bool {
        self.category & NAN_MASK != 0
    }

    /// Test if the number is NaN or has a zero value.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.mantissa == 0 || self.is_nan()
    }

    /// Obtain an integer at a fixed decimal point. This is useful for
    /// converting monetary values and doing arithmetic on them without
    /// rounding errors introduced by floating point operations.
    ///
    /// Will return `None` if `Number` is negative or a NaN.
    ///
    /// ```
    /// # use json::number::Number;
    /// let price_a = Number::from(5.99);
    /// let price_b = Number::from(7);
    /// let price_c = Number::from(10.2);
    ///
    /// assert_eq!(price_a.as_fixed_point_u64(2), Some(599));
    /// assert_eq!(price_b.as_fixed_point_u64(2), Some(700));
    /// assert_eq!(price_c.as_fixed_point_u64(2), Some(1020));
    /// ```
    pub fn as_fixed_point_u64(&self, point: u16) -> Option<u64> {
        if self.category != POSITIVE {
            return None;
        }

        let e_diff = point as i16 + self.exponent;

        Some(if e_diff == 0 {
            self.mantissa
        } else if e_diff < 0 {
            self.mantissa.wrapping_div(decimal_power(-e_diff as u16))
        } else {
            self.mantissa.wrapping_mul(decimal_power(e_diff as u16))
        })
    }

    /// Analog to `as_fixed_point_u64`, except returning a signed
    /// `i64`, properly handling negative numbers.
    ///
    /// ```
    /// # use json::number::Number;
    /// let balance_a = Number::from(-1.49);
    /// let balance_b = Number::from(42);
    ///
    /// assert_eq!(balance_a.as_fixed_point_i64(2), Some(-149));
    /// assert_eq!(balance_b.as_fixed_point_i64(2), Some(4200));
    /// ```
    pub fn as_fixed_point_i64(&self, point: u16) -> Option<i64> {
        if self.is_nan() {
            return None;
        }

        let num = if self.is_sign_positive() {
            self.mantissa as i64
        } else {
            -(self.mantissa as i64)
        };

        let e_diff = point as i16 + self.exponent;

        Some(if e_diff == 0 {
            num
        } else if e_diff < 0 {
            num.wrapping_div(decimal_power(-e_diff as u16) as i64)
        } else {
            num.wrapping_mul(decimal_power(e_diff as u16) as i64)
        })
    }
}

impl PartialEq for Number {
    #[inline]
    fn eq(&self, other: &Number) -> bool {
        if self.is_zero() && other.is_zero() || self.is_nan() && other.is_nan() {
            return true;
        }

        if self.category != other.category {
            return false;
        }

        let e_diff = self.exponent - other.exponent;

        if e_diff == 0 {
            return self.mantissa == other.mantissa;
        } else if e_diff > 0 {
            let power = decimal_power(e_diff as u16);

            self.mantissa.wrapping_mul(power) == other.mantissa
        } else {
            let power = decimal_power(-e_diff as u16);

            self.mantissa == other.mantissa.wrapping_mul(power)
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            if self.is_nan() {
                return f.write_str("nan");
            }
            let (positive, mantissa, exponent) = self.as_parts();
            let mut buf = Vec::new();
            print_dec::write(&mut buf, positive, mantissa, exponent).unwrap();
            f.write_str(&String::from_utf8_unchecked(buf))
        }
    }
}

fn exponentiate_f64(n: f64, e: i16) -> f64 {
    static CACHE_POWERS: [f64; 23] = [
        1.0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15, 1e16,
        1e17, 1e18, 1e19, 1e20, 1e21, 1e22,
    ];

    if e >= 0 {
        let index = e as usize;

        n * if index < 23 {
            CACHE_POWERS[index]
        } else {
            10f64.powf(index as f64)
        }
    } else {
        let index = -e as usize;

        n / if index < 23 {
            CACHE_POWERS[index]
        } else {
            10f64.powf(index as f64)
        }
    }
}

fn exponentiate_f32(n: f32, e: i16) -> f32 {
    static CACHE_POWERS: [f32; 23] = [
        1.0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15, 1e16,
        1e17, 1e18, 1e19, 1e20, 1e21, 1e22,
    ];

    if e >= 0 {
        let index = e as usize;

        n * if index < 23 {
            CACHE_POWERS[index]
        } else {
            10f32.powf(index as f32)
        }
    } else {
        let index = -e as usize;

        n / if index < 23 {
            CACHE_POWERS[index]
        } else {
            10f32.powf(index as f32)
        }
    }
}

impl From<Number> for f64 {
    fn from(num: Number) -> f64 {
        if num.is_nan() {
            return f64::NAN;
        }

        let mut n = num.mantissa as f64;
        let mut e = num.exponent;

        if e < -308 {
            n = exponentiate_f64(n, e + 308);
            e = -308;
        }

        let f = exponentiate_f64(n, e);
        if num.is_sign_positive() {
            f
        } else {
            -f
        }
    }
}

impl From<Number> for f32 {
    fn from(num: Number) -> f32 {
        if num.is_nan() {
            return f32::NAN;
        }

        let mut n = num.mantissa as f32;
        let mut e = num.exponent;

        if e < -127 {
            n = exponentiate_f32(n, e + 127);
            e = -127;
        }

        let f = exponentiate_f32(n, e);
        if num.is_sign_positive() {
            f
        } else {
            -f
        }
    }
}

impl From<f64> for Number {
    fn from(float: f64) -> Number {
        match float.classify() {
            FpCategory::Infinite | FpCategory::Nan => return NAN,
            _ => {}
        }

        if !float.is_sign_positive() {
            let (mantissa, exponent) = grisu2::convert(-float);

            Number::from_parts(false, mantissa, exponent)
        } else {
            let (mantissa, exponent) = grisu2::convert(float);

            Number::from_parts(true, mantissa, exponent)
        }
    }
}

impl From<f32> for Number {
    fn from(float: f32) -> Number {
        match float.classify() {
            FpCategory::Infinite | FpCategory::Nan => return NAN,
            _ => {}
        }

        if !float.is_sign_positive() {
            let (mantissa, exponent) = grisu2::convert(-float as f64);

            Number::from_parts(false, mantissa, exponent)
        } else {
            let (mantissa, exponent) = grisu2::convert(float as f64);

            Number::from_parts(true, mantissa, exponent)
        }
    }
}

impl PartialEq<f64> for Number {
    fn eq(&self, other: &f64) -> bool {
        f64::from(*self) == *other
    }
}

impl PartialEq<f32> for Number {
    fn eq(&self, other: &f32) -> bool {
        f32::from(*self) == *other
    }
}

impl PartialEq<Number> for f64 {
    fn eq(&self, other: &Number) -> bool {
        f64::from(*other) == *self
    }
}

impl PartialEq<Number> for f32 {
    fn eq(&self, other: &Number) -> bool {
        f32::from(*other) == *self
    }
}

macro_rules! impl_unsigned {
    ($( $t:ty ),*) => ($(
        impl From<$t> for Number {
            #[inline]
            fn from(num: $t) -> Number {
                Number {
                    category: POSITIVE,
                    exponent: 0,
                    mantissa: num as u64,
                }
            }
        }

        impl_integer!($t);
    )*)
}

macro_rules! impl_signed {
    ($( $t:ty ),*) => ($(
        impl From<$t> for Number {
            fn from(num: $t) -> Number {
                if num < 0 {
                    Number {
                        category: NEGATIVE,
                        exponent: 0,
                        mantissa: -num as u64,
                    }
                } else {
                    Number {
                        category: POSITIVE,
                        exponent: 0,
                        mantissa: num as u64,
                    }
                }
            }
        }

        impl_integer!($t);
    )*)
}

macro_rules! impl_integer {
    ($t:ty) => {
        impl From<Number> for $t {
            fn from(num: Number) -> $t {
                let (positive, mantissa, exponent) = num.as_parts();

                if exponent <= 0 {
                    if positive {
                        mantissa as $t
                    } else {
                        -(mantissa as i64) as $t
                    }
                } else {
                    // This may overflow, which is fine
                    if positive {
                        (mantissa * 10u64.pow(exponent as u32)) as $t
                    } else {
                        (-(mantissa as i64) * 10i64.pow(exponent as u32)) as $t
                    }
                }
            }
        }

        impl PartialEq<$t> for Number {
            fn eq(&self, other: &$t) -> bool {
                *self == Number::from(*other)
            }
        }

        impl PartialEq<Number> for $t {
            fn eq(&self, other: &Number) -> bool {
                Number::from(*self) == *other
            }
        }
    };
}

impl_signed!(isize, i8, i16, i32, i64);
impl_unsigned!(usize, u8, u16, u32, u64);

impl ops::Neg for Number {
    type Output = Number;

    #[inline]
    fn neg(self) -> Number {
        Number {
            category: self.category ^ POSITIVE,
            exponent: self.exponent,
            mantissa: self.mantissa,
        }
    }
}

// Commented out for now - not doing math ops for 0.10.0
// -----------------------------------------------------
//
// impl ops::Mul for Number {
//     type Output = Number;

//     #[inline]
//     fn mul(self, other: Number) -> Number {
//         // If either is a NaN, return a NaN
//         if (self.category | other.category) & NAN_MASK != 0 {
//             NAN
//         } else {
//             Number {
//                 // If both signs are the same, xoring will produce 0.
//                 // If they are different, xoring will produce 1.
//                 // Xor again with 1 to get a proper proper sign!
//                 // Xor all the things!                              ^ _ ^

//                 category: self.category ^ other.category ^ POSITIVE,
//                 exponent: self.exponent + other.exponent,
//                 mantissa: self.mantissa * other.mantissa,
//             }
//         }
//     }
// }

// impl ops::MulAssign for Number {
//     #[inline]
//     fn mul_assign(&mut self, other: Number) {
//         *self = *self * other;
//     }
// }

#[inline]
fn decimal_power(mut e: u16) -> u64 {
    static CACHED: [u64; 20] = [
        1,
        10,
        100,
        1000,
        10000,
        100000,
        1000000,
        10000000,
        100000000,
        1000000000,
        10000000000,
        100000000000,
        1000000000000,
        10000000000000,
        100000000000000,
        1000000000000000,
        10000000000000000,
        100000000000000000,
        1000000000000000000,
        10000000000000000000,
    ];

    if e < 20 {
        CACHED[e as usize]
    } else {
        let mut pow = 1u64;
        while e >= 20 {
            pow = pow.saturating_mul(CACHED[(e % 20) as usize]);
            e /= 20;
        }

        pow
    }
}
