use super::Value;
use crate::StaticNode;
use crate::cow::Cow;

impl<'value> From<StaticNode> for Value<'value> {
    fn from(s: StaticNode) -> Self {
        Self::Static(s)
    }
}

impl<'value> From<i8> for Value<'value> {
    fn from(v: i8) -> Self {
        Self::Static(StaticNode::I64(i64::from(v)))
    }
}

impl<'value> From<i16> for Value<'value> {
    fn from(v: i16) -> Self {
        Self::Static(StaticNode::I64(i64::from(v)))
    }
}

impl<'value> From<i32> for Value<'value> {
    fn from(v: i32) -> Self {
        Self::Static(StaticNode::I64(i64::from(v)))
    }
}

impl<'value> From<i64> for Value<'value> {
    fn from(v: i64) -> Self {
        Self::Static(StaticNode::I64(v))
    }
}

impl<'value> From<u8> for Value<'value> {
    fn from(v: u8) -> Self {
        Self::Static(StaticNode::U64(u64::from(v)))
    }
}

impl<'value> From<u16> for Value<'value> {
    fn from(v: u16) -> Self {
        Self::Static(StaticNode::U64(u64::from(v)))
    }
}

impl<'value> From<u32> for Value<'value> {
    fn from(v: u32) -> Self {
        Self::Static(StaticNode::U64(u64::from(v)))
    }
}

impl<'value> From<u64> for Value<'value> {
    fn from(v: u64) -> Self {
        Self::Static(StaticNode::U64(v))
    }
}

impl<'value> From<f32> for Value<'value> {
    fn from(v: f32) -> Self {
        Self::Static(StaticNode::from(f64::from(v)))
    }
}

impl<'value> From<f64> for Value<'value> {
    fn from(v: f64) -> Self {
        Self::Static(StaticNode::from(v))
    }
}

impl<'value> From<bool> for Value<'value> {
    fn from(v: bool) -> Self {
        Self::Static(StaticNode::Bool(v))
    }
}

impl<'value> From<()> for Value<'value> {
    fn from(_: ()) -> Self {
        Self::Static(StaticNode::Null)
    }
}

impl<'value> From<String> for Value<'value> {
    fn from(v: String) -> Self {
        Self::String(Cow::from(v))
    }
}

impl<'value> From<&'value str> for Value<'value> {
    fn from(v: &'value str) -> Self {
        Self::String(Cow::from(v))
    }
}

impl<'value> From<Cow<'value, str>> for Value<'value> {
    fn from(v: Cow<'value, str>) -> Self {
        Self::String(v)
    }
}

impl<'value, T> From<Vec<T>> for Value<'value>
where
    Value<'value>: From<T>,
{
    fn from(v: Vec<T>) -> Self {
        Self::Array(Box::new(v.into_iter().map(Value::from).collect()))
    }
}

impl<'value, K, S> From<indexmap::IndexMap<K, Value<'value>, S>> for Value<'value>
where
    K: Into<Cow<'value, str>>,
    S: std::hash::BuildHasher,
{
    fn from(v: indexmap::IndexMap<K, Value<'value>, S>) -> Self {
        Self::Object(Box::new(v.into_iter().map(|(k, v)| (k.into(), v)).collect()))
    }
}

impl<'value, V: Into<Value<'value>>> FromIterator<V> for Value<'value> {
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        Self::Array(Box::new(iter.into_iter().map(Into::into).collect()))
    }
}

impl<'value> std::iter::FromIterator<(Cow<'value, str>, Value<'value>)> for Value<'value> {
    fn from_iter<T: IntoIterator<Item = (Cow<'value, str>, Value<'value>)>>(iter: T) -> Self {
        Self::Object(Box::new(iter.into_iter().collect()))
    }
}

#[cfg(feature = "128bit")]
impl<'value> From<i128> for Value<'value> {
    fn from(i: i128) -> Self {
        Self::Static(StaticNode::I128(i))
    }
}

#[cfg(feature = "128bit")]
impl<'value> From<u128> for Value<'value> {
    fn from(i: u128) -> Self {
        Self::Static(StaticNode::U128(i))
    }
}
