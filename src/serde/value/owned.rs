mod de;
mod se;

use crate::value::owned::Value;
use crate::Result;
use serde_ext::de::DeserializeOwned;
use serde_ext::ser::Serialize;

pub fn to_value<T>(value: T) -> Result<Value>
where
    T: Serialize,
{
    value.serialize(se::Serializer::default())
}

pub fn from_value<T>(value: Value) -> Result<T>
where
    T: DeserializeOwned,
{
    T::deserialize(value)
}
