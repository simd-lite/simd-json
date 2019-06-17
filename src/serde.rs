/// simdjson-rs integrates with serde, this module holds this integration.
/// note that when parsing to a dom you should use the functions in
/// `to_owned_value` or `to_borrowed_value` as they provide much
/// better performance.
///
/// However if have to use serde for other readons or are psrsing
/// directly to structs this is th4 place to go.
mod de;
mod value;
use crate::{stry, Deserializer, Error, ErrorType, Result};
use serde_ext::Deserialize;
use std::fmt;

pub use self::value::*;

/// parses a byte slice using a serde deserializer.
/// note that the slice will be rewritten in the process.
#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn from_slice<'a, T>(s: &'a mut [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = stry!(Deserializer::from_slice(s));

    T::deserialize(&mut deserializer)
}
/// parses a str  using a serde deserializer.
/// note that the slice will be rewritten in the process and
/// might not remain a valid utf8 string in its entirety.
#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn from_str<'a, T>(s: &'a mut str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = stry!(Deserializer::from_slice(unsafe { s.as_bytes_mut() }));

    T::deserialize(&mut deserializer)
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::generic(ErrorType::Serde(msg.to_string()))
    }
}

impl serde_ext::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::generic(ErrorType::Serde(msg.to_string()))
    }
}

// Functions purely used by serde
impl<'de> Deserializer<'de> {
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

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn peek(&self) -> Result<u8> {
        if let Some(idx) = self.structural_indexes.get(self.idx + 1) {
            unsafe { Ok(*self.input.get_unchecked(*idx as usize)) }
        } else {
            Err(self.error(ErrorType::UnexpectedEnd))
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_signed(&mut self) -> Result<i64> {
        match self.next_() {
            b'-' => match stry!(self.parse_number(true)).as_i64() {
                Some(n) => Ok(n),
                _ => Err(self.error(ErrorType::ExpectedSigned)),
            },
            b'0'...b'9' => match stry!(self.parse_number(false)).as_i64() {
                Some(n) => Ok(n),
                _ => Err(self.error(ErrorType::ExpectedSigned)),
            },
            _ => Err(self.error(ErrorType::ExpectedSigned)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_unsigned(&mut self) -> Result<u64> {
        match self.next_() {
            b'0'...b'9' => match stry!(self.parse_number(false)).as_u64() {
                Some(n) => Ok(n as u64),
                _ => Err(self.error(ErrorType::ExpectedUnsigned)),
            },
            _ => Err(self.error(ErrorType::ExpectedUnsigned)),
        }
    }
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_double(&mut self) -> Result<f64> {
        match self.next_() {
            b'-' => match stry!(self.parse_number(true)).as_f64() {
                Some(n) => Ok(n),
                _ => Err(self.error(ErrorType::ExpectedFloat)),
            },
            b'0'...b'9' => match stry!(self.parse_number(false)).as_f64() {
                Some(n) => Ok(n),
                _ => Err(self.error(ErrorType::ExpectedFloat)),
            },
            _ => Err(self.error(ErrorType::ExpectedFloat)),
        }
    }
}
