use super::{Object, Value};
use crate::cow::Cow;
use crate::OwnedValue;
use crate::StaticNode;
use std::iter::FromIterator;

impl<'a> From<OwnedValue> for Value<'a> {
    #[inline]
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

impl<'v> From<StaticNode> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(s: StaticNode) -> Self {
        Self::Static(s)
    }
}
/********* str_ **********/
impl<'v> From<&'v str> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(s: &'v str) -> Self {
        Value::String(Cow::from(s))
    }
}

#[cfg(feature = "beef")]
impl<'v> From<std::borrow::Cow<'v, str>> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(c: std::borrow::Cow<'v, str>) -> Self {
        Value::String(c.into())
    }
}

#[cfg(not(feature = "beef"))]
impl<'v> From<std::borrow::Cow<'v, str>> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(c: std::borrow::Cow<'v, str>) -> Self {
        Value::String(c)
    }
}

#[cfg(feature = "beef")]
impl<'v> From<beef::lean::Cow<'v, str>> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(c: beef::lean::Cow<'v, str>) -> Self {
        Self::String(c.into())
    }
}

impl<'v> From<String> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(s: String) -> Self {
        Value::String(s.into())
    }
}

/********* atoms **********/
impl<'v> From<bool> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(b: bool) -> Self {
        Value::Static(StaticNode::Bool(b))
    }
}
impl<'v> From<()> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(_b: ()) -> Self {
        Value::Static(StaticNode::Null)
    }
}

/********* i_ **********/
impl<'v> From<i8> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(i: i8) -> Self {
        Value::Static(StaticNode::I64(i64::from(i)))
    }
}

impl<'v> From<i16> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(i: i16) -> Self {
        Value::Static(StaticNode::I64(i64::from(i)))
    }
}

impl<'v> From<i32> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(i: i32) -> Self {
        Value::Static(StaticNode::I64(i64::from(i)))
    }
}

impl<'v> From<i64> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(i: i64) -> Self {
        Value::Static(StaticNode::I64(i))
    }
}

#[cfg(feature = "128bit")]
impl<'v> From<i128> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(i: i128) -> Self {
        Value::Static(StaticNode::I128(i))
    }
}

/********* u_ **********/
impl<'v> From<u8> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(i: u8) -> Self {
        Self::Static(StaticNode::U64(u64::from(i)))
    }
}

impl<'v> From<u16> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(i: u16) -> Self {
        Self::Static(StaticNode::U64(u64::from(i)))
    }
}

impl<'v> From<u32> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(i: u32) -> Self {
        Self::Static(StaticNode::U64(u64::from(i)))
    }
}

impl<'v> From<u64> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(i: u64) -> Self {
        Value::Static(StaticNode::U64(i))
    }
}

#[cfg(feature = "128bit")]
impl<'v> From<u128> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(i: u128) -> Self {
        Value::Static(StaticNode::U128(i))
    }
}

impl<'v> From<usize> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(i: usize) -> Self {
        Self::Static(StaticNode::U64(i as u64))
    }
}

/********* f_ **********/
impl<'v> From<f32> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(f: f32) -> Self {
        Value::Static(StaticNode::F64(f64::from(f)))
    }
}

impl<'v> From<f64> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(f: f64) -> Self {
        Value::Static(StaticNode::F64(f))
    }
}

impl<'v, S> From<Vec<S>> for Value<'v>
where
    Value<'v>: From<S>,
{
    #[inline]
    #[must_use]
    fn from(v: Vec<S>) -> Self {
        v.into_iter().collect()
    }
}

impl<'v, V: Into<Value<'v>>> FromIterator<V> for Value<'v> {
    #[inline]
    #[must_use]
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        Value::Array(iter.into_iter().map(Into::into).collect())
    }
}

impl<'v, K: Into<Cow<'v, str>>, V: Into<Value<'v>>> FromIterator<(K, V)> for Value<'v> {
    #[inline]
    #[must_use]
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Value::Object(Box::new(
            iter.into_iter()
                .map(|(k, v)| (Into::into(k), Into::into(v)))
                .collect(),
        ))
    }
}

impl<'v> From<Object<'v>> for Value<'v> {
    #[inline]
    #[must_use]
    fn from(v: Object<'v>) -> Self {
        Self::Object(Box::new(v))
    }
}
