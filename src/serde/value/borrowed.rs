mod de;
mod se;

use crate::{BorrowedValue, Result};
use serde_ext::de::Deserialize;

/* TODO:
use serde_ext::ser::Serialize;
pub fn to_value<'a, T>(value: T) -> Result<Value<'a>>
where
T: Serialize,
{
value.serialize(super::se::Serializer::default())
}
*/

/// Tries to convert a `BorrowedValue` into a struct that implements
/// serde's Deserialize interface
///
/// # Errors
///
/// Will return `Err` if `value` can not be deserialized

pub fn from_value<'de, T>(value: BorrowedValue<'de>) -> Result<T>
where
    T: Deserialize<'de>,
{
    T::deserialize(value)
}
