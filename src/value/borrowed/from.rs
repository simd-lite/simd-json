use super::Value;
use crate::numberparse::Number;
use crate::OwnedValue;
use std::borrow::Cow;
use std::iter::FromIterator;

impl<'a> From<Number> for Value<'a> {
    #[inline]
    fn from(n: Number) -> Self {
        match n {
            Number::F64(n) => Value::F64(n),
            Number::I64(n) => Value::I64(n),
        }
    }
}

impl<'a> From<OwnedValue> for Value<'a> {
    fn from(b: OwnedValue) -> Self {
        match b {
            OwnedValue::Null => Value::Null,
            OwnedValue::Bool(b) => Value::Bool(b),
            OwnedValue::F64(f) => Value::F64(f),
            OwnedValue::I64(i) => Value::I64(i),
            OwnedValue::String(s) => Value::from(s.to_string()),
            OwnedValue::Array(a) => {
                Value::Array(a.into_iter().map(|v| v.into()).collect::<Vec<Value>>())
            }
            OwnedValue::Object(m) => {
                Value::Object(m.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
            }
        }
    }
}

/********* str_ **********/
impl<'v> From<&'v str> for Value<'v> {
    #[inline]
    fn from(s: &'v str) -> Self {
        Value::String(s.into())
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
        Value::Bool(b)
    }
}
impl<'v> From<()> for Value<'v> {
    fn from(_b: ()) -> Self {
        Value::Null
    }
}

/********* i_ **********/
impl<'v> From<i8> for Value<'v> {
    #[inline]
    fn from(i: i8) -> Self {
        Value::I64(i64::from(i))
    }
}

impl<'v> From<i16> for Value<'v> {
    #[inline]
    fn from(i: i16) -> Self {
        Value::I64(i64::from(i))
    }
}

impl<'v> From<i32> for Value<'v> {
    #[inline]
    fn from(i: i32) -> Self {
        Value::I64(i64::from(i))
    }
}

impl<'v> From<i64> for Value<'v> {
    #[inline]
    fn from(i: i64) -> Self {
        Value::I64(i)
    }
}

impl<'v> From<&i8> for Value<'v> {
    #[inline]
    fn from(i: &i8) -> Self {
        Value::I64(i64::from(*i))
    }
}

impl<'v> From<&i16> for Value<'v> {
    #[inline]
    fn from(i: &i16) -> Self {
        Value::I64(i64::from(*i))
    }
}

impl<'v> From<&i32> for Value<'v> {
    #[inline]
    fn from(i: &i32) -> Self {
        Value::I64(i64::from(*i))
    }
}

impl<'v> From<&i64> for Value<'v> {
    #[inline]
    fn from(i: &i64) -> Self {
        Value::I64(*i)
    }
}

/********* u_ **********/
impl<'v> From<u8> for Value<'v> {
    #[inline]
    fn from(i: u8) -> Self {
        Value::I64(i64::from(i))
    }
}

impl<'v> From<u16> for Value<'v> {
    #[inline]
    fn from(i: u16) -> Self {
        Value::I64(i64::from(i))
    }
}

impl<'v> From<u32> for Value<'v> {
    #[inline]
    fn from(i: u32) -> Self {
        Value::I64(i64::from(i))
    }
}

impl<'v> From<u64> for Value<'v> {
    #[inline]
    fn from(i: u64) -> Self {
        Value::I64(i as i64)
    }
}

impl<'v> From<&u8> for Value<'v> {
    #[inline]
    fn from(i: &u8) -> Self {
        Value::I64(i64::from(*i))
    }
}

impl<'v> From<&u16> for Value<'v> {
    #[inline]
    fn from(i: &u16) -> Self {
        Value::I64(i64::from(*i))
    }
}

impl<'v> From<&u32> for Value<'v> {
    #[inline]
    fn from(i: &u32) -> Self {
        Value::I64(i64::from(*i))
    }
}

impl<'v> From<&u64> for Value<'v> {
    #[inline]
    fn from(i: &u64) -> Self {
        Value::I64(*i as i64)
    }
}

/********* f_ **********/
impl<'v> From<f32> for Value<'v> {
    #[inline]
    fn from(f: f32) -> Self {
        Value::F64(f64::from(f))
    }
}

impl<'v> From<f64> for Value<'v> {
    #[inline]
    fn from(f: f64) -> Self {
        Value::F64(f)
    }
}

impl<'v> From<&f32> for Value<'v> {
    #[inline]
    fn from(f: &f32) -> Self {
        Value::F64(f64::from(*f))
    }
}

impl<'v> From<&f64> for Value<'v> {
    #[inline]
    fn from(f: &f64) -> Self {
        Value::F64(*f)
    }
}

impl<'v, S> From<Vec<S>> for Value<'v>
where
    Value<'v>: From<S>,
{
    fn from(v: Vec<S>) -> Self {
        Value::Array(v.into_iter().map(Value::from).collect())
    }
}

impl<'v, V: Into<Value<'v>>> FromIterator<V> for Value<'v> {
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        Value::Array(iter.into_iter().map(Into::into).collect())
    }
}

impl<'v, K: Into<Cow<'v, str>>, V: Into<Value<'v>>> FromIterator<(K, V)> for Value<'v> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Value::Object(
            iter.into_iter()
                .map(|(k, v)| (Into::into(k), Into::into(v)))
                .collect(),
        )
    }
}
