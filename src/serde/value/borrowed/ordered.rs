mod de;
mod se;

use crate::value::borrowed::ordered::Value;
use crate::Result;
use serde_ext::de::Deserialize;
use serde_ext::ser::Serialize;

/// Tries to convert a struct that implements serde's serialize into
/// an ordered `BorrowedValue`
///
/// # Errors
///
/// Will return `Err` if value fails to be turned into a borrowed ordered value
pub fn to_value<'se, T>(value: T) -> Result<Value<'se>>
where
    T: Serialize,
{
    value.serialize(se::Serializer::default())
}

/// Tries to convert a `BorrowedValue` into a struct that implements
/// serde's Deserialize interface
///
/// # Errors
///
/// Will return `Err` if `value` can not be deserialized
pub fn from_value<'de, T>(value: Value<'de>) -> Result<T>
where
    T: Deserialize<'de>,
{
    T::deserialize(value)
}

/// Tries to convert a `&BorrowedValue` into a struct that implements
/// serde's Deserialize interface
///
/// # Errors
///
/// Will return `Err` if `value` fails to be deserialized
pub fn from_refvalue<'de, T>(value: &'de Value<'de>) -> Result<T>
where
    T: Deserialize<'de>,
{
    T::deserialize(value)
}
