mod charutils;
pub mod halfbrown;
#[macro_use]
mod macros;
mod error;
mod numberparse;
mod parsedjson;
mod portability;
pub mod serde;
mod stage1;
mod stage2;
mod stringparse;
mod utf8check;
pub mod value;

extern crate serde as serde_ext;
#[macro_use]
extern crate lazy_static;

use crate::numberparse::Number;
use crate::portability::*;
use crate::stage2::*;
use crate::stringparse::*;
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::mem;
use std::str;

pub use error::{Error, ErrorType};
pub use halfbrown::HashMap;
pub use value::*;

const SIMDJSON_PADDING: usize = mem::size_of::<__m256i>();
// We only do this for the string parse function as it seems to slow down other frunctions
// odd...
lazy_static! {
    static ref MM256_SET1_EPI8_SLASH: __m256i = { unsafe { _mm256_set1_epi8(b'\\' as i8) } };
    static ref MM256_SET1_EPI8_QUOTE: __m256i = { unsafe { _mm256_set1_epi8(b'"' as i8) } };
}

#[cfg(nightly)]
#[macro_export]
macro_rules! likely {
    ($e:expr) => {
        std::intrinsics::likely($e)
    };
}

#[cfg(not(nightly))]
#[macro_export]
macro_rules! likely {
    ($e:expr) => {
        $e
    };
}

#[cfg(nightly)]
#[macro_export]
macro_rules! unlikely {
    ($e:expr) => {
        std::intrinsics::unlikely($e)
    };
}

#[cfg(not(nightly))]
#[macro_export]
macro_rules! unlikely {
    ($e:expr) => {
        $e
    };
}

#[macro_export]
macro_rules! static_cast_i8 {
    ($v:expr) => {
        mem::transmute::<_, i8>($v)
    };
}

#[macro_export]
macro_rules! static_cast_u32 {
    ($v:expr) => {
        mem::transmute::<_, u32>($v)
    };
}

#[macro_export]
macro_rules! static_cast_i64 {
    ($v:expr) => {
        mem::transmute::<_, i64>($v)
    };
}

#[macro_export]
macro_rules! static_cast_u64 {
    ($v:expr) => {
        mem::transmute::<_, u64>($v)
    };
}

#[macro_export]
macro_rules! static_cast_i32 {
    ($v:expr) => {
        mem::transmute::<_, i32>($v)
    };
}

// FROM serde-json
// We only use our own error type; no need for From conversions provided by the
// standard library's try! macro. This reduces lines of LLVM IR by 4%.
#[macro_export]
macro_rules! stry {
    ($e:expr) => {
        match $e {
            ::std::result::Result::Ok(val) => val,
            ::std::result::Result::Err(err) => return ::std::result::Result::Err(err),
        }
    };
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    input: &'de mut [u8],
    strings: Vec<u8>,
    structural_indexes: Vec<u32>,
    idx: usize,
    counts: Vec<usize>,
    str_offset: usize,
    iidx: usize,
}

impl<'de> Deserializer<'de> {
    fn error(&self, error: ErrorType) -> Error {
        Error::new(self.idx, self.iidx, self.c() as char, error)
    }
    // By convention, `Deserializer` constructors are named like `from_xyz`.
    // That way basic use cases are satisfied by something like
    // `serde_json::from_str(...)` while advanced use cases that require a
    // deserializer can make one with `serde_json::Deserializer::from_str(...)`.
    pub fn from_slice(input: &'de mut [u8]) -> Result<Self> {
        // We have to pick an initial size of the structural indexes.
        // 6 is a heuristic that seems to work well for the benchmark
        // data and limit re-allocation frequency.
        let structural_indexes = match unsafe { Deserializer::find_structural_bits(input) } {
            Ok(i) => i,
            Err(t) => {
                return Err(Error::generic(t));
            }
        };
        let (counts, str_len) = Deserializer::compute_size(input, &structural_indexes)?;

        let mut v = Vec::with_capacity(str_len + SIMDJSON_PADDING);
        unsafe {
            v.set_len(str_len + SIMDJSON_PADDING);
        };

        Ok(Deserializer {
            counts,
            structural_indexes,
            input,
            idx: 0,
            strings: v,
            str_offset: 0,
            iidx: 0,
        })
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn compute_size(input: &[u8], structural_indexes: &[u32]) -> Result<(Vec<usize>, usize)> {
        let mut counts = Vec::with_capacity(structural_indexes.len());
        unsafe {
            counts.set_len(structural_indexes.len());
        };
        let mut depth = Vec::with_capacity(structural_indexes.len() / 2); // since we are open close we know worst case this is 2x the size
        let mut arrays = Vec::with_capacity(structural_indexes.len() / 2); // since we are open close we know worst case this is 2x the size
        let mut maps = Vec::with_capacity(structural_indexes.len() / 2); // since we are open close we know worst case this is 2x the size
        let mut last_start = 1;
        let mut cnt = 0;
        let mut str_len = 0;
        for i in 1..structural_indexes.len() {
            let idx = unsafe { *structural_indexes.get_unchecked(i) };
            match unsafe { input.get_unchecked(idx as usize) } {
                b'[' | b'{' => {
                    depth.push((last_start, cnt));
                    last_start = i;
                    cnt = 0;
                }
                b']' => {
                    // if we had any elements we have to add 1 for the last element
                    if i != last_start + 1 {
                        cnt += 1;
                    }
                    let (a_last_start, a_cnt) = stry!(depth
                        .pop()
                        .ok_or_else(|| (Error::generic(ErrorType::Syntax))));
                    counts[last_start] = cnt;
                    last_start = a_last_start;
                    arrays.push(cnt);
                    cnt = a_cnt;
                }
                b'}' => {
                    // if we had any elements we have to add 1 for the last element
                    if i != last_start + 1 {
                        cnt += 1;
                    }
                    let (a_last_start, a_cnt) =
                        stry!(depth.pop().ok_or_else(|| Error::generic(ErrorType::Syntax)));
                    counts[last_start] = cnt;
                    last_start = a_last_start;
                    maps.push(cnt);
                    cnt = a_cnt;
                }
                b',' => cnt += 1,
                b'"' => {
                    let d = if let Some(next) = structural_indexes.get(i + 1) {
                        next - idx
                    } else {
                        // If we're the last element we count to the end
                        input.len() as u32 - idx
                    };
                    if d > str_len {
                        str_len = d;
                    }
                }
                _ => (),
            }
        }
        Ok((counts, str_len as usize))
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn skip(&mut self) {
        self.idx += 1;
        self.iidx = unsafe { *self.structural_indexes.get_unchecked(self.idx) as usize };
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn c(&self) -> u8 {
        unsafe {
            *self
                .input
                .get_unchecked(*self.structural_indexes.get_unchecked(self.idx) as usize)
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn next(&mut self) -> Result<u8> {
        unsafe {
            self.idx += 1;
            if let Some(idx) = self.structural_indexes.get(self.idx) {
                self.iidx = *idx as usize;
                let r = *self.input.get_unchecked(self.iidx);
                Ok(r)
            } else {
                Err(self.error(ErrorType::Syntax))
            }
        }
    }

    // pull out the check so we don't need to
    // stry every time
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn next_(&mut self) -> u8 {
        unsafe {
            self.idx += 1;
            self.iidx = *self.structural_indexes.get_unchecked(self.idx) as usize;
            *self.input.get_unchecked(self.iidx)
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn peek(&self) -> Result<u8> {
        if let Some(idx) = self.structural_indexes.get(self.idx + 1) {
            unsafe { Ok(*self.input.get_unchecked(*idx as usize)) }
        } else {
            Err(self.error(ErrorType::UnexpectedEnd))
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn count_elements(&self) -> usize {
        unsafe { *self.counts.get_unchecked(self.idx) }
    }

    // We parse a string that's likely to be less then 32 characters and without any
    // fancy in it like object keys
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_short_str_(&mut self) -> Result<&'de str> {
        let mut padding = [0u8; 32];
        let idx = self.iidx + 1;
        let src: &[u8] = unsafe { &self.input.get_unchecked(idx..) };

        //short strings are very common for IDs
        let v: __m256i = if src.len() >= 32 {
            // This is safe since we ensure src is at least 32 wide
            unsafe { _mm256_loadu_si256(src[..32].as_ptr() as *const __m256i) }
        } else {
            padding[..src.len()].clone_from_slice(&src);
            // This is safe since we ensure src is at least 32 wide
            unsafe { _mm256_loadu_si256(padding[..32].as_ptr() as *const __m256i) }
        };
        let bs_bits: u32 = unsafe {
            static_cast_u32!(_mm256_movemask_epi8(_mm256_cmpeq_epi8(
                v,
                *MM256_SET1_EPI8_SLASH
            )))
        };
        let quote_mask = unsafe { _mm256_cmpeq_epi8(v, *MM256_SET1_EPI8_QUOTE) };
        let quote_bits = unsafe { static_cast_u32!(_mm256_movemask_epi8(quote_mask)) };
        if (bs_bits.wrapping_sub(1) & quote_bits) != 0 {
            let quote_dist: u32 = trailingzeroes(quote_bits as u64) as u32;
            let v = unsafe {
                self.input.get_unchecked(idx..idx + quote_dist as usize) as *const [u8]
                    as *const str
            };
            self.str_offset = idx + quote_dist as usize;

            unsafe {
                return Ok(&*v);
            }
        }
        self.parse_str_()
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_str_(&mut self) -> Result<&'de str> {
        use std::slice::from_raw_parts_mut;
        // Add 1 to skip the initial "
        let idx = self.iidx + 1;
        let mut padding = [0u8; 32];
        //let mut read: usize = 0;
        let mut written: usize = 0;
        #[cfg(test1)]
        {
            dbg!(idx);
            dbg!(end);
        }

        let needs_relocation = idx - self.str_offset <= 32;
        // we include the terminal '"' so we know where to end
        // This is safe since we check sub's lenght in the range access above and only
        // create sub sliced form sub to `sub.len()`.

        // if we don't need relocation we can write directly to the input
        // saving us to copy data to the string storage first and then
        // back tot he input.
        // We can't always do that as if we're less then 32 characters
        // behind we'll overwrite important parts of the input.
        let mut dst: &mut [u8] = if needs_relocation {
            &mut self.strings
        } else {
            let ptr = self.input.as_mut_ptr();
            unsafe {
                from_raw_parts_mut(
                    ptr.offset(self.str_offset as isize),
                    self.input.len() - self.str_offset,
                )
            }
        };
        let mut src: &[u8] = unsafe { &self.input.get_unchecked(idx..) };

        loop {
            #[cfg(test1)]
            unsafe {
                println!("=== begin loop ===");
                dbg!(written);
                dbg!(String::from_utf8_unchecked(self.input.to_vec()));
                dbg!(String::from_utf8_unchecked(src.to_vec()));
                //                dbg!(String::from_utf8_unchecked(
                //                    &self.strings[self.sidx..written as usize].to_vec()
                //                ));
            }
            let v: __m256i = if src.len() >= 32 {
                // This is safe since we ensure src is at least 32 wide
                unsafe { _mm256_loadu_si256(src[..32].as_ptr() as *const __m256i) }
            } else {
                padding[..src.len()].clone_from_slice(&src);
                // This is safe since we ensure src is at least 32 wide
                unsafe { _mm256_loadu_si256(padding[..32].as_ptr() as *const __m256i) }
            };

            unsafe { _mm256_storeu_si256(dst[..32].as_mut_ptr() as *mut __m256i, v) };

            #[cfg(test1)]
            unsafe {
                dbg!(&src);
                dbg!(&dst);
            }

            // store to dest unconditionally - we can overwrite the bits we don't like
            // later
            let bs_bits: u32 = unsafe {
                static_cast_u32!(_mm256_movemask_epi8(_mm256_cmpeq_epi8(
                    v,
                    *MM256_SET1_EPI8_SLASH
                )))
            };
            let quote_mask = unsafe { _mm256_cmpeq_epi8(v, *MM256_SET1_EPI8_QUOTE) };
            let quote_bits = unsafe { static_cast_u32!(_mm256_movemask_epi8(quote_mask)) };
            if (bs_bits.wrapping_sub(1) & quote_bits) != 0 {
                // we encountered quotes first. Move dst to point to quotes and exit
                // find out where the quote is...
                let quote_dist: u32 = trailingzeroes(quote_bits as u64) as u32;

                ///////////////////////
                // Above, check for overflow in case someone has a crazy string (>=4GB?)
                // But only add the overflow check when the document itself exceeds 4GB
                // Currently unneeded because we refuse to parse docs larger or equal to 4GB.
                ////////////////////////

                // we advance the point, accounting for the fact that we have a NULl termination
                //pj.current_string_buf_loc = dst + quote_dist + 1;

                written += quote_dist as usize;

                unsafe {
                    if needs_relocation {
                        self.input[self.str_offset..self.str_offset + written as usize]
                            .clone_from_slice(&self.strings.get_unchecked(..written));
                    }
                    let v = self
                        .input
                        .get_unchecked(self.str_offset..self.str_offset + written as usize)
                        as *const [u8] as *const str;
                    self.str_offset += written as usize;
                    return Ok(&*v);
                }

                // we compare the pointers since we care if they are 'at the same spot'
                // not if they are the same value
            }
            if (quote_bits.wrapping_sub(1) & bs_bits) != 0 {
                // find out where the backspace is
                let bs_dist: u32 = trailingzeroes(bs_bits as u64);
                #[cfg(test1)]
                unsafe {
                    dbg!(String::from_utf8_unchecked(src.to_vec()));
                    dbg!(bs_dist);
                }
                let escape_char: u8 = src[bs_dist as usize + 1];
                // we encountered backslash first. Handle backslash
                if escape_char == b'u' {
                    // move src/dst up to the start; they will be further adjusted
                    // within the unicode codepoint handling code.
                    src = &src[bs_dist as usize..];
                    //read += bs_dist as usize;
                    dst = &mut dst[bs_dist as usize..];
                    written += bs_dist as usize;
                    let (o, s) = handle_unicode_codepoint(src, dst);
                    if o == 0 {
                        return Err(self.error(ErrorType::InvlaidUnicodeCodepoint));
                    };
                    // We moved o steps forword at the destiation and 6 on the source
                    src = &src[s..];
                    //read += s;
                    dst = &mut dst[o..];
                    written += o;
                } else {
                    // simple 1:1 conversion. Will eat bs_dist+2 characters in input and
                    // write bs_dist+1 characters to output
                    // note this may reach beyond the part of the buffer we've actually
                    // seen. I think this is ok
                    let escape_result: u8 = ESCAPE_MAP[escape_char as usize];
                    if escape_result == 0 {
                        /*
                        #ifdef JSON_TEST_STRINGS // for unit testing
                        foundBadString(buf + offset);
                        #endif // JSON_TEST_STRINGS
                        */
                        return Err(self.error(ErrorType::InvalidEscape));
                    }
                    dst[bs_dist as usize] = escape_result;
                    src = &src[bs_dist as usize + 2..];
                    //read += bs_dist as usize + 2;
                    dst = &mut dst[bs_dist as usize + 1..];
                    written += bs_dist as usize + 1;
                }
            } else {
                // they are the same. Since they can't co-occur, it means we encountered
                // neither.
                src = &src[32..];
                //read += 32;
                dst = &mut dst[32..];
                written += 32;
            }
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_null(&mut self) -> Result<()> {
        let input = unsafe { &self.input.get_unchecked(self.iidx..) };
        let len = input.len();
        if len < SIMDJSON_PADDING {
            let mut copy = vec![0u8; len + SIMDJSON_PADDING];
            copy[0..len].clone_from_slice(input);
            if is_valid_null_atom(&copy) {
                Ok(())
            } else {
                Err(self.error(ErrorType::ExpectedNull))
            }
        } else {
            if is_valid_null_atom(input) {
                Ok(())
            } else {
                Err(self.error(ErrorType::ExpectedNull))
            }
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_null_(&mut self) -> Result<()> {
        let input = unsafe { &self.input.get_unchecked(self.iidx..) };
        if is_valid_null_atom(input) {
            Ok(())
        } else {
            Err(self.error(ErrorType::ExpectedNull))
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_true(&mut self) -> Result<bool> {
        let input = unsafe { &self.input.get_unchecked(self.iidx..) };
        let len = input.len();
        if len < SIMDJSON_PADDING {
            let mut copy = vec![0u8; len + SIMDJSON_PADDING];
            unsafe {
                copy.as_mut_ptr().copy_from(input.as_ptr(), len);
            };
            if is_valid_true_atom(&copy) {
                Ok(true)
            } else {
                Err(self.error(ErrorType::ExpectedBoolean))
            }
        } else {
            if is_valid_true_atom(input) {
                Ok(true)
            } else {
                Err(self.error(ErrorType::ExpectedBoolean))
            }
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_true_(&mut self) -> Result<bool> {
        let input = unsafe { &self.input.get_unchecked(self.iidx..) };
        if is_valid_true_atom(input) {
            Ok(true)
        } else {
            Err(self.error(ErrorType::ExpectedBoolean))
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_false(&mut self) -> Result<bool> {
        let input = unsafe { &self.input.get_unchecked(self.iidx..) };
        let len = input.len();
        if len < SIMDJSON_PADDING {
            let mut copy = vec![0u8; len + SIMDJSON_PADDING];
            unsafe {
                copy.as_mut_ptr().copy_from(input.as_ptr(), len);
            };
            if is_valid_false_atom(&copy) {
                Ok(false)
            } else {
                Err(self.error(ErrorType::ExpectedBoolean))
            }
        } else {
            if is_valid_false_atom(input) {
                Ok(false)
            } else {
                Err(self.error(ErrorType::ExpectedBoolean))
            }
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_false_(&mut self) -> Result<bool> {
        let input = unsafe { &self.input.get_unchecked(self.iidx..) };
        if is_valid_false_atom(input) {
            Ok(false)
        } else {
            Err(self.error(ErrorType::ExpectedBoolean))
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_number(&mut self, minus: bool) -> Result<Number> {
        let input = unsafe { &self.input.get_unchecked(self.iidx..) };
        let len = input.len();
        if len < SIMDJSON_PADDING {
            let mut copy = vec![0u8; len + SIMDJSON_PADDING];
            unsafe {
                copy.as_mut_ptr().copy_from(input.as_ptr(), len);
            };
            self.parse_number_int(&copy, minus)
        } else {
            self.parse_number_int(input, minus)
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_number_(&mut self, minus: bool) -> Result<Number> {
        let input = unsafe { &self.input.get_unchecked(self.iidx..) };
        self.parse_number_int(input, minus)
    }

    fn parse_signed(&mut self) -> Result<i64> {
        match stry!(self.next()) {
            b'-' => match stry!(self.parse_number(true)) {
                Number::I64(n) => Ok(n),
                _ => Err(self.error(ErrorType::ExpectedSigned)),
            },
            b'0'...b'9' => match stry!(self.parse_number(false)) {
                Number::I64(n) => Ok(n),
                _ => Err(self.error(ErrorType::ExpectedSigned)),
            },
            _ => Err(self.error(ErrorType::ExpectedSigned)),
        }
    }
    fn parse_unsigned(&mut self) -> Result<u64> {
        match stry!(self.next()) {
            b'0'...b'9' => match stry!(self.parse_number(false)) {
                Number::I64(n) => Ok(n as u64),
                _ => Err(self.error(ErrorType::ExpectedUnsigned)),
            },
            _ => Err(self.error(ErrorType::ExpectedUnsigned)),
        }
    }

    fn parse_double(&mut self) -> Result<f64> {
        match stry!(self.next()) {
            b'-' => match stry!(self.parse_number(true)) {
                Number::F64(n) => Ok(n),
                Number::I64(n) => Ok(n as f64),
            },
            b'0'...b'9' => match stry!(self.parse_number(false)) {
                Number::F64(n) => Ok(n),
                Number::I64(n) => Ok(n as f64),
            },
            _ => Err(self.error(ErrorType::ExpectedFloat)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::serde::from_slice;
    use super::{owned::to_value, owned::Map, owned::Value, Deserializer};
    use hashbrown::HashMap;
    use proptest::prelude::*;
    use serde::Deserialize;
    use serde_json;

    #[test]
    fn count1() {
        let mut d = String::from("[]");
        let mut d = unsafe { d.as_bytes_mut() };
        let simd = Deserializer::from_slice(&mut d).expect("");
        assert_eq!(simd.counts[1], 0);
    }

    #[test]
    fn count2() {
        let mut d = String::from("[1]");
        let mut d = unsafe { d.as_bytes_mut() };
        let simd = Deserializer::from_slice(&mut d).expect("");
        assert_eq!(simd.counts[1], 1);
    }

    #[test]
    fn count3() {
        let mut d = String::from("[1,2]");
        let mut d = unsafe { d.as_bytes_mut() };
        let simd = Deserializer::from_slice(&mut d).expect("");
        assert_eq!(simd.counts[1], 2);
    }

    #[test]
    fn count4() {
        let mut d = String::from(" [ 1 , [ 3 ] , 2 ]");
        let mut d = unsafe { d.as_bytes_mut() };
        let simd = Deserializer::from_slice(&mut d).expect("");
        assert_eq!(simd.counts[1], 3);
        assert_eq!(simd.counts[4], 1);
    }

    #[test]
    fn bool_true() {
        let mut d = String::from("true");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };

        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::from(true)));
    }

    #[test]
    fn bool_false() {
        let mut d = String::from("false");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::from(false)));
        //assert!(false)
    }

    #[test]
    fn union() {
        let mut d = String::from("null");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::Null));
    }

    #[test]
    fn int() {
        let mut d = String::from("42");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::from(42)));
    }

    #[test]
    fn zero() {
        let mut d = String::from("0");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::from(0)));
    }

    #[test]
    fn one() {
        let mut d = String::from("1");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::from(1)));
    }

    #[test]
    fn minus_one() {
        let mut d = String::from("-1");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::from(-1)));
    }

    #[test]
    fn float() {
        let mut d = String::from("23.0");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::from(23.0)));
    }

    #[test]
    fn string() {
        let mut d = String::from(r#""snot""#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(to_value(&mut d1), Ok(Value::from("snot")));
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn empty_string() {
        let mut d = String::from(r#""""#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(to_value(&mut d1), Ok(Value::from("")));
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn empty_array() {
        let mut d = String::from(r#"[]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("parse_serde");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("parse_simd");
        assert_eq!(to_value(&mut d1), Ok(Value::Array(vec![])));
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn one_element_array() {
        let mut d = String::from(r#"["snot"]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![Value::from("snot")]))
        );
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn two_element_array() {
        let mut d = String::from(r#"["snot", "badger"]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![
                Value::from("snot"),
                Value::from("badger")
            ]))
        );
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn list() {
        let mut d = String::from(r#"[42, 23.0, "snot badger"]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![
                Value::from(42),
                Value::from(23.0),
                Value::from("snot badger")
            ]))
        );
    }

    #[test]
    fn nested_list1() {
        let mut d = String::from(r#"[42, [23.0, "snot"], "bad", "ger"]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![
                Value::from(42),
                Value::Array(vec![Value::from(23.0), Value::from("snot")]),
                Value::from("bad"),
                Value::from("ger")
            ]))
        );

        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn nested_list2() {
        let mut d = String::from(r#"[42, [23.0, "snot"], {"bad": "ger"}]"#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn utf8() {
        let mut d = String::from(r#""\u000e""#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, "\u{e}");
        // NOTE: serde is broken for this
        //assert_eq!(v_serde, "\u{e}");
        //assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn unicode() {
        let mut d = String::from(r#""¡\"""#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn odd_array() {
        let mut d = String::from("[{},null]");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![Value::Object(Map::new()), Value::Null]))
        );
    }

    #[test]
    fn map2() {
        let mut d = String::from(r#"[{"\u0000":null}]"#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn null_null() {
        let mut d = String::from(r#"[null, null]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![Value::Null, Value::Null,]))
        );
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn nested_null() {
        let mut d = String::from(r#"[[null, null]]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![Value::Array(vec![
                Value::Null,
                Value::Null,
            ])]))
        );

        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
    }

    #[test]
    fn nestednested_null() {
        let mut d = String::from(r#"[[[null, null]]]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(
            to_value(&mut d1),
            Ok(Value::Array(vec![Value::Array(vec![Value::Array(vec![
                Value::Null,
                Value::Null,
            ])])]))
        );
    }

    #[test]
    fn odd_array2() {
        let mut d = String::from("[[\"\\u0000\\\"\"]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn odd_array3() {
        let mut d = String::from("[{\"\\u0000\\u0000\":null}]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn odd_array4() {
        let mut d = String::from("[{\"\\u0000𐀀a\":null}]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn float1() {
        let mut d = String::from("2.3250706903316115e307");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("serde_json");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    // We ignore this since serde is less percise on this test
    #[ignore]
    #[test]
    fn float2() {
        let mut d = String::from("-4.5512678569607477e306");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("serde_json");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn map() {
        let mut d = String::from(r#"{"snot": "badger"}"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        let mut h = Map::new();
        h.insert("snot".into(), Value::from("badger"));
        assert_eq!(to_value(&mut d1), Ok(Value::Object(h)));
    }

    #[test]
    fn map1() {
        let mut d = String::from(r#"{"snot": "badger", "badger": "snot"}"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        let mut h = Map::new();
        h.insert("snot".into(), Value::from("badger"));
        h.insert("badger".into(), Value::from("snot"));
        assert_eq!(to_value(&mut d1), Ok(Value::Object(h)));
    }

    #[test]
    fn tpl1() {
        let mut d = String::from("[-65.613616999999977, 43.420273000000009]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: (f32, f32) = serde_json::from_slice(d).expect("serde_json");
        let v_simd: (f32, f32) = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn tpl2() {
        let mut d = String::from("[[-65.613616999999977, 43.420273000000009]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<(f32, f32)> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<(f32, f32)> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn tpl3() {
        let mut d = String::from(
            "[[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]]",
        );
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<(f32, f32)> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<(f32, f32)> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }
    #[test]
    fn tpl4() {
        let mut d = String::from("[[[-65.613616999999977,43.420273000000009]]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<Vec<(f32, f32)>> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<Vec<(f32, f32)>> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }
    #[test]
    fn tpl5() {
        let mut d = String::from("[[[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<Vec<(f32, f32)>> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<Vec<(f32, f32)>> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn tpl6() {
        let mut d = String::from("[[[[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]]]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<Vec<Vec<(f32, f32)>>> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<Vec<Vec<(f32, f32)>>> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn tpl7() {
        let mut d = String::from("[[[[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]]]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<Vec<Vec<[f32; 2]>>> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<Vec<Vec<[f32; 2]>>> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct Obj {
        a: u64,
        b: u64,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct Obj1 {
        a: Obj,
    }

    #[test]
    fn obj() {
        let mut d = String::from(r#"{"a": 1, "b":1}"#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Obj = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Obj = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn obj2() {
        let mut d =
            String::from(r#"{"a": {"a": 1, "b":1}, "b": {"a": 1, "b":1}, "c": {"a": 1, "b": 1}}"#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: HashMap<String, Obj> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: HashMap<String, Obj> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn obj3() {
        let mut d = String::from(
            r#"{"c": {"a": {"a": 1, "b":1}, "b": {"a": 1, "b":1}, "c": {"a": 1, "b": 1}}}"#,
        );
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: HashMap<String, HashMap<String, Obj>> =
            serde_json::from_slice(d).expect("serde_json");
        let v_simd: HashMap<String, HashMap<String, Obj>> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn obj4() {
        let mut d = String::from(r#"{"c": {"a": {"a": 1, "b":1}}}"#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: HashMap<String, Obj1> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: HashMap<String, Obj1> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn vecvec() {
        let mut d = String::from("[[[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]], [[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<Vec<(f32, f32)>> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<Vec<(f32, f32)>> = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn crazy_string() {
        // there is unicode in here!
        let d = "\"𐀀𐀀  𐀀𐀀0 𐀀A\\u00000A0 A \\u000b\"";
        let mut d = String::from(d);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("serde_json");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn event() {
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(deny_unknown_fields, rename_all = "camelCase")]
        pub struct CitmCatalog {
            pub area_names: HashMap<String, String>,
            pub audience_sub_category_names: HashMap<String, String>,
            pub block_names: HashMap<String, String>,
            pub events: HashMap<String, Event>,
        }
        pub type Id = u32;
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(deny_unknown_fields, rename_all = "camelCase")]
        pub struct Event {
            pub description: (),
            pub id: Id,
            pub logo: Option<String>,
            pub name: String,
            pub sub_topic_ids: Vec<Id>,
            pub subject_code: (),
            pub subtitle: (),
            pub topic_ids: Vec<Id>,
        }

        let mut d = String::from(
            r#"
{
    "areaNames": {
        "205705993": "Arrière-scène central",
        "205705994": "1er balcon central",
        "205705995": "2ème balcon bergerie cour",
        "205705996": "2ème balcon bergerie jardin",
        "205705998": "1er balcon bergerie jardin",
        "205705999": "1er balcon bergerie cour",
        "205706000": "Arrière-scène jardin",
        "205706001": "Arrière-scène cour",
        "205706002": "2ème balcon jardin",
        "205706003": "2ème balcon cour",
        "205706004": "2ème Balcon central",
        "205706005": "1er balcon jardin",
        "205706006": "1er balcon cour",
        "205706007": "Orchestre central",
        "205706008": "Orchestre jardin",
        "205706009": "Orchestre cour",
        "342752287": "Zone physique secrète"
    },
    "audienceSubCategoryNames": {
        "337100890": "Abonné"
    },
    "blockNames": {},
  "events": {
    "138586341": {
      "description": null,
      "id": 138586341,
      "logo": null,
      "name": "30th Anniversary Tour",
      "subTopicIds": [
        337184269,
        337184283
      ],
      "subjectCode": null,
      "subtitle": null,
      "topicIds": [
        324846099,
        107888604
      ]
    },
    "138586345": {
      "description": null,
      "id": 138586345,
      "logo": "/images/UE0AAAAACEKo6QAAAAZDSVRN",
      "name": "Berliner Philharmoniker",
      "subTopicIds": [
        337184268,
        337184283,
        337184275
      ],
      "subjectCode": null,
      "subtitle": null,
      "topicIds": [
        324846099,
        107888604,
        324846100
      ]
    }
  }
}
"#,
        );
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: CitmCatalog = serde_json::from_slice(d).expect("serde_json");
        let v_simd: CitmCatalog = from_slice(&mut d).expect("simd_json");
        assert_eq!(v_simd, v_serde)
    }

    fn arb_json() -> BoxedStrategy<String> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::Bool),
            //(-1.0e306f64..1.0e306f64).prop_map(|f| json!(f)), The float parsing of simd and serde are too different
            any::<i64>().prop_map(|i| json!(i)),
            ".*".prop_map(Value::from),
        ];
        leaf.prop_recursive(
            8,   // 8 levels deep
            256, // Shoot for maximum size of 256 nodes
            10,  // We put up to 10 items per collection
            |inner| {
                prop_oneof![
                    // Take the inner strategy and make the two recursive cases.
                    prop::collection::vec(inner.clone(), 0..10).prop_map(|v| json!(v)),
                    prop::collection::hash_map(".*", inner, 0..10).prop_map(|m| json!(m)),
                ]
            },
        )
        .prop_map(|v| serde_json::to_string(&v).expect("").to_string())
        .boxed()
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            // Setting both fork and timeout is redundant since timeout implies
            // fork, but both are shown for clarity.
            fork: true,
            .. ProptestConfig::default()
        })]
        #[test]
        fn json_test(d in arb_json()) {
            if let Ok(v_serde) = serde_json::from_slice::<serde_json::Value>(&d.as_bytes()) {
                let mut d = d.clone();
                let d = unsafe{ d.as_bytes_mut()};
                let v_simd: serde_json::Value = from_slice(d).expect("");
                assert_eq!(v_simd, v_serde)
            }

        }
    }

}
