use super::Value;
use crate::StaticNode;
use crate::{borrowed, cow::Cow};
use std::borrow::Cow as StdCow;

impl<'borrow, 'tape, 'value> From<borrowed::Value<'value>> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: borrowed::Value<'value>) -> Self {
        Value::Value(StdCow::Owned(v))
    }
}

impl<'borrow, 'tape, 'value> From<StaticNode> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: StaticNode) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

impl<'borrow, 'tape, 'value, T> From<Option<T>> for Value<'borrow, 'tape, 'value>
where
    borrowed::Value<'value>: From<Option<T>>,
{
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: Option<T>) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}
/********* str_ **********/
impl<'borrow, 'tape, 'value> From<&'value str> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: &'value str) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

#[cfg(feature = "beef")]
impl<'borrow, 'tape, 'value> From<std::borrow::Cow<'value, str>> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: std::borrow::Cow<'value, str>) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

#[cfg(not(feature = "beef"))]
impl<'borrow, 'tape, 'value> From<std::borrow::Cow<'value, str>> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: std::borrow::Cow<'value, str>) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

#[cfg(feature = "beef")]
impl<'borrow, 'tape, 'value> From<beef::lean::Cow<'value, str>> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: beef::lean::Cow<'value, str>) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

impl<'borrow, 'tape, 'value> From<String> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: String) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

/********* atoms **********/
impl<'borrow, 'tape, 'value> From<bool> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: bool) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}
impl<'borrow, 'tape, 'value> From<()> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: ()) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

/********* i_ **********/
impl<'borrow, 'tape, 'value> From<i8> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: i8) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

impl<'borrow, 'tape, 'value> From<i16> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: i16) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

impl<'borrow, 'tape, 'value> From<i32> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: i32) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

impl<'borrow, 'tape, 'value> From<i64> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: i64) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

#[cfg(feature = "128bit")]
impl<'borrow, 'tape, 'value> From<i128> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: i128) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

/********* u_ **********/
impl<'borrow, 'tape, 'value> From<u8> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: u8) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

impl<'borrow, 'tape, 'value> From<u16> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: u16) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

impl<'borrow, 'tape, 'value> From<u32> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: u32) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

impl<'borrow, 'tape, 'value> From<u64> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: u64) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

#[cfg(feature = "128bit")]
impl<'borrow, 'tape, 'value> From<u128> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: u128) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

impl<'borrow, 'tape, 'value> From<usize> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: usize) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

/********* f_ **********/
impl<'borrow, 'tape, 'value> From<f32> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: f32) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

impl<'borrow, 'tape, 'value> From<f64> for Value<'borrow, 'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: f64) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

impl<'borrow, 'tape, 'value, S> From<Vec<S>> for Value<'borrow, 'tape, 'value>
where
    borrowed::Value<'value>: From<Vec<S>>,
{
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: Vec<S>) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}

impl<'borrow, 'tape, 'value, V: Into<borrowed::Value<'value>>> FromIterator<V>
    for Value<'borrow, 'tape, 'value>
{
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from_iter<I: IntoIterator<Item = V>>(v: I) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from_iter(v)))
    }
}

impl<'borrow, 'tape, 'value, K: Into<Cow<'value, str>>, V: Into<borrowed::Value<'value>>>
    FromIterator<(K, V)> for Value<'borrow, 'tape, 'value>
{
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from_iter<I: IntoIterator<Item = (K, V)>>(v: I) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from_iter(v)))
    }
}

impl<'borrow, 'tape, 'value> From<crate::borrowed::Object<'value>>
    for Value<'borrow, 'tape, 'value>
{
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: crate::borrowed::Object<'value>) -> Self {
        Value::Value(StdCow::Owned(borrowed::Value::from(v)))
    }
}
