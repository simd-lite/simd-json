use super::{Object, Value};
use crate::cow::Cow;
use crate::OwnedValue;
use crate::StaticNode;

impl<'value> From<OwnedValue> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(b: OwnedValue) -> Self {
        match b {
            OwnedValue::Static(s) => Value::from(s),
            OwnedValue::String(s) => Value::from(s),
            OwnedValue::Array(a) => a.into_iter().collect(),
            OwnedValue::Object(m) => m.into_iter().collect(),
        }
    }
}

impl<'value> From<StaticNode> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(s: StaticNode) -> Self {
        Self::Static(s)
    }
}

impl<'value, T> From<Option<T>> for Value<'value>
where
    Value<'value>: From<T>,
{
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(s: Option<T>) -> Self {
        s.map_or(Value::Static(StaticNode::Null), Value::from)
    }
}
/********* str_ **********/
impl<'value> From<&'value str> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(s: &'value str) -> Self {
        Value::String(Cow::from(s))
    }
}

#[cfg(feature = "beef")]
impl<'value> From<std::borrow::Cow<'value, str>> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(c: std::borrow::Cow<'value, str>) -> Self {
        Value::String(c.into())
    }
}

#[cfg(not(feature = "beef"))]
impl<'value> From<std::borrow::Cow<'value, str>> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(c: std::borrow::Cow<'value, str>) -> Self {
        Value::String(c)
    }
}

#[cfg(feature = "beef")]
impl<'value> From<beef::lean::Cow<'value, str>> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(c: beef::lean::Cow<'value, str>) -> Self {
        Self::String(c)
    }
}

impl<'value> From<String> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(s: String) -> Self {
        Value::String(s.into())
    }
}

/********* atoms **********/
impl<'value> From<bool> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(b: bool) -> Self {
        Value::Static(StaticNode::Bool(b))
    }
}
impl<'value> From<()> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(_b: ()) -> Self {
        Value::Static(StaticNode::Null)
    }
}

/********* i_ **********/
impl<'value> From<i8> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(i: i8) -> Self {
        Value::Static(StaticNode::I64(i64::from(i)))
    }
}

impl<'value> From<i16> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(i: i16) -> Self {
        Value::Static(StaticNode::I64(i64::from(i)))
    }
}

impl<'value> From<i32> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(i: i32) -> Self {
        Value::Static(StaticNode::I64(i64::from(i)))
    }
}

impl<'value> From<i64> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(i: i64) -> Self {
        Value::Static(StaticNode::I64(i))
    }
}

#[cfg(feature = "128bit")]
impl<'value> From<i128> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(i: i128) -> Self {
        Value::Static(StaticNode::I128(i))
    }
}

/********* u_ **********/
impl<'value> From<u8> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(i: u8) -> Self {
        Self::Static(StaticNode::U64(u64::from(i)))
    }
}

impl<'value> From<u16> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(i: u16) -> Self {
        Self::Static(StaticNode::U64(u64::from(i)))
    }
}

impl<'value> From<u32> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(i: u32) -> Self {
        Self::Static(StaticNode::U64(u64::from(i)))
    }
}

impl<'value> From<u64> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(i: u64) -> Self {
        Value::Static(StaticNode::U64(i))
    }
}

#[cfg(feature = "128bit")]
impl<'value> From<u128> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(i: u128) -> Self {
        Value::Static(StaticNode::U128(i))
    }
}

impl<'value> From<usize> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(i: usize) -> Self {
        Self::Static(StaticNode::U64(i as u64))
    }
}

/********* f_ **********/
impl<'value> From<f32> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(f: f32) -> Self {
        Value::Static(StaticNode::F64(f64::from(f)))
    }
}

impl<'value> From<f64> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(f: f64) -> Self {
        Value::Static(StaticNode::F64(f))
    }
}

impl<'value, S> From<Vec<S>> for Value<'value>
where
    Value<'value>: From<S>,
{
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: Vec<S>) -> Self {
        v.into_iter().collect()
    }
}

impl<'value, V: Into<Value<'value>>> FromIterator<V> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        Value::Array(iter.into_iter().map(Into::into).collect())
    }
}

impl<'value, K: Into<Cow<'value, str>>, V: Into<Value<'value>>> FromIterator<(K, V)>
    for Value<'value>
{
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Value::Object(Box::new(
            iter.into_iter()
                .map(|(k, v)| (Into::into(k), Into::into(v)))
                .collect(),
        ))
    }
}

impl<'value> From<Object<'value>> for Value<'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: Object<'value>) -> Self {
        Self::Object(Box::new(v))
    }
}
