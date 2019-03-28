#![feature(ptr_offset_from)]

mod charutils;
mod numberparse;
mod parsedjson;
mod portability;
mod stage1;
mod stage2;
mod stringparse;

use crate::numberparse::Number;
use crate::numberparse::*;
use crate::parsedjson::*;
use crate::portability::*;
use crate::stage2::*;
use crate::stringparse::*;
use serde::forward_to_deserialize_any;
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::mem;
use std::str;

//TODO: Do compile hints like this exist in rust?
/*
macro_rules! likely {
    ($e:expr) => {
        $e
    };
}
*/
#[macro_export]
macro_rules! unlikely {
    ($e:expr) => {
        $e
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

pub type Result<T> = std::result::Result<T, Error>;

const SIMDJSON_PADDING: usize = mem::size_of::<__m256i>();

pub use crate::parsedjson::ParsedJson;
use crate::stage1::find_structural_bits;

use serde::Deserialize;
//use serde::de::Deserializer as DeserializerT;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    InvlaidUnicodeCodepoint,
    InvalidUnicodeEscape,
    InvalidEscape(char),
    ExpectedBoolean,
    TrailingCharacters,
    ExpectedArrayComma(char),
    ExpectedInteger,
    Syntax,
    UnexpectedCharacter(char, usize),
    ExpectedMapColon,
    ExpectedMapComma,
    UnexpectedEnd,
    UnterminatedString,
    Parser,
    EarlyEnd,
    ExpectedNull,
    ExpectedMapEnd,
    ExpectedArray,
    ExpectedMap,
    ExpectedEnum,
    BadKeyType,
    Serde(String),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Serde(msg.to_string())
    }
}

pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    input: &'de mut [u8],
    pj: ParsedJson,
    idx: usize,
}

impl<'de> Deserializer<'de> {
    // By convention, `Deserializer` constructors are named like `from_xyz`.
    // That way basic use cases are satisfied by something like
    // `serde_json::from_str(...)` while advanced use cases that require a
    // deserializer can make one with `serde_json::Deserializer::from_str(...)`.
    pub fn from_slice(input: &'de mut [u8]) -> Self {
        let mut pj = ParsedJson::from_slice();
        unsafe {
            find_structural_bits(input, input.len() as u32, &mut pj);
            //unified_machine(input, input.len(), &mut pj);
        };
        Deserializer { input, pj, idx: 0 }
    }
    fn next_char(&mut self) -> Result<u8> {
        if let Some(idx) = self.pj.structural_indexes.get(self.idx) {
            let r = self.input[*idx as usize];
            self.idx += 1;
            Ok(r)
        } else {
            Err(Error::UnexpectedEnd)
        }
    }
    fn next(&mut self) -> Result<(u8, usize)> {
        if let Some(idx) = self.pj.structural_indexes.get(self.idx) {
            let idx = *idx as usize;
            let r = self.input[idx];
            self.idx += 1;
            Ok((r, idx))
        } else {
            Err(Error::UnexpectedEnd)
        }
    }

    fn peek(&self) -> Result<u8> {
        if let Some(idx) = self.pj.structural_indexes.get(self.idx) {
            let idx = *idx as usize;
            let r = self.input[idx];
            Ok(r)
        } else {
            Err(Error::UnexpectedEnd)
        }
    }
    fn peek_idx(&self) -> Result<usize> {
        if let Some(idx) = self.pj.structural_indexes.get(self.idx) {
            Ok(*idx as usize)
        } else {
            Err(Error::UnexpectedEnd)
        }
    }

    #[inline(always)]
    pub fn parse_string(&mut self, idx: usize) -> Result<String> {
        use std::slice::{from_raw_parts, from_raw_parts_mut};
        unsafe {
            use std::num::Wrapping;
            let mut padding = [0u8; 32];
            let mut read: u32 = 0;
            let mut written: u32 = 0;
            let end = self.peek_idx()?;
            dbg!(idx);
            dbg!(end);
            let sub = if end == 0 {
                &mut self.input[idx..]
            } else {
                &mut self.input[idx..end]
            };
            let mut dst: &mut [u8] = from_raw_parts_mut(sub.as_mut_ptr(), sub.len());
            let mut src: &[u8] = from_raw_parts(sub.as_mut_ptr(), sub.len());
            let res = from_raw_parts(sub.as_mut_ptr(), sub.len());
            loop {
                dbg!(written);
                dbg!(String::from_utf8_unchecked(self.input.to_vec()));
                dbg!(String::from_utf8_unchecked(src.to_vec()));
                dbg!(String::from_utf8_unchecked(
                    res[..written as usize].to_vec()
                ));
                let v: __m256i = if src.len() >= 32 {
                    _mm256_loadu_si256(src[..32].as_ptr() as *const __m256i)
                } else {
                    padding[..src.len()].clone_from_slice(&src);
                    _mm256_loadu_si256(padding[..32].as_ptr() as *const __m256i)
                };

                // store to dest unconditionally - we can overwrite the bits we don't like
                // later
                let bs_bits: u32 = static_cast_u32!(_mm256_movemask_epi8(_mm256_cmpeq_epi8(
                    v,
                    _mm256_set1_epi8(b'\\' as i8)
                )));
                let quote_mask = _mm256_cmpeq_epi8(v, _mm256_set1_epi8(b'"' as i8));
                let quote_bits = static_cast_u32!(_mm256_movemask_epi8(quote_mask));
                if ((Wrapping(bs_bits) - Wrapping(1)).0 & quote_bits) != 0 {
                    dbg!();
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

                    if read != written {
                        unsafe {
                            dbg!(String::from_utf8_unchecked(res.to_vec()));
                            dbg!(String::from_utf8_unchecked(dst.to_vec()));
                            dbg!(read);
                            dbg!(written);
                            dbg!(quote_dist);
                        }
                        dst[..quote_dist as usize].clone_from_slice(&src[..quote_dist as usize]);
                        unsafe {
                            dbg!(String::from_utf8_unchecked(dst.to_vec()));
                        }
                    }
                    written += quote_dist;
                    let s = String::from_utf8_lossy(&res[..written as usize]).to_string();
                    return Ok(s);
                /*
                return Ok(str::from_utf8_unchecked(s));
                     */
                // we compare the pointers since we care if they are 'at the same spot'
                // not if they are the same value
                } else if read != written {
                    if src.len() >= 32 {
                        dbg!();
                        _mm256_storeu_si256(dst.as_mut_ptr() as *mut __m256i, v);
                    } else {
                        dbg!();
                        dst[..src.len()].clone_from_slice(src);
                    }
                }
                if ((Wrapping(quote_bits) - Wrapping(1)).0 & bs_bits) != 0 {
                    dbg!();
                    // find out where the backspace is
                    let bs_dist: u32 = trailingzeroes(bs_bits as u64);
                    dbg!(String::from_utf8_unchecked(src.to_vec()));
                    dbg!(bs_dist);
                    let escape_char: u8 = src[bs_dist as usize + 1];
                    // we encountered backslash first. Handle backslash
                    if escape_char == b'u' {
                        // move src/dst up to the start; they will be further adjusted
                        // within the unicode codepoint handling code.
                        src = &src[bs_dist as usize..];
                        read += bs_dist;
                        dst = &mut dst[bs_dist as usize..];
                        written += bs_dist;
                        let o = handle_unicode_codepoint(&mut src, &mut dst);
                        if o == 0 {
                            return Err(Error::InvlaidUnicodeCodepoint);
                        };
                        // We moved o steps forword at the destiation and 6 on the source
                        src = &src[6..];
                        read += 6;
                        dst = &mut dst[o..];
                        written += o as u32;
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
                            return Err(Error::InvalidEscape(escape_char as char));
                        }
                        dst[bs_dist as usize] = escape_result;
                        src = &src[bs_dist as usize + 2..];
                        read += bs_dist + 2;
                        dst = &mut dst[bs_dist as usize + 1..];
                        written += bs_dist + 1;
                    }
                } else {
                    // they are the same. Since they can't co-occur, it means we encountered
                    // neither.
                    src = &src[32..];
                    read += 32;
                    dst = &mut dst[32..];
                    written += 32;
                }
            }
        }
    }
}

pub fn from_slice<'a, T>(s: &'a mut [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_slice(s);

    T::deserialize(&mut deserializer)
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.next()? {
            (b'n', idx) => {
                let input = &self.input[idx..];
                let len = input.len();
                if len < SIMDJSON_PADDING {
                    let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                    unsafe {
                        copy.as_mut_ptr().copy_from(input.as_ptr(), len);
                    };
                    if is_valid_null_atom(&copy) {
                        visitor.visit_unit()
                    } else {
                        Err(Error::ExpectedNull)
                    }
                } else {
                    if is_valid_null_atom(input) {
                        visitor.visit_unit()
                    } else {
                        Err(Error::ExpectedNull)
                    }
                }
            }
            (b't', idx) => {
                let input = &self.input[idx..];
                let len = input.len();
                if len < SIMDJSON_PADDING {
                    let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                    unsafe {
                        copy.as_mut_ptr().copy_from(input.as_ptr(), len);
                    };
                    if is_valid_true_atom(&copy) {
                        visitor.visit_bool(true)
                    } else {
                        Err(Error::ExpectedBoolean)
                    }
                } else {
                    if is_valid_true_atom(input) {
                        visitor.visit_bool(true)
                    } else {
                        Err(Error::ExpectedBoolean)
                    }
                }
            }
            (b'f', idx) => {
                let input = &self.input[idx..];
                let len = input.len();
                if len < SIMDJSON_PADDING {
                    let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                    unsafe {
                        copy.as_mut_ptr().copy_from(input.as_ptr(), len);
                    };
                    if is_valid_false_atom(&copy) {
                        visitor.visit_bool(false)
                    } else {
                        Err(Error::ExpectedBoolean)
                    }
                } else {
                    if is_valid_false_atom(input) {
                        visitor.visit_bool(false)
                    } else {
                        Err(Error::ExpectedBoolean)
                    }
                }
            }
            (b'0'...b'9', idx) => {
                let input = &self.input[idx..];
                let len = input.len();
                if len < SIMDJSON_PADDING {
                    let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                    unsafe {
                        copy.as_mut_ptr().copy_from(input.as_ptr(), len);
                    };
                    match parse_number(&copy, false) {
                        Ok(Number::F64(n)) => visitor.visit_f64(n),
                        Ok(Number::I64(n)) => visitor.visit_i64(n),
                        _ => Err(Error::ExpectedInteger),
                    }
                } else {
                    match parse_number(input, false) {
                        Ok(Number::F64(n)) => visitor.visit_f64(n),
                        Ok(Number::I64(n)) => visitor.visit_i64(n),
                        _ => Err(Error::ExpectedInteger),
                    }
                }
            }
            (b'-', idx) => {
                let input = &self.input[idx..];
                let len = input.len();
                if len < SIMDJSON_PADDING {
                    let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                    unsafe {
                        copy.as_mut_ptr().copy_from(input.as_ptr(), len);
                    };
                    match parse_number(&copy, true)? {
                        Number::F64(n) => visitor.visit_f64(n),
                        Number::I64(n) => visitor.visit_i64(n),
                    }
                } else {
                    match parse_number(input, true)? {
                        Number::F64(n) => visitor.visit_f64(n),
                        Number::I64(n) => visitor.visit_i64(n),
                    }
                }
            }
            (b'"', idx) => {
                unsafe {
                    dbg!(String::from_utf8_unchecked(self.input.to_vec()));
                }
                let r = visitor.visit_string(self.parse_string(idx + 1)?);
                unsafe {
                    dbg!(String::from_utf8_unchecked(self.input.to_vec()));
                }
                r
            }
            (b'[', _idx) => visitor.visit_seq(CommaSeparated::new(&mut self)),
            (b'{', _idx) => visitor.visit_map(CommaSeparated::new(&mut self)),
            (c, idx) => Err(Error::UnexpectedCharacter(c as char, idx)),
        }
    }
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
    }
}

// In order to handle commas correctly when deserializing a JSON array or map,
// we need to track whether we are on the first element or past the first
// element.
struct CommaSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        CommaSeparated { de, first: true }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        // Check if there are no more elements.
        if self.de.peek()? == b']' {
            self.de.next()?;
            return Ok(None);
        }
        // Comma is required before every element except the first.
        if !self.first {
            let c = self.de.next_char()?;
            if c != b',' {
                return Err(Error::ExpectedArrayComma(self.de.next_char()? as char));
            }
        }
        self.first = false;
        // Deserialize an array element.
        seed.deserialize(&mut *self.de).map(Some)
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a> MapAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // Check if there are no more entries.
        if self.de.peek()? == b'}' {
            self.de.next()?;
            return Ok(None);
        }
        // Comma is required before every entry except the first.
        if !self.first && self.de.next_char()? != b',' {
            return Err(Error::ExpectedMapComma);
        }
        self.first = false;
        // Deserialize a map key.
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // It doesn't make a difference whether the colon is parsed at the end
        // of `next_key_seed` or at the beginning of `next_value_seed`. In this
        // case the code is a bit simpler having it here.
        let c = self.de.next_char()?;
        if c != b':' {
            return Err(Error::ExpectedMapColon);
        }
        // Deserialize a map value.
        seed.deserialize(&mut *self.de)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::{self, json, Value};

    #[test]
    fn bool_true() {
        let mut d = String::from("true");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn bool_false() {
        let mut d = String::from("false");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn union() {
        let mut d = String::from("null");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn int() {
        let mut d = String::from("42");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn zero() {
        let mut d = String::from("0");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn one() {
        let mut d = String::from("1");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn minus_one() {
        let mut d = String::from("-1");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn float() {
        let mut d = String::from("23.0");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn string() {
        let mut d = String::from(r#""snot""#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn list() {
        let mut d = String::from(r#"[42, 23.0, "snot badger"]"#);
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn nested_list() {
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
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
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
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn odd_array2() {
        let mut d = String::from("[[\"\\u0000\\\"\"]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    fn arb_json() -> BoxedStrategy<String> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::Bool),
            (-1.0e308f64..1.0e308f64).prop_map(|f| json!(f)),
            any::<i64>().prop_map(|i| json!(i)),
            ".*".prop_map(Value::String),
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
                let mut d = unsafe{ d.as_bytes_mut()};
                let v_simd: serde_json::Value = from_slice(d).expect("");
                assert_eq!(v_simd, v_serde)
            }

        }
    }

}
