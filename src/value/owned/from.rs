use super::Value;
use crate::BorrowedValue;
use std::iter::FromIterator;

impl From<crate::BorrowedValue<'_>> for Value {
    fn from(b: BorrowedValue<'_>) -> Self {
        match b {
            BorrowedValue::Null => Value::Null,
            BorrowedValue::Bool(b) => Value::Bool(b),
            BorrowedValue::Number(n) => Value::Number(n),
            BorrowedValue::String(s) => Value::from(s.to_string()),

            BorrowedValue::SmallString(s) => Value::from(s.to_string()),

            BorrowedValue::Array(a) => {
                Value::Array(a.into_iter().map(|v| v.into()).collect::<Vec<Value>>())
            }
            BorrowedValue::Object(m) => Value::Object(
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
        Value::String(s.to_owned())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&String> for Value {
    fn from(s: &String) -> Self {
        Value::String(s.to_owned())
    }
}

/********* atoms **********/

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<()> for Value {
    fn from(_b: ()) -> Self {
        Value::Null
    }
}

/********* i_ **********/
impl From<i8> for Value {
    fn from(i: i8) -> Self {
        Value::Number(i.into())
    }
}

impl From<i16> for Value {
    fn from(i: i16) -> Self {
        Value::Number(i.into())
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Number(i.into())
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Number(i.into())
    }
}

/********* u_ **********/
impl From<u8> for Value {
    fn from(i: u8) -> Self {
        Value::Number(i.into())
    }
}

impl From<u16> for Value {
    fn from(i: u16) -> Self {
        Value::Number(i.into())
    }
}

impl From<u32> for Value {
    fn from(i: u32) -> Self {
        Value::Number(i.into())
    }
}

impl From<u64> for Value {
    fn from(i: u64) -> Self {
        Value::Number(i.into())
    }
}

/********* f_ **********/
impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::Number(f.into())
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Number(f.into())
    }
}

impl<S> From<Vec<S>> for Value
where
    Value: From<S>,
{
    fn from(v: Vec<S>) -> Self {
        Value::Array(v.into_iter().map(Value::from).collect())
    }
}

impl<V: Into<Value>> FromIterator<V> for Value {
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        Value::Array(iter.into_iter().map(Into::into).collect())
    }
}

impl<K: Into<String>, V: Into<Value>> FromIterator<(K, V)> for Value {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Value::Object(
            iter.into_iter()
                .map(|(k, v)| (Into::into(k), Into::into(v)))
                .collect(),
        )
    }
}
