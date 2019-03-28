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
use crate::stage2::*;
use serde::forward_to_deserialize_any;
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::mem;

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

const SIMDJSON_PADDING: usize = mem::size_of::<__m256i>();

pub use crate::parsedjson::ParsedJson;
use crate::stage1::find_structural_bits;

use serde::Deserialize;
//use serde::de::Deserializer as DeserializerT;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    ExpectedBoolean,
    TrailingCharacters,
    ExpectedArrayComma,
    ExpectedInteger,
    Syntax,
    UnexpectedCharacter(char, usize),
    ExpectedMapColon,
    ExpectedMapComma,
    UnexpectedEnd,
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
    input: &'de [u8],
    pj: ParsedJson<'de>,
    idx: usize,
}

impl<'de> Deserializer<'de> {
    // By convention, `Deserializer` constructors are named like `from_xyz`.
    // That way basic use cases are satisfied by something like
    // `serde_json::from_str(...)` while advanced use cases that require a
    // deserializer can make one with `serde_json::Deserializer::from_str(...)`.
    pub fn from_slice(input: &'de [u8]) -> Self {
        let mut pj = ParsedJson::from_slice(input);
        unsafe {
            find_structural_bits(input, input.len() as u32, &mut pj);
            //unified_machine(input, input.len(), &mut pj);
        };
        Deserializer { input, pj, idx: 0 }
    }
    fn next_char(&mut self) -> Result<u8, Error> {
        if let Some(idx) = self.pj.structural_indexes.get(self.idx) {
            let r = self.input[*idx as usize];
            self.idx += 1;
            Ok(r)
        } else {
            Err(Error::UnexpectedEnd)
        }
    }
    fn next(&mut self) -> Result<(u8, usize), Error> {
        if let Some(idx) = self.pj.structural_indexes.get(self.idx) {
            let idx = *idx as usize;
            let r = self.input[idx];
            self.idx += 1;
            Ok((r, idx))
        } else {
            Err(Error::UnexpectedEnd)
        }
    }

    fn peek(&self) -> Result<u8, Error> {
        if let Some(idx) = self.pj.structural_indexes.get(self.idx) {
            let idx = *idx as usize;
            let r = self.input[idx];
            Ok(r)
        } else {
            Err(Error::UnexpectedEnd)
        }
    }
    fn cur(&self) -> Option<&(usize, ItemType)> {
        let r = self.pj.tape.get(self.idx);
        r
    }
}

pub fn from_slice<'a, T>(s: &'a [u8]) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_slice(s);
    T::deserialize(&mut deserializer)
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
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
                    match parse_number(&copy, true) {
                        Ok(Number::F64(n)) => visitor.visit_f64(n),
                        Ok(Number::I64(n)) => visitor.visit_i64(n),
                        _ => Err(Error::ExpectedInteger),
                    }
                } else {
                    match parse_number(input, true) {
                        Ok(Number::F64(n)) => visitor.visit_f64(n),
                        Ok(Number::I64(n)) => visitor.visit_i64(n),
                        _ => Err(Error::ExpectedInteger),
                    }
                }
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

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        // Check if there are no more elements.
        if self.de.peek()? == b']' {
            self.de.next()?;
            return Ok(None);
        }
        // Comma is required before every element except the first.
        if !self.first && self.de.next_char()? != b',' {
            return Err(Error::ExpectedArrayComma);
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

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
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

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: DeserializeSeed<'de>,
    {
        // It doesn't make a difference whether the colon is parsed at the end
        // of `next_key_seed` or at the beginning of `next_value_seed`. In this
        // case the code is a bit simpler having it here.
        if self.de.next_char()? != b':' {
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
        let d = b"true";
        let v_serde: serde_json::Value = serde_json::from_slice(d).unwrap();
        let v_simd: serde_json::Value = from_slice(d).unwrap();
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn bool_false() {
        let d = b"false";
        let v_serde: serde_json::Value = serde_json::from_slice(d).unwrap();
        let v_simd: serde_json::Value = from_slice(d).unwrap();
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn union() {
        let d = b"null";
        let v_serde: serde_json::Value = serde_json::from_slice(d).unwrap();
        let v_simd: serde_json::Value = from_slice(d).unwrap();
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn int() {
        let d = b"42";
        let v_serde: serde_json::Value = serde_json::from_slice(d).unwrap();
        let v_simd: serde_json::Value = from_slice(d).unwrap();
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn zero() {
        let d = b"0";
        let v_serde: serde_json::Value = serde_json::from_slice(d).unwrap();
        let v_simd: serde_json::Value = from_slice(d).unwrap();
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn one() {
        let d = b"1";
        let v_serde: serde_json::Value = serde_json::from_slice(d).unwrap();
        let v_simd: serde_json::Value = from_slice(d).unwrap();
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn minus_one() {
        let d = b"-1";
        let v_serde: serde_json::Value = serde_json::from_slice(d).unwrap();
        let v_simd: serde_json::Value = from_slice(d).unwrap();
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn float() {
        let d = b"23.0";
        let v_serde: serde_json::Value = serde_json::from_slice(d).unwrap();
        let v_simd: serde_json::Value = from_slice(d).unwrap();
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn list() {
        let d = br#"[42, 23.0, "snot badger"]"#;
        let v_serde: serde_json::Value = serde_json::from_slice(d).unwrap();
        let v_simd: serde_json::Value = from_slice(d).unwrap();
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn nested_list() {
        let d = br#"[42, [23.0, "snot"], {"bad": "ger"}]"#;
        let v_serde: serde_json::Value = serde_json::from_slice(d).unwrap();
        let v_simd: serde_json::Value = from_slice(d).unwrap();
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn utf8() {
        let d = b"\"\\u000e\"";
        let v_serde: serde_json::Value = serde_json::from_slice(d).unwrap();
        let v_simd: serde_json::Value = from_slice(d).unwrap();
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn odd_array() {
        let d = b"[{},null]";
        let v_serde: serde_json::Value = serde_json::from_slice(d).unwrap();
        let v_simd: serde_json::Value = from_slice(d).unwrap();
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn crazy() {
        let d = &[
            91, 102, 97, 108, 115, 101, 44, 123, 34, 92, 117, 48, 48, 48, 101, 92, 98, 48, 97, 92,
            117, 48, 48, 48, 101, 240, 144, 128, 128, 48, 65, 35, 32, 92, 117, 48, 48, 48, 48, 97,
            240, 144, 128, 128, 240, 144, 128, 128, 240, 144, 128, 128, 240, 144, 128, 128, 65, 32,
            240, 144, 128, 128, 92, 92, 34, 58, 110, 117, 108, 108, 125, 93,
        ];
        let v_serde: serde_json::Value = serde_json::from_slice(d).unwrap();
        let v_simd: serde_json::Value = from_slice(d).unwrap();
        assert_eq!(v_simd, v_serde)
    }
    fn arb_json() -> BoxedStrategy<Value> {
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
        .boxed()
    }

    proptest! {
    //        #[test]
            fn json_test(j in arb_json()) {
                let d = serde_json::to_vec(&j).unwrap();
                if let Ok(v_serde) = serde_json::from_slice::<serde_json::Value>(&d) {
                    dbg!(&d);
                    dbg!(&String::from_utf8(d.clone()).unwrap());
                    let v_simd: serde_json::Value = from_slice(&d).unwrap();
                    assert_eq!(v_simd, v_serde)
                }

            }
        }

}
