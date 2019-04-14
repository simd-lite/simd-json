use super::{MaybeBorrowedString, Value};
use crate::numberparse::Number;
use crate::OwnedValue;

impl<'a> From<&'a str> for MaybeBorrowedString<'a> {
    #[inline]
    fn from(v: &'a str) -> Self {
        MaybeBorrowedString::B(v)
    }
}

impl<'a> From<String> for MaybeBorrowedString<'a> {
    #[inline]
    fn from(v: String) -> Self {
        MaybeBorrowedString::O(v)
    }
}

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
impl<'a> From<&'a str> for Value<'a> {
    #[inline]
    fn from(s: &'a str) -> Self {
        Value::String(MaybeBorrowedString::B(s))
    }
}

impl<'a> From<String> for Value<'a> {
    #[inline]
    fn from(s: String) -> Self {
        Value::String(MaybeBorrowedString::O(s))
    }
}

impl<'a> From<MaybeBorrowedString<'a>> for Value<'a> {
    #[inline]
    fn from(s: MaybeBorrowedString<'a>) -> Self {
        Value::String(s)
    }
}

/********* atoms **********/
impl<'a> From<bool> for Value<'a> {
    #[inline]
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

/********* i_ **********/
impl<'a> From<i8> for Value<'a> {
    #[inline]
    fn from(i: i8) -> Self {
        Value::I64(i as i64)
    }
}

impl<'a> From<i16> for Value<'a> {
    #[inline]
    fn from(i: i16) -> Self {
        Value::I64(i as i64)
    }
}

impl<'a> From<i32> for Value<'a> {
    #[inline]
    fn from(i: i32) -> Self {
        Value::I64(i as i64)
    }
}

impl<'a> From<i64> for Value<'a> {
    #[inline]
    fn from(i: i64) -> Self {
        Value::I64(i as i64)
    }
}

impl<'a> From<&i8> for Value<'a> {
    #[inline]
    fn from(i: &i8) -> Self {
        Value::I64(*i as i64)
    }
}

impl<'a> From<&i16> for Value<'a> {
    #[inline]
    fn from(i: &i16) -> Self {
        Value::I64(*i as i64)
    }
}

impl<'a> From<&i32> for Value<'a> {
    #[inline]
    fn from(i: &i32) -> Self {
        Value::I64(*i as i64)
    }
}

impl<'a> From<&i64> for Value<'a> {
    #[inline]
    fn from(i: &i64) -> Self {
        Value::I64(*i as i64)
    }
}

/********* u_ **********/
impl<'a> From<u8> for Value<'a> {
    #[inline]
    fn from(i: u8) -> Self {
        Value::I64(i as i64)
    }
}

impl<'a> From<u16> for Value<'a> {
    #[inline]
    fn from(i: u16) -> Self {
        Value::I64(i as i64)
    }
}

impl<'a> From<u32> for Value<'a> {
    #[inline]
    fn from(i: u32) -> Self {
        Value::I64(i as i64)
    }
}

impl<'a> From<u64> for Value<'a> {
    #[inline]
    fn from(i: u64) -> Self {
        Value::I64(i as i64)
    }
}

impl<'a> From<&u8> for Value<'a> {
    #[inline]
    fn from(i: &u8) -> Self {
        Value::I64(*i as i64)
    }
}

impl<'a> From<&u16> for Value<'a> {
    #[inline]
    fn from(i: &u16) -> Self {
        Value::I64(*i as i64)
    }
}

impl<'a> From<&u32> for Value<'a> {
    #[inline]
    fn from(i: &u32) -> Self {
        Value::I64(*i as i64)
    }
}

impl<'a> From<&u64> for Value<'a> {
    #[inline]
    fn from(i: &u64) -> Self {
        Value::I64(*i as i64)
    }
}

/********* f_ **********/
impl<'a> From<f32> for Value<'a> {
    #[inline]
    fn from(f: f32) -> Self {
        Value::F64(f as f64)
    }
}

impl<'a> From<f64> for Value<'a> {
    #[inline]
    fn from(f: f64) -> Self {
        Value::F64(f as f64)
    }
}

impl<'a> From<&f32> for Value<'a> {
    #[inline]
    fn from(f: &f32) -> Self {
        Value::F64(*f as f64)
    }
}

impl<'a> From<&f64> for Value<'a> {
    #[inline]
    fn from(f: &f64) -> Self {
        Value::F64(*f as f64)
    }
}
