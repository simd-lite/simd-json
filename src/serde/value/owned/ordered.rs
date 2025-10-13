mod de;
mod se;

use crate::value::owned::ordered::Value;
use crate::Result;
use serde_ext::ser::Serialize;
use serde_ext::de::DeserializeOwned;

/// Tries to convert a struct that implements serde's serialize into
/// an ordered `OwnedValue`
///
/// # Errors
///
/// Will return `Err` if value fails to be turned into an owned ordered value
pub fn to_value<T>(value: T) -> Result<Value>
where
    T: Serialize,
{
    value.serialize(se::Serializer::default())
}

/// Tries to convert a `OwnedValue` into a struct that implements
/// serde's Deserialize interface
///
/// # Errors
///
/// Will return `Err` if `value` fails to be deserialized
pub fn from_value<T>(value: Value) -> Result<T>
where
    T: DeserializeOwned,
{
    T::deserialize(value)
}

/// Tries to convert a `&OwnedValue` into a struct that implements
/// serde's Deserialize interface
///
/// # Errors
///
/// Will return `Err` if `value` fails to be deserialized
pub fn from_refvalue<T>(value: &Value) -> Result<T>
where
    T: DeserializeOwned,
{
    T::deserialize(value)
}
