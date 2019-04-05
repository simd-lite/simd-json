use super::{MaybeBorrowedString, Value};
use crate::Number;

impl<'a> From<&'a str> for MaybeBorrowedString<'a> {
    fn from(v: &'a str) -> Self {
        MaybeBorrowedString::B(v)
    }
}

impl<'a> From<String> for MaybeBorrowedString<'a> {
    fn from(v: String) -> Self {
        MaybeBorrowedString::O(v)
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(s: &'a str) -> Self {
        Value::String(MaybeBorrowedString::B(s))
    }
}

impl<'a> From<String> for Value<'a> {
    fn from(s: String) -> Self {
        Value::String(MaybeBorrowedString::O(s))
    }
}

impl<'a> From<i8> for Value<'a> {
    fn from(i: i8) -> Self {
        Value::Number(Number::I64(i as i64))
    }
}

impl<'a> From<i16> for Value<'a> {
    fn from(i: i16) -> Self {
        Value::Number(Number::I64(i as i64))
    }
}

impl<'a> From<i32> for Value<'a> {
    fn from(i: i32) -> Self {
        Value::Number(Number::I64(i as i64))
    }
}

impl<'a> From<i64> for Value<'a> {
    fn from(i: i64) -> Self {
        Value::Number(Number::I64(i as i64))
    }
}

impl<'a> From<f32> for Value<'a> {
    fn from(f: f32) -> Self {
        Value::Number(Number::F64(f as f64))
    }
}

impl<'a> From<f64> for Value<'a> {
    fn from(f: f64) -> Self {
        Value::Number(Number::F64(f as f64))
    }
}

impl<'a> From<bool> for Value<'a> {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}
