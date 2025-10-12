mod de;
mod se;

use crate::value::owned::ordered::Value;
use crate::Result;
use serde_ext::ser::Serialize;

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
