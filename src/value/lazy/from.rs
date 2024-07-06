use super::Value;
use crate::StaticNode;
use crate::{borrowed, cow::Cow};

impl<'tape, 'value> From<StaticNode> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: StaticNode) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

impl<'tape, 'value, T> From<Option<T>> for Value<'tape, 'value>
where
    borrowed::Value<'value>: From<Option<T>>,
{
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: Option<T>) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}
/********* str_ **********/
impl<'tape, 'value> From<&'value str> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: &'value str) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

#[cfg(feature = "beef")]
impl<'tape, 'value> From<std::borrow::Cow<'value, str>> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: std::borrow::Cow<'value, str>) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

#[cfg(not(feature = "beef"))]
impl<'tape, 'value> From<std::borrow::Cow<'value, str>> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: std::borrow::Cow<'value, str>) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

#[cfg(feature = "beef")]
impl<'tape, 'value> From<beef::lean::Cow<'value, str>> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: beef::lean::Cow<'value, str>) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

impl<'tape, 'value> From<String> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: String) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

/********* atoms **********/
impl<'tape, 'value> From<bool> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: bool) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}
impl<'tape, 'value> From<()> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: ()) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

/********* i_ **********/
impl<'tape, 'value> From<i8> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: i8) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

impl<'tape, 'value> From<i16> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: i16) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

impl<'tape, 'value> From<i32> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: i32) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

impl<'tape, 'value> From<i64> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: i64) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

#[cfg(feature = "128bit")]
impl<'tape, 'value> From<i128> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: i128) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

/********* u_ **********/
impl<'tape, 'value> From<u8> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: u8) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

impl<'tape, 'value> From<u16> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: u16) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

impl<'tape, 'value> From<u32> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: u32) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

impl<'tape, 'value> From<u64> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: u64) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

#[cfg(feature = "128bit")]
impl<'tape, 'value> From<u128> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: u128) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

impl<'tape, 'value> From<usize> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: usize) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

/********* f_ **********/
impl<'tape, 'value> From<f32> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: f32) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

impl<'tape, 'value> From<f64> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: f64) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

impl<'tape, 'value, S> From<Vec<S>> for Value<'tape, 'value>
where
    borrowed::Value<'value>: From<Vec<S>>,
{
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: Vec<S>) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}

impl<'tape, 'value, V: Into<borrowed::Value<'value>>> FromIterator<V> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from_iter<I: IntoIterator<Item = V>>(v: I) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from_iter(v)))
    }
}

impl<'tape, 'value, K: Into<Cow<'value, str>>, V: Into<borrowed::Value<'value>>>
    FromIterator<(K, V)> for Value<'tape, 'value>
{
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from_iter<I: IntoIterator<Item = (K, V)>>(v: I) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from_iter(v)))
    }
}

impl<'tape, 'value> From<crate::borrowed::Object<'value>> for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn from(v: crate::borrowed::Object<'value>) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::from(v)))
    }
}
