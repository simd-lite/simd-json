mod charutils;
mod numberparse;
mod parsedjson;
mod portability;
mod stage1;
mod stage2;
mod stringparse;
mod utf8check;

use crate::numberparse::Number;
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
//use std::ops::{AddAssign, MulAssign, Neg};
use std::str;

#[macro_use]
extern crate lazy_static;

pub type Map<'a> = HashMap<&'a str, Value<'a>>;


// We only do this for the string parse function as it seems to slow down other frunctions
// odd...
lazy_static! {
    static ref MM256_SET1_EPI8_SLASH: __m256i = {unsafe{ _mm256_set1_epi8(b'\\' as i8)}};
    static ref MM256_SET1_EPI8_QUOTE: __m256i = {unsafe{ _mm256_set1_epi8(b'"' as i8)}};
}

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

use serde::Deserialize;
//use serde::de::Deserializer as DeserializerT;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ErrorType {
    BadKeyType,
    EarlyEnd,
    ExpectedArray,
    ExpectedArrayComma,
    ExpectedBoolean,
    ExpectedString,
    ExpectedSigned,
    ExpectedUnsigned,
    ExpectedEnum,
    ExpectedInteger,
    ExpectedNumber,
    ExpectedMap,
    ExpectedMapColon,
    ExpectedMapComma,
    ExpectedMapEnd,
    ExpectedNull,
    InvalidNumber,
    InvalidExponent,
    InternalError,
    InvalidEscape,
    InvalidUTF8,
    InvalidUnicodeEscape,
    InvlaidUnicodeCodepoint,
    NoStructure,
    Parser,
    Serde(String),
    Syntax,
    TrailingCharacters,
    UnexpectedCharacter,
    UnexpectedEnd,
    UnterminatedString,
}

#[derive(Debug, PartialEq)]
pub struct Error {
    structural: usize,
    index: usize,
    character: char,
    error: ErrorType,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} at chracter {} ('{}')",
            self.error, self.index, self.character
        )
    }
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error {
            structural: 0,
            index: 0,
            character: '💩', //this is the poop emoji
            error: ErrorType::Serde(msg.to_string()),
        }
    }
}

pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    input: &'de mut [u8],
    strings: Vec<u8>,
    structural_indexes: Vec<u32>,
    idx: usize,
    counts: Vec<usize>,
}

impl<'de> Deserializer<'de> {
    fn error(&self, error: ErrorType) -> Error {
        Error {
            structural: self.idx,
            index: self.idx(),
            character: self.c() as char,
            error,
        }
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
                return Err(Error {
                    structural: 0,
                    index: 0,
                    character: '💩', //this is the poop emoji
                    error: t,
                });
            }
        };
        let (counts, str_len) = Deserializer::compute_size(input, &structural_indexes)?;

        let mut v = Vec::with_capacity(str_len + SIMDJSON_PADDING*2);
        unsafe {
            v.set_len(str_len +  SIMDJSON_PADDING*2);
        };

        Ok(Deserializer {
            counts,
            structural_indexes,
            input,
            idx: 0,
            strings: v,
        })
    }

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
            let idx = structural_indexes[i];
            match input[idx as usize] {
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
                    let (a_last_start, a_cnt) = stry!(depth.pop().ok_or_else(|| (Error {
                        structural: 0,
                        index: 0,
                        character: '💩', //this is the poop emoji
                        error: ErrorType::Syntax
                    })));
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
                    let (a_last_start, a_cnt) = stry!(depth.pop().ok_or_else(|| (Error {
                        structural: 0,
                        index: 0,
                        character: '💩', //this is the poop emoji
                        error: ErrorType::Syntax
                    })));
                    counts[last_start] = cnt;
                    last_start = a_last_start;
                    maps.push(cnt);
                    cnt = a_cnt;
                }
                b',' => cnt += 1,
                b'"' => {
                    if let Some(next) = structural_indexes.get(i + 1) {
                        let d = next - idx;
                        if d > str_len {
                            str_len = d;
                        }
                    }
                }
                _ => (),
            }
        }
        Ok((counts, str_len as usize))
    }

    #[cfg_attr(feature = "inline", inline(always))]
    fn skip(&mut self) {
        self.idx += 1;
    }

    #[cfg_attr(feature = "inline", inline(always))]
    fn idx(&self) -> usize {
        self.structural_indexes[self.idx] as usize
    }

    #[cfg_attr(feature = "inline", inline(always))]
    fn c(&self) -> u8 {
        self.input[self.structural_indexes[self.idx] as usize]
    }

    #[cfg_attr(feature = "inline", inline(always))]
    fn next(&mut self) -> Result<u8> {
        self.idx += 1;
        if let Some(idx) = self.structural_indexes.get(self.idx) {
            let r = self.input[*idx as usize];
            Ok(r)
        } else {
            Err(self.error(ErrorType::UnexpectedEnd))
        }
    }

    // pull out the check so we don't need to
    // stry every time
    #[cfg_attr(feature = "inline", inline(always))]
    fn next_(&mut self) -> u8 {
        self.idx += 1;
        self.input[self.structural_indexes[self.idx] as usize]
    }

    #[cfg_attr(feature = "inline", inline(always))]
    fn peek(&self) -> Result<u8> {
        if let Some(idx) = self.structural_indexes.get(self.idx + 1) {
            let idx = *idx as usize;
            let r = self.input[idx];
            Ok(r)
        } else {
            Err(self.error(ErrorType::UnexpectedEnd))
        }
    }

    #[cfg_attr(feature = "inline", inline(always))]
    pub fn to_value(&mut self) -> Result<Value<'de>> {
        if self.idx + 1 > self.structural_indexes.len() {
            return Err(self.error(ErrorType::UnexpectedEnd));
        }
        match self.next_() {
            b'"' => self.parse_str_().map(Value::String),
            b'n' => {
                stry!(self.parse_null_());
                Ok(Value::Null)
            }
            b't' => self.parse_true_().map(Value::Bool),
            b'f' => self.parse_false_().map(Value::Bool),
            b'-' => self.parse_number_(true).map(Value::Number),
            b'0'...b'9' => self.parse_number_(false).map(Value::Number),
            b'[' => self.parse_array_().map(Value::Array),
            b'{' => self.parse_map_().map(Value::Map),
            _c => Err(self.error(ErrorType::UnexpectedCharacter)),
        }
    }

    #[cfg_attr(feature = "inline", inline(always))]
    fn count_elements(&self) -> usize {
        self.counts[self.idx]
        /*
        let mut idx = self.idx + 1;
        let mut depth = 0;
        let mut count = 0;
        loop {
            match self.at(idx)? {
                b'[' => depth += 1,
                b']' if depth == 0 => return Some(count + 1),
                b']' => depth -= 1,
                b'{' => depth += 1,
                b'}' if depth == 0 => return Some(count + 1),
                b'}' => depth -= 1,
                b',' if depth == 0 => count += 1,
                _ => (),
            }
            idx += 1
        }
         */
    }

    #[cfg_attr(feature = "inline", inline(always))]
    fn parse_array_(&mut self) -> Result<Vec<Value<'de>>> {
        // We short cut for empty arrays
        if stry!(self.peek()) == b']' {
            self.skip();
            return Ok(Vec::new());
        }

        let mut res = Vec::with_capacity(self.count_elements());

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
                _c => return Err(self.error(ErrorType::ExpectedArrayComma)),
            }
            res.push(stry!(self.to_value()));
        }
        // We found a closing bracket and ended our loop, we skip it
        Ok(res)
    }

    #[cfg_attr(feature = "inline", inline(always))]
    fn parse_map_(&mut self) -> Result<Map<'de>> {
        // We short cut for empty arrays

        if stry!(self.peek()) == b'}' {
            self.skip();
            return Ok(Map::new());
        }

        let mut res = Map::with_capacity(self.count_elements());

        // Since we checked if it's empty we know that we at least have one
        // element so we eat this

        if stry!(self.next()) != b'"' {
            return Err(self.error(ErrorType::ExpectedString));
        }

        let key = stry!(self.parse_short_str_());

        match stry!(self.next()) {
            b':' => (),
            _c => return Err(self.error(ErrorType::ExpectedMapColon)),
        }
        res.insert(key, stry!(self.to_value()));
        loop {
            // We now exect one of two things, a comma with a next
            // element or a closing bracket
            match stry!(self.peek()) {
                b'}' => break,
                b',' => self.skip(),
                _c => return Err(self.error(ErrorType::ExpectedArrayComma)),
            }
            if stry!(self.next()) != b'"' {
                return Err(self.error(ErrorType::ExpectedString));
            }
            let key = stry!(self.parse_short_str_());

            match stry!(self.next()) {
                b':' => (),
                _c => return Err(self.error(ErrorType::ExpectedMapColon)),
            }
            res.insert(key, stry!(self.to_value()));
        }
        // We found a closing bracket and ended our loop, we skip it
        self.skip();
        Ok(res)
    }

    #[cfg_attr(feature = "inline", inline(always))]
    fn parse_str(&mut self) -> Result<&'de str> {
        if stry!(self.next()) != b'"' {
            return Err(self.error(ErrorType::ExpectedString));
        }
        self.parse_str_()
    }

    // We parse a string that's likely to be less then 32 characters and without any
    // fancy in it like object keys
    #[cfg_attr(feature = "inline", inline(always))]
    fn parse_short_str_(&mut self) -> Result<&'de str> {
        use std::num::Wrapping;
        let mut padding = [0u8; 32];
        let idx = self.idx() + 1;
        let mut src: &[u8] = &self.input[idx..];

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
        if ((Wrapping(bs_bits) - Wrapping(1)).0 & quote_bits) != 0 {
            let quote_dist: u32 = trailingzeroes(quote_bits as u64) as u32;
            let v = &self.input[idx..idx + quote_dist as usize] as *const [u8] as *const str;
            unsafe{
                return Ok(&*v);
            }
        }
        self.parse_str_()
    }

    #[cfg_attr(feature = "inline", inline(always))]
    fn parse_str_(&mut self) -> Result<&'de str> {
        use std::num::Wrapping;
        // Add 1 to skip the initial "
        let idx = self.idx() + 1;
        let mut padding = [0u8; 32];
        //let mut read: usize = 0;
        let mut written: usize = 0;
        #[cfg(test1)]
        {
            dbg!(idx);
            dbg!(end);
        }
        // we include the terminal '"' so we know where to end
        // This is safe since we check sub's lenght in the range access above and only
        // create sub sliced form sub to `sub.len()`.
        let mut dst: &mut [u8] = &mut self.strings;
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
                    *MM256_SET1_EPI8_SLASH
                )))
            };
            let quote_mask = unsafe { _mm256_cmpeq_epi8(v, *MM256_SET1_EPI8_QUOTE) };
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
                    self.input[idx..idx + written].clone_from_slice(&self.strings[..written]);
                    //let v = &self.strings[self.sidx..self.sidx + written as usize] as *const [u8] as *const str;

                    let v = &self.input[idx..idx + written] as *const [u8] as *const str;
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

    #[cfg_attr(feature = "inline", inline(always))]
    fn parse_null_(&mut self) -> Result<()> {
        let input = &self.input[self.idx()..];
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

    #[cfg_attr(feature = "inline", inline(always))]
    fn parse_true_(&mut self) -> Result<bool> {
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

    #[cfg_attr(feature = "inline", inline(always))]
    fn parse_false_(&mut self) -> Result<bool> {
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

    #[cfg_attr(feature = "inline", inline(always))]
    fn parse_bool_(&mut self) -> Result<bool> {
        match self.c() {
            b't' => self.parse_true_(),
            b'f' => self.parse_false_(),
            _ => Err(self.error(ErrorType::ExpectedBoolean)),
        }
    }

    /*
    fn parse_number(&mut self) -> Result<Number> {
        match stry!(self.next()) {
            b'0'...b'9' | b'-' => self.parse_number_(),
            _ => Err(self.error(ErrorType::ExpectedNumber)),
        }
    }

    */
    #[cfg_attr(feature = "inline", inline(always))]
    fn parse_number_(&mut self, minus: bool) -> Result<Number> {
        let input = &self.input[self.idx()..];
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

    /*
    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + From<i64>,
    {
        match stry!(self.parse_number()) {
            Number::I64(i) => Ok(T::from(i)),
            _ => Err(self.error(ErrorType::ExpectedSigned)),
        }
    }

    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: AddAssign<T> + MulAssign<T> + From<u64>,
    {
        match stry!(self.parse_number()) {
            Number::I64(i) if i >= 0 => Ok(T::from(i as u64)),
            _ => Err(self.error(ErrorType::ExpectedUnsigned)),
        }
    }
    */
}

#[cfg_attr(feature = "inline", inline(always))]
pub fn from_slice<'a, T>(s: &'a mut [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = stry!(Deserializer::from_slice(s));

    T::deserialize(&mut deserializer)
}

#[cfg_attr(feature = "inline", inline(always))]
pub fn from_str<'a, T>(s: &'a mut str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = stry!(Deserializer::from_slice(unsafe { s.as_bytes_mut() }));

    T::deserialize(&mut deserializer)
}

#[cfg_attr(feature = "inline", inline(always))]
pub fn to_value<'a>(s: &'a mut [u8]) -> Result<Value<'a>> {
    let mut deserializer = stry!(Deserializer::from_slice(s));
    deserializer.to_value()
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    #[cfg_attr(feature = "inline", inline(always))]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match stry!(self.next()) {
            b'n' => {
                stry!(self.parse_null_());
                visitor.visit_unit()
            }
            b't' => visitor.visit_bool(stry!(self.parse_true_())),
            b'f' => visitor.visit_bool(stry!(self.parse_false_())),
            b'-' => match stry!(self.parse_number_(true)) {
                Number::F64(n) => visitor.visit_f64(n),
                Number::I64(n) => visitor.visit_i64(n),
            },
            b'0'...b'9' => match stry!(self.parse_number_(false)) {
                Number::F64(n) => visitor.visit_f64(n),
                Number::I64(n) => visitor.visit_i64(n),
            },
            b'"' => visitor.visit_borrowed_str(stry!(self.parse_str_())),
            b'[' => visitor.visit_seq(CommaSeparated::new(&mut self)),
            b'{' => visitor.visit_map(CommaSeparated::new(&mut self)),
            _c => Err(self.error(ErrorType::UnexpectedCharacter)),
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
            return Err(ErrorType::ExpectedString);
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

    */
    // An absent optional is represented as the JSON `null` and a present
    // optional is represented as just the contained value.
    //
    // As commented in `Serializer` implementation, this is a lossy
    // representation. For example the values `Some(())` and `None` both
    // serialize as just `null`. Unfortunately this is typically what people
    // expect when working with JSON. Other formats are encouraged to behave
    // more intelligently if possible.

    #[cfg_attr(feature = "inline", inline)]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if stry!(self.peek()) == b'n' {
            self.skip();
            stry!(self.parse_null_());
            visitor.visit_unit()
        } else {
            visitor.visit_some(self)
        }
    }

    /*
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
            Err(ErrorType::ExpectedArray(self.idx(), self.c() as char))
        }
    }

     */

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.

    #[cfg_attr(feature = "inline", inline)]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let r = self.deserialize_seq(visitor);
        // tuples have a known length damn you serde ...
        self.skip();
        r
    }

    forward_to_deserialize_any! {
        seq  bool i8 i16 i32 i64 u8 u16 u32 u64 string str unit
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
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        CommaSeparated { first: true, de }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    #[cfg_attr(feature = "inline", inline)]
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
                c => return Err(ErrorType::ExpectedArrayComma(self.de.idx(), c as char)),
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
                self.de.skip();
                return Ok(None);
            }
            b',' if !self.first => stry!(self.de.next()),
            b => {
                if self.first {
                    self.first = false;
                    b
                } else {
                    return Err(self.de.error(ErrorType::ExpectedArrayComma));
                }
            }
        };
        match peek {
            b']' => Err(self.de.error(ErrorType::ExpectedArrayComma)),
            _ => Ok(Some(stry!(seed.deserialize(&mut *self.de)))),
        }
    }
    #[cfg_attr(feature = "inline", inline)]
    fn size_hint(&self) -> Option<usize> {
        Some(self.de.count_elements())
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a> MapAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    #[cfg_attr(feature = "inline", inline)]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        /*
        // Check if there are no more entries.
        if stry!(self.de.peek()) == b'}' {
            self.de.skip();
            return Ok(None)
        };

        let r = if self.first {
            self.first = false;
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            let c = stry!(self.de.next());
            if c != b',' {
                return Err(ErrorType::ExpectedMapComma(self.de.idx(), c as char));
            }
            seed.deserialize(&mut *self.de).map(Some)
        };

        let c = stry!(self.de.next());
        if c != b':' {
            return Err(ErrorType::ExpectedMapColon(self.de.idx(), c as char));
        }
        r
         */

        let peek = match stry!(self.de.peek()) {
            b'}' => {
                self.de.skip();
                return Ok(None);
            }
            b',' if !self.first => {
                self.de.skip();
                stry!(self.de.peek())
            }
            b => {
                if self.first {
                    self.first = false;
                    b
                } else {
                    return Err(self.de.error(ErrorType::ExpectedArrayComma));
                }
            }
        };

        match peek {
            b'"' => seed.deserialize(&mut *self.de).map(Some),
            b'}' => Err(self.de.error(ErrorType::ExpectedArrayComma)), //Err(self.de.peek_error(ErrorCode::TrailingComma)),
            _ => Err(self.de.error(ErrorType::ExpectedString)), // TODO: Err(self.de.peek_error(ErrorCode::KeyMustBeAString)),
        }
    }

    #[cfg_attr(feature = "inline", inline)]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let c = stry!(self.de.next());
        if c != b':' {
            return Err(self.de.error(ErrorType::ExpectedMapColon));
        }
        seed.deserialize(&mut *self.de)
    }

    #[cfg_attr(feature = "inline", inline)]
    fn size_hint(&self) -> Option<usize> {
        Some(self.de.count_elements())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde::Deserialize;
    use serde_json::{self, json};

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
        dbg!(&simd.counts);
        assert_eq!(simd.counts[1], 1);
    }

    #[test]
    fn count3() {
        let mut d = String::from("[1,2]");
        let mut d = unsafe { d.as_bytes_mut() };
        let simd = Deserializer::from_slice(&mut d).expect("");
        dbg!(&simd.counts);
        assert_eq!(simd.counts[1], 2);
    }

    #[test]
    fn count4() {
        let mut d = String::from(" [ 1 , [ 3 ] , 2 ]");
        let mut d = unsafe { d.as_bytes_mut() };
        let simd = Deserializer::from_slice(&mut d).expect("");
        dbg!(&simd.counts);
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
                Value::Array(vec![
                    Value::Number(Number::F64(23.0)),
                    Value::String("snot")
                ]),
                Value::String("bad"),
                Value::String("ger")
            ]))
        );

        let v_serde: serde_json::Value = serde_json::from_slice(d).expect("");
        let v_simd: serde_json::Value = from_slice(&mut d).expect("");
        assert_eq!(v_simd, v_serde);
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
        h.insert("snot", Value::String("badger"));
        assert_eq!(to_value(&mut d1), Ok(Value::Map(h)));
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
        h.insert("snot", Value::String("badger"));
        h.insert("badger", Value::String("snot"));
        assert_eq!(to_value(&mut d1), Ok(Value::Map(h)));
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
            Just(serde_json::Value::Null),
            any::<bool>().prop_map(serde_json::Value::Bool),
            //(-1.0e306f64..1.0e306f64).prop_map(|f| json!(f)), The float parsing of simd and serde are too different
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
