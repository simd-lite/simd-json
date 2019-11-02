use super::{Object, Value};
use crate::stage2::StaticTape;
use crate::OwnedValue;
use std::borrow::Cow;
use std::iter::FromIterator;

impl<'a> From<OwnedValue> for Value<'a> {
    #[inline]
    fn from(b: OwnedValue) -> Self {
        match b {
            OwnedValue::Null => Value::Static(StaticTape::Null),
            OwnedValue::Bool(b) => Value::Static(StaticTape::Bool(b)),
            OwnedValue::F64(f) => Value::Static(StaticTape::F64(f)),
            OwnedValue::I64(i) => Value::Static(StaticTape::I64(i)),
            OwnedValue::U64(i) => Value::Static(StaticTape::U64(i)),
            OwnedValue::String(s) => Value::from(s.to_string()),
            OwnedValue::Array(a) => a.into_iter().collect(),
            OwnedValue::Object(m) => m.into_iter().collect(),
        }
    }
}

/********* str_ **********/
impl<'v> From<&'v str> for Value<'v> {
    #[inline]
    fn from(s: &'v str) -> Self {
        Value::String(Cow::Borrowed(s))
    }
}

impl<'v> From<Cow<'v, str>> for Value<'v> {
    #[inline]
    fn from(c: Cow<'v, str>) -> Self {
        Value::String(c)
    }
}

impl<'v> From<String> for Value<'v> {
    #[inline]
    fn from(s: String) -> Self {
        Value::String(s.into())
    }
}

/********* atoms **********/
impl<'v> From<bool> for Value<'v> {
    #[inline]
    fn from(b: bool) -> Self {
        Value::Static(StaticTape::Bool(b))
    }
}
impl<'v> From<()> for Value<'v> {
    #[inline]
    fn from(_b: ()) -> Self {
        Value::Static(StaticTape::Null)
    }
}

/********* i_ **********/
impl<'v> From<i8> for Value<'v> {
    #[inline]
    fn from(i: i8) -> Self {
        Value::Static(StaticTape::I64(i64::from(i)))
    }
}

impl<'v> From<i16> for Value<'v> {
    #[inline]
    fn from(i: i16) -> Self {
        Value::Static(StaticTape::I64(i64::from(i)))
    }
}

impl<'v> From<i32> for Value<'v> {
    #[inline]
    fn from(i: i32) -> Self {
        Value::Static(StaticTape::I64(i64::from(i)))
    }
}

impl<'v> From<i64> for Value<'v> {
    #[inline]
    fn from(i: i64) -> Self {
        Value::Static(StaticTape::I64(i))
    }
}

/********* u_ **********/
impl<'v> From<u8> for Value<'v> {
    #[inline]
    fn from(i: u8) -> Self {
        Self::Static(StaticTape::U64(u64::from(i)))
    }
}

impl<'v> From<u16> for Value<'v> {
    #[inline]
    fn from(i: u16) -> Self {
        Self::Static(StaticTape::U64(u64::from(i)))
    }
}

impl<'v> From<u32> for Value<'v> {
    #[inline]
    fn from(i: u32) -> Self {
        Self::Static(StaticTape::U64(u64::from(i)))
    }
}

impl<'v> From<u64> for Value<'v> {
    #[inline]
    fn from(i: u64) -> Self {
        Value::Static(StaticTape::U64(i))
    }
}

impl<'v> From<usize> for Value<'v> {
    #[inline]
    fn from(i: usize) -> Self {
        Self::Static(StaticTape::U64(i as u64))
    }
}

/********* f_ **********/
impl<'v> From<f32> for Value<'v> {
    #[inline]
    fn from(f: f32) -> Self {
        Value::Static(StaticTape::F64(f64::from(f)))
    }
}

impl<'v> From<f64> for Value<'v> {
    #[inline]
    fn from(f: f64) -> Self {
        Value::Static(StaticTape::F64(f))
    }
}

impl<'v, S> From<Vec<S>> for Value<'v>
where
    Value<'v>: From<S>,
{
    #[inline]
    fn from(v: Vec<S>) -> Self {
        v.into_iter().collect()
    }
}

impl<'v, V: Into<Value<'v>>> FromIterator<V> for Value<'v> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        Value::Array(iter.into_iter().map(Into::into).collect())
    }
}

impl<'v, K: Into<Cow<'v, str>>, V: Into<Value<'v>>> FromIterator<(K, V)> for Value<'v> {
    #[inline]
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
    fn from(v: Object<'v>) -> Self {
        Self::Object(Box::new(v))
    }
}
