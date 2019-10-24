use super::{Object, Value};
use crate::numberparse::Number;
use crate::BorrowedValue;
use std::borrow::Cow;
use std::iter::FromIterator;

impl From<Number> for Value {
    #[inline]
    fn from(n: Number) -> Self {
        match n {
            Number::F64(n) => Self::F64(n),
            Number::I64(n) => Self::I64(n),
        }
    }
}

impl From<crate::BorrowedValue<'_>> for Value {
    fn from(b: BorrowedValue<'_>) -> Self {
        match b {
            BorrowedValue::Null => Self::Null,
            BorrowedValue::Bool(b) => Self::Bool(b),
            BorrowedValue::F64(f) => Self::F64(f),
            BorrowedValue::I64(i) => Self::I64(i),
            BorrowedValue::String(s) => Self::from(s.to_string()),
            BorrowedValue::Array(a) => {
                Self::Array(a.into_iter().map(|v| v.into()).collect::<Vec<Self>>())
            }
            BorrowedValue::Object(m) => Self::Object(
                m.into_iter()
                    .map(|(k, v)| (k.to_string(), v.into()))
                    .collect(),
            ),
        }
    }
}

/********* str_ **********/

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::String(s.to_owned())
    }
}

impl<'v> From<Cow<'v, str>> for Value {
    fn from(c: Cow<'v, str>) -> Self {
        Self::String(c.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&String> for Value {
    fn from(s: &String) -> Self {
        Self::String(s.to_owned())
    }
}

/********* atoms **********/

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<()> for Value {
    fn from(_b: ()) -> Self {
        Self::Null
    }
}

/********* i_ **********/
impl From<i8> for Value {
    fn from(i: i8) -> Self {
        Self::I64(i64::from(i))
    }
}

impl From<i16> for Value {
    fn from(i: i16) -> Self {
        Self::I64(i64::from(i))
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Self::I64(i64::from(i))
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Self::I64(i)
    }
}

/********* u_ **********/
impl From<u8> for Value {
    fn from(i: u8) -> Self {
        Self::I64(i64::from(i))
    }
}

impl From<u16> for Value {
    fn from(i: u16) -> Self {
        Self::I64(i64::from(i))
    }
}

impl From<u32> for Value {
    fn from(i: u32) -> Self {
        Self::I64(i64::from(i))
    }
}

impl From<u64> for Value {
    fn from(i: u64) -> Self {
        #[allow(clippy::cast_possible_wrap)]
        Self::I64(i as i64)
    }
}

impl From<usize> for Value {
    fn from(i: usize) -> Self {
        #[allow(clippy::cast_possible_wrap)]
        Self::I64(i as i64)
    }
}

/********* f_ **********/
impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Self::F64(f64::from(f))
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Self::F64(f)
    }
}

impl<S> From<Vec<S>> for Value
where
    Value: From<S>,
{
    fn from(v: Vec<S>) -> Self {
        Self::Array(v.into_iter().map(Self::from).collect())
    }
}

impl<V: Into<Value>> FromIterator<V> for Value {
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        Self::Array(iter.into_iter().map(Into::into).collect())
    }
}

impl<K: Into<String>, V: Into<Value>> FromIterator<(K, V)> for Value {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self::Object(
            iter.into_iter()
                .map(|(k, v)| (Into::into(k), Into::into(v)))
                .collect(),
        )
    }
}

impl From<Object> for Value {
    fn from(v: Object) -> Self {
        Value::Object(v)
    }
}
