use crate::charutils::is_structural_or_whitespace;
use crate::error::Error;
use crate::numberconst::*;
use crate::unlikely;
use crate::StaticNode;
use crate::*;

#[cfg(all(target_arch = "x86", feature = "swar-number-parsing"))]
use std::arch::x86::*;
#[cfg(all(target_arch = "x86_64", feature = "swar-number-parsing"))]
use std::arch::x86_64::*;

macro_rules! get {
    ($buf:ident, $idx:expr) => {
        unsafe { *$buf.get_unchecked($idx as usize) }
    };
}
macro_rules! err {
    ($idx:ident, $num:expr) => {
        return Err(Error::new($idx, $num as char, ErrorType::InvalidNumber));
    };
}

#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn is_integer(c: u8) -> bool {
    c.wrapping_sub(b'0') <= 9
}

// We need to check that the character following a zero is valid. This is
// probably frequent and it is hard than it looks. We are building all of this
// just to differentiate between 0x1 (invalid), 0,1 (valid) 0e1 (valid)...
const STRUCTURAL_OR_WHITESPACE_OR_EXPONENT_OR_DECIMAL_NEGATED: [bool; 256] = [
    false, true, true, true, true, true, true, true, true, false, false, true, true, false, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, false, true, true, true, true, true, true, true, true, true, true, true, false, true,
    false, true, true, true, true, true, true, true, true, true, true, true, false, true, true,
    true, true, true, true, true, true, true, true, false, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    false, true, false, true, true, true, true, true, true, true, false, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, false, true, false, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true,
];

#[cfg_attr(not(feature = "no-inline"), inline(always))]
fn is_not_structural_or_whitespace_or_exponent_or_decimal(c: u8) -> bool {
    get!(STRUCTURAL_OR_WHITESPACE_OR_EXPONENT_OR_DECIMAL_NEGATED, c)
}

// check quickly whether the next 8 chars are made of digits
// at a glance, it looks better than Mula's
// http://0x80.pl/articles/swar-digits-validate.html

#[cfg(feature = "swar-number-parsing")]
#[cfg_attr(not(feature = "no-inline"), inline)]
#[allow(clippy::cast_ptr_alignment)]
fn is_made_of_eight_digits_fast(chars: &[u8]) -> bool {
    // We know what we're doing right? :P
    let val: u64 = unsafe { *(chars.as_ptr() as *const u64) };
    ((val & 0xF0F0_F0F0_F0F0_F0F0)
        | (((val.wrapping_add(0x0606_0606_0606_0606)) & 0xF0F0_F0F0_F0F0_F0F0) >> 4))
        == 0x3333_3333_3333_3333
}

#[cfg_attr(not(feature = "no-inline"), inline)]
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    feature = "swar-number-parsing"
))]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_ptr_alignment
)]
fn parse_eight_digits_unrolled(chars: &[u8]) -> u32 {
    unsafe {
        // this actually computes *16* values so we are being wasteful.
        let ascii0: __m128i = _mm_set1_epi8(b'0' as i8);
        let mul_1_10: __m128i =
            _mm_setr_epi8(10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1);
        let mul_1_100: __m128i = _mm_setr_epi16(100, 1, 100, 1, 100, 1, 100, 1);
        let mul_1_10000: __m128i = _mm_setr_epi16(10000, 1, 10000, 1, 10000, 1, 10000, 1);
        // We know what we're doing right? :P
        let input: __m128i = _mm_sub_epi8(
            _mm_loadu_si128(chars.get_unchecked(0..16).as_ptr() as *const __m128i),
            ascii0,
        );
        let t1: __m128i = _mm_maddubs_epi16(input, mul_1_10);
        let t2: __m128i = _mm_madd_epi16(t1, mul_1_100);
        let t3: __m128i = _mm_packus_epi32(t2, t2);
        let t4: __m128i = _mm_madd_epi16(t3, mul_1_10000);
        _mm_cvtsi128_si32(t4) as u32 // only captures the sum of the first 8 digits, drop the rest
    }
}

#[cfg_attr(not(feature = "no-inline"), inline)]
#[cfg(target_feature = "neon")]
fn parse_eight_digits_unrolled(chars: &[u8]) -> u32 {
    let val: u64 = unsafe { *(chars.as_ptr() as *const u64) };
    let val = (val & 0x0F0F0F0F0F0F0F0F).wrapping_mul(2561) >> 8;
    let val = (val & 0x00FF00FF00FF00FF).wrapping_mul(6553601) >> 16;
    return ((val & 0x0000FFFF0000FFFF).wrapping_mul(42949672960001) >> 32) as u32;
}

#[cfg_attr(not(feature = "no-inline"), inline)]
fn multiply_as_u128(a: u64, b: u64) -> (u64, u64) {
    let res: u128 = a as u128 * b as u128;
    (res as u64, (res >> 64) as u64)
}

impl<'de> Deserializer<'de> {
    #[allow(unused_unsafe)]
    #[allow(clippy::unreadable_literal)]
    pub(crate) fn parse_number(start_idx: usize, buf: &[u8], negative: bool) -> Result<StaticNode> {
        let mut idx = start_idx;
        if negative {
            idx += 1;
            if !is_integer(get!(buf, idx)) {
                err!(idx, get!(buf, idx))
            }
        }
        let mut start = idx;
        let mut num: u64 = 0;
        if get!(buf, idx) == b'0' {
            idx += 1;
            if is_not_structural_or_whitespace_or_exponent_or_decimal(get!(buf, idx)) {
                err!(idx, get!(buf, idx))
            }
        } else {
            if !is_integer(get!(buf, idx)) {
                err!(idx, get!(buf, idx))
            }
            num = (get!(buf, idx) - b'0') as u64;
            idx += 1;
            while is_integer(get!(buf, idx)) {
                num = 10_u64
                    .wrapping_mul(num)
                    .wrapping_add((get!(buf, idx) - b'0') as u64);
                idx += 1;
            }
        }
        let mut exponent: i64 = 0;
        let mut is_float = false;
        if get!(buf, idx) == b'.' {
            is_float = true;
            idx += 1;
            let first_after_period = idx as i64;
            if is_integer(get!(buf, idx)) {
                num = 10_u64.wrapping_mul(num) + ((get!(buf, idx) - b'0') as u64);
                idx += 1;
            } else {
                err!(idx, get!(buf, idx))
            }

            #[cfg(feature = "swar-number-parsing")]
            {
                if is_made_of_eight_digits_fast(&buf[idx..]) {
                    num = 100000000_u64
                        .wrapping_mul(num)
                        .wrapping_add(parse_eight_digits_unrolled(&buf[idx..]) as u64);
                    idx += 8;
                }
            }
            while is_integer(get!(buf, idx)) {
                num = 10_u64.wrapping_mul(num) + ((get!(buf, idx) - b'0') as u64);
                idx += 1;
            }
            exponent = first_after_period.wrapping_sub(idx as i64);
        }
        let mut digit_count = idx - start_idx - 1;
        match get!(buf, idx) {
            b'e' | b'E' => {
                is_float = true;
                idx += 1;
                let neg_exp: bool;
                match get!(buf, idx) {
                    b'-' => {
                        neg_exp = true;
                        idx += 1;
                    }
                    b'+' => {
                        neg_exp = false;
                        idx += 1;
                    }
                    _ => {
                        neg_exp = false;
                    }
                }
                if !is_integer(get!(buf, idx)) {
                    err!(idx, get!(buf, idx))
                }
                let mut exp_number = (get!(buf, idx) - b'0') as i64;
                idx += 1;
                if is_integer(get!(buf, idx)) {
                    exp_number = 10 * exp_number + ((get!(buf, idx) - b'0') as i64);
                    idx += 1;
                }
                if is_integer(get!(buf, idx)) {
                    exp_number = 10 * exp_number + ((get!(buf, idx) - b'0') as i64);
                    idx += 1;
                }
                while is_integer(get!(buf, idx)) {
                    if exp_number > 0x100000000 {
                        err!(idx, get!(buf, idx))
                    }
                    exp_number = 10 * exp_number + ((get!(buf, idx) - b'0') as i64);
                    idx += 1;
                }
                exponent += if neg_exp { -exp_number } else { exp_number };
            }
            _ => {}
        }
        if is_float {
            if unlikely!(digit_count >= 19) {
                let start_digits = get!(buf, start) as usize;
                while get!(buf, start) == b'0' || get!(buf, start) == b'.' {
                    start += 1;
                }
                digit_count = digit_count.wrapping_sub((start.wrapping_sub(start_digits)) as usize);
                if digit_count >= 19 {
                    return f64_from_parts_slow(&buf[start_idx..idx], start_idx);
                }
            }
            if is_structural_or_whitespace(get!(buf, idx)) == 0 {
                err!(idx, get!(buf, idx))
            }
            f64_from_parts(
                !negative,
                num,
                exponent as i32,
                &buf[start_idx..idx],
                start_idx,
            )
        } else if unlikely!(digit_count >= 18) {
            parse_large_integer(start_idx, buf, negative)
        } else if is_structural_or_whitespace(get!(buf, idx)) == 0 {
            err!(idx, get!(buf, idx))
        } else {
            Ok(StaticNode::I64(if negative {
                unsafe { static_cast_i64!(num.wrapping_neg()) } // -(num as i64)
            } else {
                num as i64
            }))
        }
    }
}

#[cold]
#[allow(clippy::unreadable_literal)]
fn parse_large_integer(start_idx: usize, buf: &[u8], negative: bool) -> Result<StaticNode> {
    let mut idx = start_idx;
    if negative {
        idx += 1;
    }
    let mut num: u64 = 0;
    if get!(buf, idx) == b'0' {
        idx += 1;
    } else {
        num = (get!(buf, idx) - b'0') as u64;
        idx += 1;
        while is_integer(get!(buf, idx)) {
            let digit = (get!(buf, idx) - b'0') as u64;
            {
                let (res, overflowed) = 10_u64.overflowing_mul(num);
                if overflowed {
                    err!(idx, get!(buf, idx))
                }
                num = res;
            }
            {
                let (res, overflowed) = num.overflowing_add(digit);
                if overflowed {
                    err!(idx, get!(buf, idx))
                }
                num = res;
            }
            idx += 1;
        }
    }
    match (negative, num) {
        (true, 9223372036854775808) => Ok(StaticNode::I64(i64::MIN)),
        (true, 9223372036854775809..=u64::MAX) => err!(idx, get!(buf, idx)),
        (true, 0..=9223372036854775807) => Ok(StaticNode::I64(-(num as i64))),
        (false, 0..=9223372036854775807) => Ok(StaticNode::I64(num as i64)),
        (false, _) => Ok(StaticNode::U64(num)),
    }
}

#[allow(clippy::unreadable_literal)]
fn f64_from_parts(
    positive: bool,
    significand: u64,
    exponent: i32,
    slice: &[u8],
    offset: usize,
) -> Result<StaticNode> {
    if -22 <= exponent && exponent <= 22 && significand <= 9007199254740991 {
        let mut f = significand as f64;
        if exponent < 0 {
            f = f / get!(POW10, -exponent);
        } else {
            f = f * get!(POW10, exponent);
        }
        Ok(StaticNode::F64(if positive { f } else { -f }))
    } else if significand == 0 {
        Ok(StaticNode::F64(if positive { 0.0 } else { -0.0 }))
    } else if exponent >= -325 && exponent <= 308 {
        let (factor_mantissa, factor_exponent) = get!(POW10_COMPONENTS, exponent + 325);
        let mut leading_zeroes = significand.leading_zeros() as u64;
        let f = significand << leading_zeroes;
        let (mut lower, mut upper) = multiply_as_u128(f, factor_mantissa);
        if upper & 0x1FF == 0x1FF && lower.wrapping_add(f) < lower {
            let factor_mantissa_low = get!(MANTISSA_128, exponent + 325);
            let (product_low, product_middle2) = multiply_as_u128(f, factor_mantissa_low);
            let product_middle1 = lower;
            let mut product_high = upper;
            let product_middle = product_middle1.wrapping_add(product_middle2);
            if product_middle < product_middle1 {
                product_high += 1;
            }
            if product_middle.wrapping_add(1) == 0
                && product_high & 0x1FF == 0x1FF
                && product_low.wrapping_add(f) < product_low
            {
                return f64_from_parts_slow(slice, offset);
            }
            upper = product_high;
            lower = product_middle;
        }
        let upperbit = upper.wrapping_shr(63);
        let mut mantissa = upper.wrapping_shr((upperbit + 9) as u32);
        leading_zeroes += 1 ^ upperbit;

        if lower == 0 && upper & 0x1FF == 0 && mantissa & 3 == 1 {
            return f64_from_parts_slow(slice, offset);
        }
        mantissa += mantissa & 1;
        mantissa >>= 1;

        if mantissa >= 1 << 53 {
            mantissa = 1 << 52;
            leading_zeroes -= 1;
        }
        mantissa &= !(1 << 52);
        let real_exponent = (factor_exponent as u64).wrapping_sub(leading_zeroes);
        // we have to check that real_exponent is in range, otherwise we bail out
        if real_exponent < 1 || real_exponent > 2046 {
            return f64_from_parts_slow(slice, offset);
        }
        mantissa |= real_exponent.wrapping_shl(52);
        mantissa |= (!positive as u64) << 63;
        let res = f64::from_bits(mantissa);
        if res.is_infinite() {
            err!(offset, get!(slice, offset))
        } else {
            Ok(StaticNode::F64(res))
        }
    } else {
        f64_from_parts_slow(slice, offset)
    }
}

#[cold]
fn f64_from_parts_slow(slice: &[u8], offset: usize) -> Result<StaticNode> {
    match lexical_core::parse_format::<f64>(slice, lexical_core::NumberFormat::JSON) {
        Ok(val) => {
            if val.is_infinite() {
                err!(offset, get!(slice, offset))
            } else {
                Ok(StaticNode::F64(val))
            }
        }
        Err(_) => err!(offset, get!(slice, offset)),
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::default_trait_access)]
    use crate::error::Error;
    use crate::value::owned::to_value;
    use crate::value::owned::Value;
    use crate::value::owned::Value::Static;
    use value_trait::StaticNode::F64;
    use value_trait::StaticNode::I64;

    fn to_value_from_str(buf: &str) -> Result<Value, Error> {
        let mut val = String::from(buf);
        let mut val = unsafe { val.as_bytes_mut() };
        to_value(&mut val)
    }

    #[test]
    fn float() {
        assert_eq!(
            to_value_from_str("0.4e5").expect("40000.0"),
            Static(F64(40000.0))
        );
        assert_eq!(
            to_value_from_str("-12345678901234.56789012").unwrap(),
            Static(F64(-12345678901234.56789012))
        );
        assert_eq!(to_value_from_str("0.4e-001").unwrap(), Static(F64(0.04)));
        assert_eq!(
            to_value_from_str("0.123456789e-12").unwrap(),
            Static(F64(1.23456789e-13))
        );
        assert_eq!(to_value_from_str("1.234567890E+34").unwrap(), 1.23456789e34);
        assert_eq!(
            to_value_from_str("23456789012E66").unwrap(),
            Static(F64(2.3456789012e76))
        );
        assert_eq!(
            to_value_from_str("0.0000000000000000000000000000000000000000000000000123e50")
                .expect("1.23"),
            Static(F64(1.23))
        );
    }

    #[test]
    fn float_precision() {
        assert_eq!(
            to_value_from_str("31.245270191439438").unwrap(),
            31.245270191439438
        );
        assert_eq!(
            to_value_from_str("-31.245270191439438").unwrap(),
            -31.245270191439438
        );
        assert_eq!(
            to_value_from_str("121.48791951161945").unwrap(),
            121.48791951161945
        );
        assert_eq!(
            to_value_from_str("-121.48791951161945").unwrap(),
            -121.48791951161945
        );
        assert_eq!(
            to_value_from_str("100.78399658203125").unwrap(),
            100.78399658203125
        );
        assert_eq!(
            to_value_from_str("-100.78399658203125").unwrap(),
            -100.78399658203125
        );
    }

    #[test]
    fn int_trailing_invalid() {
        // todo: these should fail but is not distinguished from trailing padding
        assert!(to_value_from_str("123\x00").is_ok());
        assert!(to_value_from_str("[123\x00]").is_ok());
    }

    #[test]
    fn float_trailing_invalid() {
        assert!(to_value_from_str("0.1.2").is_err());
        assert!(to_value_from_str("[0.1.2]").is_err());
        assert!(to_value_from_str("[-1.0.]").is_err());
        assert!(to_value_from_str("[1.2a-3]").is_err());
        assert!(to_value_from_str("[1.8011670033376514H-308]").is_err());
    }

    #[test]
    fn bad_dot() {
        assert!(to_value_from_str("1.").is_err());
        assert!(to_value_from_str("1.e").is_err());
        assert!(to_value_from_str("100000000000000000000000000000000000000000000.").is_err());
        assert!(to_value_from_str("100000000000000000000000000000000000000000000.e").is_err());
    }

    #[test]
    fn bad_e() {
        assert!(to_value_from_str("1.0e").is_err());
        assert!(to_value_from_str("100000000000000000000000000000000000000000000.0e").is_err());
        assert!(to_value_from_str("100000000000000000000000000000000000000000000.0ee").is_err());
    }

    #[test]
    fn infinite_literal() {
        assert!(to_value_from_str("inf").is_err());
        assert!(to_value_from_str("Inf").is_err());
        assert!(to_value_from_str("-inf").is_err());
        assert!(to_value_from_str("-Inf").is_err());
        assert!(to_value_from_str("infinity").is_err());
        assert!(to_value_from_str("Infinity").is_err());
        assert!(to_value_from_str("-infinity").is_err());
        assert!(to_value_from_str("-Infinity").is_err());
    }

    #[test]
    fn infinite_exponent() {
        assert!(to_value_from_str("1e309").is_err());
        assert!(to_value_from_str("1e1000").is_err());
        assert!(to_value_from_str("100000000000000000000000000000000000000000000e309").is_err());
        assert!(to_value_from_str("100000000000000000000000000000000000000000000e1000").is_err());
    }

    #[test]
    fn nan() {
        assert!(to_value_from_str("NaN").is_err());
        assert!(to_value_from_str("nan").is_err());
    }

    #[test]
    fn zero_int() {
        assert_eq!(to_value_from_str("0").expect("0"), Static(I64(0)));
    }

    #[test]
    fn zero_float() {
        assert_eq!(to_value_from_str("0e1").expect("0e1"), Static(F64(0.0)));
        assert_eq!(to_value_from_str("0.00e-00").unwrap(), Static(F64(0.0)));
        assert_eq!(to_value_from_str("0e-1").expect("0e-1"), Static(F64(-0.0)));
        assert_eq!(to_value_from_str("-0.00e-00").unwrap(), Static(F64(-0.0)));
    }

    #[test]
    fn int() {
        assert_eq!(to_value_from_str("1").unwrap(), Static(I64(1)));
        assert_eq!(to_value_from_str("257").unwrap(), Static(I64(257)));
    }

    #[test]
    fn minus_309() {
        assert_eq!(
            to_value_from_str("-5.96916642387374e-309").unwrap(),
            Static(F64(-5.96916642387374e-309))
        );
    }
    #[allow(clippy::unreadable_literal)]
    #[test]
    fn tiny_float() {
        assert_eq!(to_value_from_str("-0.00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000596916642387374").unwrap(), Static(F64(-0.00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000596916642387374)));
    }
}
