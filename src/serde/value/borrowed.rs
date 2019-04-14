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

pub fn from_value<'de, T>(value: BorrowedValue<'de>) -> Result<T>
where
    T: Deserialize<'de>,
{
    T::deserialize(value)
}
