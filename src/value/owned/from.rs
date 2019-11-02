use super::{Object, Value};
use crate::numberparse::Number;
use crate::stage2::StaticTape;
use crate::BorrowedValue;
use std::borrow::Cow;
use std::iter::FromIterator;

impl From<Number> for Value {
    #[inline]
    fn from(n: Number) -> Self {
        match n {
            Number::F64(n) => Self::F64(n),
            Number::I64(n) => Self::I64(n),
            Number::U64(n) => Self::U64(n),
        }
    }
}

impl From<crate::BorrowedValue<'_>> for Value {
    #[inline]
    fn from(b: BorrowedValue<'_>) -> Self {
        match b {
            BorrowedValue::Static(StaticTape::Null) => Self::Null,
            BorrowedValue::Static(StaticTape::Bool(b)) => Self::Bool(b),
            BorrowedValue::Static(StaticTape::F64(f)) => Self::F64(f),
            BorrowedValue::Static(StaticTape::I64(i)) => Self::I64(i),
            BorrowedValue::Static(StaticTape::U64(i)) => Self::U64(i),
            BorrowedValue::String(s) => Self::from(s.to_string()),
            BorrowedValue::Array(a) => a.into_iter().collect(),
            BorrowedValue::Object(m) => m.into_iter().collect(),
        }
    }
}

/********* str_ **********/

impl From<&str> for Value {
    #[inline]
    fn from(s: &str) -> Self {
        Self::String(s.to_owned())
    }
}

impl<'v> From<Cow<'v, str>> for Value {
    #[inline]
    fn from(c: Cow<'v, str>) -> Self {
        Self::String(c.to_string())
    }
}

impl From<String> for Value {
    #[inline]
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&String> for Value {
    #[inline]
    fn from(s: &String) -> Self {
        Self::String(s.to_owned())
    }
}

/********* atoms **********/

impl From<bool> for Value {
    #[inline]
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<()> for Value {
    #[inline]
    fn from(_b: ()) -> Self {
        Self::Null
    }
}

/********* i_ **********/
impl From<i8> for Value {
    #[inline]
    fn from(i: i8) -> Self {
        Self::I64(i64::from(i))
    }
}

impl From<i16> for Value {
    #[inline]
    fn from(i: i16) -> Self {
        Self::I64(i64::from(i))
    }
}

impl From<i32> for Value {
    #[inline]
    fn from(i: i32) -> Self {
        Self::I64(i64::from(i))
    }
}

impl From<i64> for Value {
    #[inline]
    fn from(i: i64) -> Self {
        Self::I64(i)
    }
}

/********* u_ **********/
impl From<u8> for Value {
    #[inline]
    fn from(i: u8) -> Self {
        Self::U64(u64::from(i))
    }
}

impl From<u16> for Value {
    #[inline]
    fn from(i: u16) -> Self {
        Self::U64(u64::from(i))
    }
}

impl From<u32> for Value {
    #[inline]
    fn from(i: u32) -> Self {
        Self::U64(u64::from(i))
    }
}

impl From<u64> for Value {
    #[inline]
    fn from(i: u64) -> Self {
        #[allow(clippy::cast_possible_wrap)]
        Self::U64(i)
    }
}

impl From<usize> for Value {
    #[inline]
    fn from(i: usize) -> Self {
        #[allow(clippy::cast_possible_wrap)]
        Self::U64(i as u64)
    }
}

/********* f_ **********/
impl From<f32> for Value {
    #[inline]
    fn from(f: f32) -> Self {
        Self::F64(f64::from(f))
    }
}

impl From<f64> for Value {
    #[inline]
    fn from(f: f64) -> Self {
        Self::F64(f)
    }
}

impl<S> From<Vec<S>> for Value
where
    Value: From<S>,
{
    #[inline]
    fn from(v: Vec<S>) -> Self {
        v.into_iter().collect()
    }
}

impl<V: Into<Value>> FromIterator<V> for Value {
    #[inline]
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        Self::Array(iter.into_iter().map(Into::into).collect())
    }
}

impl<K: ToString, V: Into<Value>> FromIterator<(K, V)> for Value {
    #[inline]
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self::Object(Box::new(
            iter.into_iter()
                .map(|(k, v)| (k.to_string(), Into::into(v)))
                .collect(),
        ))
    }
}

impl From<Object> for Value {
    #[inline]
    fn from(v: Object) -> Self {
        Self::Object(Box::new(v))
    }
}
