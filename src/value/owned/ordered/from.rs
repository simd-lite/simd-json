use super::Value;
use crate::StaticNode;
use crate::cow::Cow;

impl From<StaticNode> for Value {
    fn from(s: StaticNode) -> Self {
        Self::Static(s)
    }
}

impl From<i8> for Value {
    fn from(v: i8) -> Self {
        Self::Static(StaticNode::I64(i64::from(v)))
    }
}

impl From<i16> for Value {
    fn from(v: i16) -> Self {
        Self::Static(StaticNode::I64(i64::from(v)))
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Self::Static(StaticNode::I64(i64::from(v)))
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Self::Static(StaticNode::I64(v))
    }
}

impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Self::Static(StaticNode::U64(u64::from(v)))
    }
}

impl From<u16> for Value {
    fn from(v: u16) -> Self {
        Self::Static(StaticNode::U64(u64::from(v)))
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Self::Static(StaticNode::U64(u64::from(v)))
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Self::Static(StaticNode::U64(v))
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Self::Static(StaticNode::from(f64::from(v)))
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Static(StaticNode::from(v))
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Static(StaticNode::Bool(v))
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Self::Static(StaticNode::Null)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

impl From<Cow<'_, str>> for Value {
    fn from(v: Cow<'_, str>) -> Self {
        Self::String(v.into_owned())
    }
}

impl<T> From<Vec<T>> for Value
where
    Value: From<T>,
{
    fn from(v: Vec<T>) -> Self {
        Self::Array(Box::new(v.into_iter().map(Value::from).collect()))
    }
}

impl<K, S> From<indexmap::IndexMap<K, Value, S>> for Value
where
    K: Into<String>,
    S: std::hash::BuildHasher,
{
    fn from(v: indexmap::IndexMap<K, Value, S>) -> Self {
        Self::Object(Box::new(v.into_iter().map(|(k, v)| (k.into(), v)).collect()))
    }
}

impl<V: Into<Value>> FromIterator<V> for Value {
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        Self::Array(Box::new(iter.into_iter().map(Into::into).collect()))
    }
}

impl<K, V> FromIterator<(K, V)> for Value
where
    K: Into<String>,
    V: Into<Value>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self::Object(Box::new(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect()
        ))
    }
}

impl From<usize> for Value {
    fn from(v: usize) -> Self {
        Self::Static(StaticNode::U64(v as u64))
    }
}

impl<T> From<Option<T>> for Value
where
    Value: From<T>,
{
    fn from(v: Option<T>) -> Self {
        match v {
            Some(val) => Value::from(val),
            None => Value::Static(StaticNode::Null),
        }
    }
}

impl<'a> From<Value> for crate::value::borrowed::ordered::Value<'a> {
    fn from(v: Value) -> Self {
        match v {
            Value::Static(s) => crate::value::borrowed::ordered::Value::Static(s),
            Value::String(s) => crate::value::borrowed::ordered::Value::String(Cow::from(s)),
            Value::Array(arr) => crate::value::borrowed::ordered::Value::Array(Box::new(
                arr.into_iter().map(Into::into).collect()
            )),
            Value::Object(obj) => crate::value::borrowed::ordered::Value::Object(Box::new(
                obj.into_iter()
                    .map(|(k, v)| (Cow::from(k), v.into()))
                    .collect()
            )),
        }
    }
}

#[cfg(feature = "128bit")]
impl From<i128> for Value {
    fn from(i: i128) -> Self {
        Self::Static(StaticNode::I128(i))
    }
}

#[cfg(feature = "128bit")]
impl From<u128> for Value {
    fn from(i: u128) -> Self {
        Self::Static(StaticNode::U128(i))
    }
}
