mod de;
mod se;

use crate::value::borrowed::ordered::Value;
use crate::Result;
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
