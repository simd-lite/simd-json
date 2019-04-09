mod de;
use crate::value::Value;
use crate::{stry, Deserializer, Error, ErrorType, Result};
use serde::Deserialize;
use serde_ext::Serialize;
use std::fmt;

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

#[cfg(not(feature = "no-borrow"))]
#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn to_value<'a, T>(value: T) -> Result<Value<'a>>
where
    T: Serialize,
{
    value.serialize(crate::value::Serializer::default())
}

#[cfg(feature = "no-borrow")]
#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn to_value<T>(value: T) -> Result<Value>
where
    T: Serialize,
{
    value.serialize(crate::value::Serializer::default())
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
