mod de;
use crate::value::*;
use crate::{stry, Deserializer, Error, ErrorType, Result};
use serde::Deserialize;
use std::fmt;

pub use borrowed::to_value as to_borrowed_value;
pub use owned::to_value as to_owned_value;

#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn from_slice<'a, T>(s: &'a mut [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = stry!(Deserializer::from_slice(s));

    T::deserialize(&mut deserializer)
}

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
