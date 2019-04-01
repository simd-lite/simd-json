#![feature(ptr_offset_from)]

mod charutils;
mod numberparse;
mod parsedjson;
mod portability;
mod stage1;
mod stage2;
mod stringparse;
mod utf8check;

use crate::numberparse::Number;
use crate::numberparse::*;
use crate::portability::*;
use crate::stage2::*;
use crate::stringparse::*;
use hashbrown::HashMap;
use serde::forward_to_deserialize_any;
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::mem;
use std::ops::{AddAssign, MulAssign, Neg};
use std::str;

pub type Map<'a> = HashMap<&'a str, Value<'a>>;

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Array(Vec<Value<'a>>),
    Bool(bool),
    Map(Map<'a>),
    Null,
    Number(Number),
    String(&'a str),
}

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

// FROM serde-json
// We only use our own error type; no need for From conversions provided by the
// standard library's try! macro. This reduces lines of LLVM IR by 4%.
macro_rules! stry {
    ($e:expr) => {
        match $e {
            ::std::result::Result::Ok(val) => val,
            ::std::result::Result::Err(err) => return ::std::result::Result::Err(err),
        }
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

#[derive(Debug, PartialEq)]
pub enum Error {
    BadKeyType,
    EarlyEnd,
    ExpectedArray(usize, char),
    ExpectedArrayComma(usize, char),
    ExpectedBoolean,
    ExpectedString,
    ExpectedSigned,
    ExpectedUnsigned,
    ExpectedEnum,
    ExpectedInteger,
    ExpectedNumber,
    ExpectedMap(usize, char),
    ExpectedMapColon(usize, char),
    ExpectedMapComma,
    ExpectedMapEnd,
    ExpectedNull,
    InternalError,
    InvalidEscape(usize, char),
    InvalidUTF8,
    InvalidUnicodeEscape,
    InvlaidUnicodeCodepoint,
    NoStructure,
    Parser,
    Serde(String),
    Syntax,
    TrailingCharacters,
    UnexpectedCharacter(usize, char),
    UnexpectedEnd(usize),
    UnterminatedString,
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
    strings: Vec<u8>,
    sidx: usize,
    pj: ParsedJson,
    idx: usize,
}

impl<'de> Deserializer<'de> {
    // By convention, `Deserializer` constructors are named like `from_xyz`.
    // That way basic use cases are satisfied by something like
    // `serde_json::from_str(...)` while advanced use cases that require a
    // deserializer can make one with `serde_json::Deserializer::from_str(...)`.
    pub fn from_slice(input: &'de mut [u8]) -> Result<Self> {
        let mut pj = ParsedJson::from_slice();
        let len = input.len();
        unsafe {
            stry!(find_structural_bits(input, len as u32, &mut pj));
            //unified_machine(input, input.len(), &mut pj);
        };
        let mut v = Vec::with_capacity(len + SIMDJSON_PADDING);
        unsafe {
            v.set_len(len + SIMDJSON_PADDING);
        }
        dbg!(&pj);
        Ok(Deserializer {
            input,
            pj,
            idx: 0,
            strings: v,
            sidx: 0,
        })
    }

    fn skip(&mut self) {
        dbg!(self.idx);
        self.idx += 1;
    }

    fn idx(&self) -> usize {
        self.pj.structural_indexes[self.idx] as usize
    }

    fn c(&self) -> u8 {
        self.input[self.pj.structural_indexes[self.idx] as usize]
    }

    fn next(&mut self) -> Result<u8> {
        self.idx += 1;
        if let Some(idx) = self.pj.structural_indexes.get(self.idx) {
            let r = self.input[*idx as usize];
            Ok(r)
        } else {
            Err(Error::UnexpectedEnd(self.idx))
        }
    }

    fn peek(&self) -> Result<u8> {
        if let Some(idx) = self.pj.structural_indexes.get(self.idx + 1) {
            let idx = *idx as usize;
            let r = self.input[idx];
            Ok(r)
        } else {
            Err(Error::UnexpectedEnd(self.idx+1))
        }
    }

    fn at(&self, idx: usize) -> Option<&u8> {
        self.input.get(idx)
    }

    /*
    pub fn to_value(&mut self) -> Result<Value<'de>> {
        match stry!(self.peek()) {
            b'n' => {
                stry!(self.parse_null());
                Ok(Value::Null)
            }
            b't' | b'f' => self.parse_bool().map(Value::Bool),
            b'0'...b'9' | b'-' => self.parse_number().map(Value::Number),
            b'"' => self.parse_str().map(Value::String),
            b'[' => self.parse_array().map(Value::Array),
            b'{' => self.parse_map().map(Value::Map),
            c => Err(Error::UnexpectedCharacter(c as char)),
        }
    }
     */

    pub fn to_value(&mut self) -> Result<Value<'de>> {
        dbg!(self.idx);
        match stry!(self.next()) {
            b'n' => {
                stry!(self.parse_null_());
                Ok(Value::Null)
            }
            b't' | b'f' => self.parse_bool_().map(Value::Bool),
            b'0'...b'9' | b'-' => {
                let v = stry!(self.parse_number_());
                Ok(Value::Number(v))
            }
            b'"' => self.parse_str_().map(Value::String),
            b'[' => {
                let a = stry!(self.parse_array_());
                Ok(Value::Array(a))
            }
            b'{' => self.parse_map_().map(Value::Map),
            c => Err(Error::UnexpectedCharacter(self.idx(), c as char)),
        }
    }

    fn count_elements(&self, mut idx: usize) -> Result<usize> {
        let mut depth = 0;
        let mut count = 0;
        loop {
            match self.at(idx) {
                Some(b'[') if depth == 0 => {
                    depth += 1;
                    count += 1;
                }
                Some(b'[') => depth += 1,
                Some(b']') if depth == 0 => return Ok(count + 1),
                Some(b']') => depth -= 1,
                Some(b'{') if depth == 0 => {
                    depth += 1;
                    count += 1;
                }
                Some(b'{') => depth += 1,
                Some(b'}') if depth == 0 => return Ok(count + 1),
                Some(b'}') => depth -= 1,
                None => return Err(Error::Syntax),
                Some(b',') if depth == 0 => count += 1,
                _ => (),
            }
            idx += 1
        }
    }

    fn parse_array_(&mut self) -> Result<Vec<Value<'de>>> {

        dbg!(self.idx);
        dbg!(self.c() as char);

        // We short cut for empty arrays
        if stry!(self.peek()) == b']' {
            self.skip();
            return Ok(Vec::new());
        }

        let mut res = Vec::with_capacity(stry!(self.count_elements(self.idx)));

        // Since we checked if it's empty we know that we at least have one
        // element so we eat this

        res.push(stry!(self.to_value()));
        loop {
            // We now exect one of two things, a comma with a next
            // element or a closing bracket
            match stry!(self.peek()) {
                b']' => {
                    self.skip();
                    break;
                }
                b',' => self.skip(),
                c => return Err(Error::ExpectedArrayComma(self.idx(), c as char)),
            }
            dbg!();
            res.push(stry!(self.to_value()));
        }
        // We found a closing bracket and ended our loop, we skip it
        Ok(res)
    }

    fn parse_map(&mut self) -> Result<Map<'de>> {
        let c = stry!(self.next());
        if c == b'{' {
            self.parse_map()
        } else {
            Err(Error::ExpectedMap(self.idx(), c as char))
        }
    }

    fn parse_map_(&mut self) -> Result<Map<'de>> {
        // We short cut for empty arrays

        if stry!(self.peek()) == b'}' {
            self.skip();
            return Ok(Map::new());
        }

        let mut res = Map::with_capacity(stry!(self.count_elements(self.idx)));

        // Since we checked if it's empty we know that we at least have one
        // element so we eat this

        let key = stry!(self.parse_str());

        match stry!(self.next()) {
            b':' => (),
            c => return Err(Error::ExpectedMapColon(self.idx(), c as char)),
        }
        res.insert(key, stry!(self.to_value()));
        loop {
            // We now exect one of two things, a comma with a next
            // element or a closing bracket
            match stry!(self.peek()) {
                b'}' => break,
                b',' => self.skip(),
                c => return Err(Error::ExpectedArrayComma(self.idx(), c as char)),
            }
            let key = stry!(self.parse_str());

            match stry!(self.next()) {
                b':' => (),
                c => return Err(Error::ExpectedMapColon(self.idx(), c as char)),
            }
            res.insert(key, stry!(self.to_value()));
        }
        // We found a closing bracket and ended our loop, we skip it
        self.skip();
        Ok(res)
    }
    fn parse_str(&mut self) -> Result<&'de str> {
        if stry!(self.next()) != b'"' {
            return Err(Error::ExpectedString);
        }
        self.parse_str_()
    }
    //#[inline(always)]
    fn parse_str_(&mut self) -> Result<&'de str> {
        use std::num::Wrapping;
        // Add 1 to skip the initial "
        let idx = self.idx() + 1;
        let mut padding = [0u8; 32];
        let mut read: usize = 0;
        let mut written: usize = 0;
        #[cfg(test1)]
        {
            dbg!(idx);
            dbg!(end);
        }
        // we include the terminal '"' so we know where to end
        // This is safe since we check sub's lenght in the range access above and only
        // create sub sliced form sub to `sub.len()`.
        let mut dst: &mut [u8] = &mut self.strings[self.sidx..];
        let mut src: &[u8] = &self.input[idx..];
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
                    _mm256_set1_epi8(b'\\' as i8)
                )))
            };
            let quote_mask = unsafe { _mm256_cmpeq_epi8(v, _mm256_set1_epi8(b'"' as i8)) };
            let quote_bits = unsafe { static_cast_u32!(_mm256_movemask_epi8(quote_mask)) };
            if ((Wrapping(bs_bits) - Wrapping(1)).0 & quote_bits) != 0 {
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
                //let s = String::from_utf8_lossy(&self.strings[self.sidx..self.sidx + written as usize]).to_string();
                unsafe {
                    // We need to copy this back into the original data structure to guarantee that it lives as long as claimed.
                    self.input[idx..idx + written]
                        .clone_from_slice(&self.strings[self.sidx..self.sidx + written]);
                    //let v = &self.strings[self.sidx..self.sidx + written as usize] as *const [u8] as *const str;

                    let v = &self.input[idx..idx + written] as *const [u8] as *const str;
                    self.sidx += written;
                    return Ok(&*v);
                }
                /*
                return Ok(str::from_utf8_unchecked(s));
                     */
                // we compare the pointers since we care if they are 'at the same spot'
                // not if they are the same value
            }
            if ((Wrapping(quote_bits) - Wrapping(1)).0 & bs_bits) != 0 {
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
                    read += bs_dist as usize;
                    dst = &mut dst[bs_dist as usize..];
                    written += bs_dist as usize;
                    let (o, s) = handle_unicode_codepoint(src, dst);
                    if o == 0 {
                        return Err(Error::InvlaidUnicodeCodepoint);
                    };
                    // We moved o steps forword at the destiation and 6 on the source
                    src = &src[s..];
                    read += s;
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
                        return Err(Error::InvalidEscape(self.idx(), escape_char as char));
                    }
                    dst[bs_dist as usize] = escape_result;
                    src = &src[bs_dist as usize + 2..];
                    read += bs_dist as usize + 2;
                    dst = &mut dst[bs_dist as usize + 1..];
                    written += bs_dist as usize + 1;
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

    fn parse_null(&mut self) -> Result<()> {
        if stry!(self.next()) == b'n' {
            self.parse_null()
        } else {
            Err(Error::ExpectedNull)
        }
    }

    fn parse_null_(&mut self) -> Result<()> {
        let input = &self.input[self.idx()..];
        let len = input.len();
        if len < SIMDJSON_PADDING {
            let mut copy = vec![0u8; len + SIMDJSON_PADDING];
            copy[0..len].clone_from_slice(input);
            if is_valid_null_atom(&copy) {
                Ok(())
            } else {
                Err(Error::ExpectedNull)
            }
        } else {
            if is_valid_null_atom(input) {
                Ok(())
            } else {
                Err(Error::ExpectedNull)
            }
        }
    }

    fn parse_bool(&mut self) -> Result<bool> {
        stry!(self.next());
        self.parse_bool_()
    }

    fn parse_bool_(&mut self) -> Result<bool> {
        match self.c() {
            b't' => {
                let input = &self.input[self.idx()..];
                let len = input.len();
                if len < SIMDJSON_PADDING {
                    let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                    unsafe {
                        copy.as_mut_ptr().copy_from(input.as_ptr(), len);
                    };
                    if is_valid_true_atom(&copy) {
                        Ok(true)
                    } else {
                        Err(Error::ExpectedBoolean)
                    }
                } else {
                    if is_valid_true_atom(input) {
                        Ok(true)
                    } else {
                        Err(Error::ExpectedBoolean)
                    }
                }
            }
            b'f' => {
                let input = &self.input[self.idx()..];
                let len = input.len();
                if len < SIMDJSON_PADDING {
                    let mut copy = vec![0u8; len + SIMDJSON_PADDING];
                    unsafe {
                        copy.as_mut_ptr().copy_from(input.as_ptr(), len);
                    };
                    if is_valid_false_atom(&copy) {
                        Ok(false)
                    } else {
                        Err(Error::ExpectedBoolean)
                    }
                } else {
                    if is_valid_false_atom(input) {
                        Ok(false)
                    } else {
                        Err(Error::ExpectedBoolean)
                    }
                }
            }
            _ => Err(Error::ExpectedBoolean),
        }
    }

    fn parse_number(&mut self) -> Result<Number> {
        match stry!(self.next()) {
            b'0'...b'9' | b'-' => self.parse_number_(),
            _ => Err(Error::ExpectedNumber),
        }
    }

    fn parse_number_(&mut self) -> Result<Number> {
        let input = &self.input[self.idx()..];
        let len = input.len();
        if len < SIMDJSON_PADDING {
            let mut copy = vec![0u8; len + SIMDJSON_PADDING];
            unsafe {
                copy.as_mut_ptr().copy_from(input.as_ptr(), len);
            };
            parse_number(&copy, self.c() == b'-')
        } else {
            parse_number(input, self.c() == b'-')
        }
    }

    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + From<i64>,
    {
        match stry!(self.parse_number()) {
            Number::I64(i) => Ok(T::from(i)),
            _ => Err(Error::ExpectedSigned),
        }
    }

    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: AddAssign<T> + MulAssign<T> + From<u64>,
    {
        match stry!(self.parse_number()) {
            Number::I64(i) if i >= 0 => Ok(T::from(i as u64)),
            _ => Err(Error::ExpectedUnsigned),
        }
    }
}

pub fn from_slice<'a, T>(s: &'a mut [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = stry!(Deserializer::from_slice(s));

    T::deserialize(&mut deserializer)
}

pub fn to_value<'a>(s: &'a mut [u8]) -> Result<Value<'a>> {
    let mut deserializer = stry!(Deserializer::from_slice(s));
    deserializer.to_value()
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    //#[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        dbg!(stry!(self.peek()) as char);
        match stry!(self.next()) {
            b'n' => {
                stry!(self.parse_null_());
                visitor.visit_unit()
            }
            b't' | b'f' => visitor.visit_bool(stry!(self.parse_bool_())),
            b'0'...b'9' | b'-' => match stry!(self.parse_number_()) {
                Number::F64(n) => visitor.visit_f64(n),
                Number::I64(n) => visitor.visit_i64(n),
            },
            b'"' => visitor.visit_borrowed_str(stry!(self.parse_str_())),
            b'[' => visitor.visit_seq(CommaSeparated::new(&mut self)),
            b'{' => visitor.visit_map(CommaSeparated::new(&mut self)),
            c => Err(Error::UnexpectedCharacter(self.idx(), c as char)),
        }
    }
    /*

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
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.c() != b'"' {
            return Err(Error::ExpectedString);
        }
        visitor.visit_borrowed_str(stry!(self.parse_str_()))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // The `parse_signed` function is generic over the integer type `T` so here
    // it is invoked with `T=i8`. The next 8 methods are similar.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v: i64 = stry!(self.parse_signed());
        visitor.visit_i8(v as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v: i64 = stry!(self.parse_signed());
        visitor.visit_i16(v as i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v: i64 = stry!(self.parse_signed());
        visitor.visit_i32(v as i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(stry!(self.parse_signed()))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v: u64 = stry!(self.parse_unsigned());
        visitor.visit_u8(v as u8)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v: u64 = stry!(self.parse_unsigned());
        visitor.visit_u16(v as u16)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v: u64 = stry!(self.parse_unsigned());
        visitor.visit_u32(v as u32)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(stry!(self.parse_unsigned()))
    }

    // An absent optional is represented as the JSON `null` and a present
    // optional is represented as just the contained value.
    //
    // As commented in `Serializer` implementation, this is a lossy
    // representation. For example the values `Some(())` and `None` both
    // serialize as just `null`. Unfortunately this is typically what people
    // expect when working with JSON. Other formats are encouraged to behave
    // more intelligently if possible.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if stry!(self.peek()) == b'n' {
            stry!(self.parse_null());
            visitor.visit_unit()
        } else {
            visitor.visit_some(self)
        }
    }

    // In Serde, unit means an anonymous value containing no data.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        stry!(self.parse_null());
        visitor.visit_unit()
    }

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        dbg!();
        // Parse the opening bracket of the sequence.
        if stry!(self.next()) == b'[' {
            // Give the visitor access to each element of the sequence.
            visitor.visit_seq(CommaSeparated::new(&mut self))
        } else {
            Err(Error::ExpectedArray(self.idx(), self.c() as char))
        }
    }

     */

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        dbg!();
        let r = self.deserialize_seq(visitor);
        // tuples have a known length damn you serde ...
        self.skip();
        r
    }

    forward_to_deserialize_any! {
        seq  bool i8 i16 i32 i64 u8 u16 u32 u64 string str option unit 
        i128 u128 f32 f64 char
            bytes byte_buf  unit_struct newtype_struct
            tuple_struct map struct enum identifier ignored_any
    }
}

// In order to handle commas correctly when deserializing a JSON array or map,
// we need to track whether we are on the first element or past the first
// element.
struct CommaSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
    idx: usize,
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        println!("==[{}] Start array: {}", de.idx, de.input[de.pj.structural_indexes[de.idx] as usize] as char);
        CommaSeparated { first: true, idx: de.idx, de }
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

        /*
        println!("===[{}] loop", self.idx);
        // If the next structural  would be a ] eat it and end the array
        if self.done {
            return Ok(None)
        }
        let r = if self.first {
            println!("===[{}] first", self.idx);
            if stry!(self.de.peek()) == b']' {
                println!("==[{}] Ending array", self.idx);
                return Ok(None)
            };
            self.first = false;
            println!("===[{}] value => {}", self.idx, stry!(self.de.peek()) as char );
            println!("===[{}] loop end 2", self.idx);
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            println!("===[{}] successive", self.idx);
            match stry!(self.de.next()) {
                b','  => (),
                b']' => {
                    println!("===[{}] Ending array", self.idx);
                    return Ok(None)
                },
                c => return Err(Error::ExpectedArrayComma(self.de.idx(), c as char)),
            }
            println!("===[{}] value => {}", self.idx, stry!(self.de.peek()) as char);
        seed.deserialize(&mut *self.de).map(Some)
        };

        // Serde is evil it won't ask for the next element if it knows the length so we
        // have to make sure we check if the next iteration would be a terminal ] and
        // if so consume it.
        if self.de.c() == b']' {
            println!("===[{}] Ending array", self.idx);
            self.de.skip();
            self.done = true
        }
        r
         */

        let peek = match stry!(self.de.peek()) {
            b']' => {
                println!("===[{}] Ending array", self.idx);
                self.de.skip();
                return Ok(None);
            }
            b',' if !self.first => {
                stry!(self.de.next())
            }
            b => {
                if self.first {
                    self.first = false;
                    b
                } else {
                    return Err(Error::ExpectedArrayComma(self.de.idx(), b as char))
                }
            }
        };
        match peek {
            b']' => Err(Error::ExpectedArrayComma(self.de.idx(), ']')),
            _ => Ok(Some(stry!(seed.deserialize(&mut *self.de)))),
        }

    }
    /*
    fn size_hint(&self) -> Option<usize> {
        let mut depth = 0;
        let mut count = 0;
        let mut i = self.de.idx;
        loop {
            match self.de.at(i) {
                Some(b'[') if depth == 0 => {
                    depth += 1;
                    count += 1;
                }
                Some(b'[') => depth += 1,
                Some(b']') if depth == 0 => break,
                Some(b']') => depth -= 1,
                Some(b'{') if depth == 0 => {
                    depth += 1;
                    count += 1;
                }
                Some(b'{') => depth += 1,
                Some(b'}') => depth -= 1,
                None => break,
                _ if depth == 0 => count += 1,
                _ => (),
            }
            i += 1
        }
        Some(count)
    }*/
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
        if stry!(self.de.peek()) == b'}' {
            return Ok(None)
        };

        if self.first {
            self.first = false;
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            match stry!(self.de.next()) {
                b','  => (),
                _c => return Err(Error::ExpectedMapComma),
            }
            seed.deserialize(&mut *self.de).map(Some)
        }

    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // It doesn't make a difference whether the colon is parsed at the end
        // of `next_key_seed` or at the beginning of `next_value_seed`. In this
        // case the code is a bit simpler having it here.
        let c = self.de.c();
        if c != b':' {
            return Err(Error::ExpectedMapColon(self.de.idx(), c as char));
        }
        self.de.skip();
        // Deserialize a map value.
        seed.deserialize(&mut *self.de)
    }
    fn size_hint(&self) -> Option<usize> {
        let mut depth = 0;
        let mut count = 0;
        let mut i = self.de.idx;
        loop {
            match self.de.at(i) {
                Some(b'[') if depth == 0 => {
                    depth += 1;
                    count += 1;
                }
                Some(b'[') => depth += 1,
                Some(b']') => depth -= 1,
                Some(b'{') if depth == 0 => {
                    depth += 1;
                    count += 1;
                }
                Some(b'{') => depth += 1,
                Some(b'}') if depth == 0 => break,
                Some(b'}') => depth -= 1,
                None => break,
                _ if depth == 0 => count += 1,
                _ => (),
            }
            i += 1
        }
        Some(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde::Deserialize;
    use serde_json::{self, json};

    #[test]
    #[test]
    fn bool_true() {
        let mut d = String::from("true");
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };

        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        assert_eq!(to_value(&mut d1), Ok(Value::Bool(true)));
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
        assert_eq!(to_value(&mut d1), Ok(Value::Bool(false)));
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
        assert_eq!(to_value(&mut d1), Ok(Value::Number(Number::I64(42))));
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
        assert_eq!(to_value(&mut d1), Ok(Value::Number(Number::I64(0))));
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
        assert_eq!(to_value(&mut d1), Ok(Value::Number(Number::I64(1))));
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
        assert_eq!(to_value(&mut d1), Ok(Value::Number(Number::I64(-1))));
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
        assert_eq!(to_value(&mut d1), Ok(Value::Number(Number::F64(23.0))));
    }

    #[test]
    fn string() {
        let mut d = String::from(r#""snot""#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(to_value(&mut d1), Ok(Value::String("snot")));
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
        assert_eq!(to_value(&mut d1), Ok(Value::String("")));
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
        //assert_eq!(to_value(&mut d1), Ok(Value::Array(vec![])));
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
            Ok(Value::Array(vec![Value::String("snot")]))
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
                Value::String("snot"),
                Value::String("badger")
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
                Value::Number(Number::I64(42)),
                Value::Number(Number::F64(23.0)),
                Value::String("snot badger")
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
                Value::Number(Number::I64(42)),
                Value::Array(vec![Value::Number(Number::F64(23.0)), Value::String("snot")]),
                Value::String("bad"),
                Value::String("ger")
            ])));

        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn nested_list2() {
        let mut d = String::from(r#"[42, [23.0, "snot"], {"bad": "ger"}]"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
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
            Ok(Value::Array(vec![Value::Map(Map::new()), Value::Null]))
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
    fn map() {
        let mut d = String::from(r#"{"snot": "badger"}"#);
        let mut d1 = d.clone();
        let mut d1 = unsafe { d1.as_bytes_mut() };
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
        let mut h = Map::new();
        h.insert("snot", Value::String("badger"));
        assert_eq!(to_value(&mut d1), Ok(Value::Map(h)));
    }

    #[test]
    fn tpl1() {
        let mut d = String::from("[-65.613616999999977, 43.420273000000009]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: (f32, f32) = serde_json::from_slice(d).expect("serde_json");
        let v_simd: (f32, f32) = from_slice(&mut d).expect("simd_json");
        //        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn tpl2() {
        let mut d = String::from("[[-65.613616999999977, 43.420273000000009]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<(f32, f32)> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<(f32, f32)> = from_slice(&mut d).expect("simd_json");
        //        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn tpl3() {
        let mut d = String::from("[[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<(f32, f32)> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<(f32, f32)> = from_slice(&mut d).expect("simd_json");
        //        assert_eq!(v_simd, v_serde)
    }
    #[test]
    fn tpl4() {
        let mut d = String::from("[[[-65.613616999999977,43.420273000000009]]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<Vec<(f32, f32)>> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<Vec<(f32, f32)>> = from_slice(&mut d).expect("simd_json");
        //        assert_eq!(v_simd, v_serde)
    }
    #[test]
    fn tpl5() {
        let mut d = String::from("[[[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<Vec<(f32, f32)>> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<Vec<(f32, f32)>> = from_slice(&mut d).expect("simd_json");
        //        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn tpl6() {
        let mut d = String::from("[[[[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]]]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<Vec<Vec<(f32, f32)>>> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<Vec<Vec<(f32, f32)>>> = from_slice(&mut d).expect("simd_json");
        //        assert_eq!(v_simd, v_serde)
    }

    #[test]
    fn vecvec() {
        let mut d = String::from("[[[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]], [[-65.613616999999977,43.420273000000009], [-65.613616999999977,43.420273000000009]]]");
        let mut d = unsafe { d.as_bytes_mut() };
        let v_serde: Vec<Vec<(f32, f32)>> = serde_json::from_slice(d).expect("serde_json");
        let v_simd: Vec<Vec<(f32, f32)>> = from_slice(&mut d).expect("simd_json");
        //        assert_eq!(v_simd, v_serde)
    }

    fn arb_json() -> BoxedStrategy<String> {
        let leaf = prop_oneof![
            Just(serde_json::Value::Null),
            any::<bool>().prop_map(serde_json::Value::Bool),
            (-1.0e308f64..1.0e308f64).prop_map(|f| json!(f)),
            any::<i64>().prop_map(|i| json!(i)),
            ".*".prop_map(serde_json::Value::String),
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
//        #[test]
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
