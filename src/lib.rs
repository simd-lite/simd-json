//#![feature(stdsimd)]
#![feature(reverse_bits)]
#![feature(ptr_offset_from)]

mod charutils;
mod numberparse;
mod parsedjson;
mod portability;
mod stage1;
mod stage2;
mod stringparse;

use parsedjson::*;

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

pub use crate::parsedjson::ParsedJson;
use crate::stage1::find_structural_bits;
use crate::stage2::unified_machine;
pub use crate::stage2::MachineError;

pub fn parse(data: &[u8]) -> Result<ParsedJson, MachineError> {
    let mut pj = ParsedJson::from_slice(data);
    unsafe {
        find_structural_bits(data, data.len() as u32, &mut pj);
        unified_machine(data, data.len(), &mut pj)?;
    };
    Ok(pj)
}


use serde::Deserialize;
//use serde::de::Deserializer as DeserializerT;
use serde::de::{
    self,  IntoDeserializer, Visitor,
    SeqAccess, DeserializeSeed, MapAccess
};
use std::ops::{AddAssign, MulAssign, Neg};
use std::fmt;

#[derive(Debug)]
pub enum DeserializerError {
    ExpectedBoolean,
    TrailingCharacters,
    ExpectedInteger,
    Syntax,
    EarlyEnd,
    ExpectedNull,
    ExpectedMapEnd,
    ExpectedArray,
    ExpectedMap,
    ExpectedEnum,
    BadKeyType,
    Serde(String)
}
impl fmt::Display for DeserializerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for MachineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for DeserializerError {}
impl std::error::Error for MachineError {}

impl serde::de::Error for DeserializerError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        DeserializerError::Serde(msg.to_string())
    }
}

impl serde::de::Error for MachineError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        MachineError::Serde(msg.to_string())
    }
}

pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    input: &'de [u8],
    pj: ParsedJson<'de>,
    idx: usize,
    int_idx: usize,
    double_idx: usize,
    string_idx: usize,
    depth: usize


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
            unified_machine(input, input.len(), &mut pj);
        };
        Deserializer { input, pj, idx: 0, int_idx: 0, double_idx: 0, string_idx:0, depth: 0 }
    }
    fn cur(&self) -> Option<&(usize, ItemType)> {
        let r = self.pj.tape.get(self.idx);
        r
    }
}

pub fn from_slice<'a, T>(s: &'a [u8]) -> Result<T, DeserializerError>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_slice(s);
    T::deserialize(&mut deserializer)
}


// SERDE IS NOT A PARSING LIBRARY. This impl block defines a few basic parsing
// functions from scratch. More complicated formats may wish to use a dedicated
// parsing library to help implement their Serde deserializer.
impl<'de> Deserializer<'de> {
    // // Look at the first character in the input without consuming it.
    // fn peek_char(&mut self) -> Result<char> {
    //     self.input.chars().next().ok_or(DeserializerError::Eof)
    // }

    // // Consume the first character in the input.
    // fn next_char(&mut self) -> Result<char> {
    //     let ch = self.peek_char()?;
    //     self.input = &self.input[ch.len_utf8()..];
    //     Ok(ch)
    // }


    // Parse a group of decimal digits as an unsigned integer of type T.
    //
    // This implementation is a bit too lenient, for example `001` is not
    // allowed in JSON. Also the various arithmetic operations can overflow and
    // panic or return bogus data. But it is good enough for example code!
    fn parse_unsigned<T>(&mut self) -> Result<T, DeserializerError>
    where
        T: AddAssign<T> + MulAssign<T> + From<u64>,
    {
        match self.pj.tape.get(self.idx) {
            Some((_, ItemType::I64)) => {
                self.idx += 1;
                let r = self.pj.ints[self.int_idx] as u64;
                self.int_idx += 1;
                Ok(r.into())
            }
            _ => Err(DeserializerError::ExpectedInteger)
        }
    }

    // Parse a possible minus sign followed by a group of decimal digits as a
    // signed integer of type T.
    fn parse_signed<T>(&mut self) -> Result<T, DeserializerError>
    where
        T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + From<i64>,
    {
        match self.pj.tape.get(self.idx) {
            Some((_, ItemType::I64)) => {
                self.idx += 1;
                let r = self.pj.ints[self.int_idx];
                self.int_idx += 1;
                Ok(r.into())
            }
            _ => Err(DeserializerError::ExpectedInteger)
        }
    }

    // Parse a possible minus sign followed by a group of decimal digits as a
    // signed integer of type T.
    fn parse_float<T>(&mut self) -> Result<T, DeserializerError>
    where
        T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + From<f64>,
    {
        match self.pj.tape.get(self.idx) {
            Some((_, ItemType::Double)) => {
                self.idx += 1;
                let r = self.pj.doubles[self.double_idx];
                self.double_idx += 1;
                Ok(r.into())
            }
            _ => Err(DeserializerError::ExpectedInteger)
        }
    }

    // Parse a string until the next '"' character.
    //
    // Makes no attempt to handle escape sequences. What did you expect? This is
    // example code!
    fn parse_string(&mut self) -> Result<String, DeserializerError> {
        match self.pj.tape.get(self.idx) {
            Some((_, ItemType::String)) => {
                self.idx += 1;
                let r = Ok(self.pj.strings[self.string_idx].clone()) ;
                self.string_idx += 1;
                r
            }
            _ => Err(DeserializerError::ExpectedInteger)
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = DeserializerError;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        match self.cur() {
            Some((_, ItemType::Root)) if self.idx == 0 => {self.idx += 1; self.deserialize_any(visitor)}
            Some((_, ItemType::Null))  => self.deserialize_unit(visitor),
            Some((_, ItemType::True)) | Some((_, ItemType::False)) => self.deserialize_bool(visitor),
            Some((_, ItemType::String))  => self.deserialize_str(visitor),
            Some((_, ItemType::I64))  => self.deserialize_i64(visitor),
            Some((_, ItemType::Double))  => self.deserialize_f64(visitor),
            Some((_, ItemType::Array))  => self.deserialize_seq(visitor),
            Some((_, ItemType::Object))  => self.deserialize_map(visitor),
            Some((_, ItemType::Root)) | Some((_, ItemType::ArrayEnd)) |  Some((_, ItemType::ObjectEnd)) => Err(DeserializerError::Syntax),
            None => Err(DeserializerError::Syntax),
        }
    }

    // Uses the `parse_bool` parsing function defined above to read the JSON
    // identifier `true` or `false` from the input.
    //
    // Parsing refers to looking at the input and deciding that it contains the
    // JSON value `true` or `false`.
    //
    // Deserialization refers to mapping that JSON value into Serde's data
    // model by invoking one of the `Visitor` methods. In the case of JSON and
    // bool that mapping is straightforward so the distinction may seem silly,
    // but in other cases Deserializers sometimes perform non-obvious mappings.
    // For example the TOML format has a Datetime type and Serde's data model
    // does not. In the `toml` crate, a Datetime in the input is deserialized by
    // mapping it to a Serde data model "struct" type with a special name and a
    // single field containing the Datetime represented as a string.
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        match self.pj.tape.get(self.idx) {
            Some((_, ItemType::True)) => {
                self.idx += 1;
                visitor.visit_bool(true)
            }
            Some((_, ItemType::False)) => {
                self.idx += 1;
                visitor.visit_bool(false)
            }
            _ => Err(DeserializerError::ExpectedBoolean)
        }
    }

    // The `parse_signed` function is generic over the integer type `T` so here
    // it is invoked with `T=i8`. The next 8 methods are similar.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        let v: i64 = self.parse_signed()?;
        visitor.visit_i8(v as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        let v: i64 = self.parse_signed()?;
        visitor.visit_i16(v as i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        let v: i64 = self.parse_signed()?;
        visitor.visit_i32(v as i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_signed()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        let v: u64 = self.parse_unsigned()?;
        visitor.visit_u8(v as u8)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        let v: u64 = self.parse_unsigned()?;
        visitor.visit_u16(v as u16)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        let v: u64 = self.parse_unsigned()?;
        visitor.visit_u32(v as u32)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_unsigned()?)
    }

    // Float parsing is stupidly hard.
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        let v: f64 = self.parse_float()?;
        visitor.visit_f32(v as f32)
    }

    // Float parsing is stupidly hard.
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_float()?)
    }

    // The `Serializer` implementation on the previous page serialized chars as
    // single-character strings so handle that representation here.
    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        // Parse a string, check that it is one character, call `visit_char`.
        unimplemented!()
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.parse_string()?)
//        unimplemented!();
        //visitor.visit_borrowed_str(self.parse_string()?.into())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.parse_string()?)
            //self.deserialize_str(visitor)
    }

    // The `Serializer` implementation on the previous page serialized byte
    // arrays as JSON arrays of bytes. Handle that representation here.
    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // An absent optional is represented as the JSON `null` and a present
    // optional is represented as just the contained value.
    //
    // As commented in `Serializer` implementation, this is a lossy
    // representation. For example the values `Some(())` and `None` both
    // serialize as just `null`. Unfortunately this is typically what people
    // expect when working with JSON. Other formats are encouraged to behave
    // more intelligently if possible.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        match self.pj.tape.get(self.idx) {
            Some((_, ItemType::Null)) => {
                self.idx += 1;
                visitor.visit_none()
            }
            Some(_) => {
                visitor.visit_some(self)
            }
            _ => Err(DeserializerError::EarlyEnd)
        }
    }

    // In Serde, unit means an anonymous value containing no data.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        if let Some((_, ItemType::Null)) = self.pj.tape.get(self.idx) {
            self.idx += 1;
            visitor.visit_unit()
        } else {
            Err(DeserializerError::ExpectedNull)
        }
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain. That means not
    // parsing anything other than the contained value.
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    
    {
        // Parse the opening bracket of the sequence.
        if let Some((_depth, ItemType::Array)) = self.pj.tape.get(self.idx) {
            self.idx += 1;
            self.depth += 1;
            let value = visitor.visit_seq(&mut self)?;
            self.depth -= 1;
            Ok(value)
        } else {
            Err(DeserializerError::ExpectedArray)
        }
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Tuple structs look just like sequences in JSON.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Much like `deserialize_seq` but calls the visitors `visit_map` method
    // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
    // method with a `SeqAccess` implementation.
    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        if let Some((_depth, ItemType::Object)) = self.pj.tape.get(self.idx) {
            self.idx += 1;
            self.depth += 1;
            let value = visitor.visit_map(&mut self)?;
            self.depth -= 1;
            Ok(value)
        } else {
            Err(DeserializerError::ExpectedMap)
        }


    }

    // Structs look just like maps in JSON.
    //
    // Notice the `fields` parameter - a "struct" in the Serde data model means
    // that the `Deserialize` implementation is required to know what the fields
    // are before even looking at the input data. Any key-value pairing in which
    // the fields cannot be known ahead of time is probably a map.
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        match self.pj.tape.get(self.idx) {
            Some((_, ItemType::String)) => visitor.visit_enum(self.parse_string()?.into_deserializer()),
            _ => Err(DeserializerError::ExpectedEnum)
            // Some((_, ItemType::Object)) => {
            //     self.idx += 1;
            //     let value = visitor.visit_enum(Enum::new(self))?;
            //     if let Some((_depth_end, ItemType::Object)) = self.pj.tape.get(self.idx) {
            //         //TODO: check depth but it should be fine!
            //         self.idx += 1;
            //         Ok(value)
            //     } else {
            //         Err(DeserializerError::ExpectedMapEnd)
            //     }
            // }
        }
    }

    // An identifier in Serde is the type that identifies a field of a struct or
    // the variant of an enum. In JSON, struct fields and enum variants are
    // represented as strings. In other formats they may be represented as
    // numeric indices.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // Like `deserialize_any` but indicates to the `Deserializer` that it makes
    // no difference which `Visitor` method is called because the data is
    // ignored.
    //
    // Some deserializers are able to implement this more efficiently than
    // `deserialize_any`, for example by rapidly skipping over matched
    // delimiters without paying close attention to the data in between.
    //
    // Some formats are not able to implement this at all. Formats that can
    // implement `deserialize_any` and `deserialize_ignored_any` are known as
    // self-describing.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, DeserializerError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> SeqAccess<'de> for Deserializer<'de> {
    type Error = DeserializerError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, DeserializerError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.cur() {
            Some((_d, ItemType::ArrayEnd)) => {
                self.idx += 1;
                Ok(None)
            },
            Some(_) => seed.deserialize(self).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        //TODO
        // match self.iter.size_hint() {
        //     (lower, Some(upper)) if lower == upper => Some(upper),
        //     _ => None,
        // }
        None
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a> MapAccess<'de> for Deserializer<'de> {
    type Error = DeserializerError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, DeserializerError>
    where
        K: DeserializeSeed<'de>,
    {
        match self.cur() {
            Some((_d, ItemType::ObjectEnd)) => {
                self.idx += 1;
                Ok(None)
            },
            Some((_, ItemType::String)) => seed.deserialize(self).map(Some),
            Some((_, _t)) => Err(DeserializerError::BadKeyType),
            None => Ok(None),
        }

    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, DeserializerError>
    where
        V: DeserializeSeed<'de>,
    {
        match self.cur() {
            Some((_d, ItemType::ObjectEnd)) => {
                self.idx += 1;
                Err(DeserializerError::Syntax)
            },
            Some(_) => seed.deserialize(self),
            None => Err(DeserializerError::Syntax),
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::{self, Value, json};
    use super::*;

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
    91,
    102,
    97,
    108,
    115,
    101,
    44,
    123,
    34,
    92,
    117,
    48,
    48,
    48,
    101,
    92,
    98,
    48,
    97,
    92,
    117,
    48,
    48,
    48,
    101,
    240,
    144,
    128,
    128,
    48,
    65,
    35,
    32,
    92,
    117,
    48,
    48,
    48,
    48,
    97,
    240,
    144,
    128,
    128,
    240,
    144,
    128,
    128,
    240,
    144,
    128,
    128,
    240,
    144,
    128,
    128,
    65,
    32,
    240,
    144,
    128,
    128,
    92,
    92,
    34,
    58,
    110,
    117,
    108,
    108,
    125,
    93
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
            8, // 8 levels deep
            256, // Shoot for maximum size of 256 nodes
            10, // We put up to 10 items per collection
            |inner| prop_oneof![
                // Take the inner strategy and make the two recursive cases.
                prop::collection::vec(inner.clone(), 0..10)
                    .prop_map(|v| json!(v)),
                prop::collection::hash_map(".*", inner, 0..10)
                    .prop_map(|m| json!(m)),
            ]).boxed()
    }

    proptest! {
        #[test]

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
