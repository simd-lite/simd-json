use crate::numberparse::Number;
use crate::scalemap::ScaleMap;
use std::fmt;

pub type Map = ScaleMap<String, Value>;

pub enum MaybeBorrowedString {
    O(String)
}

impl fmt::Display for MaybeBorrowedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MaybeBorrowedString::O(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Debug for MaybeBorrowedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MaybeBorrowedString::O(s) => write!(f, "{:?}", s),
        }
    }
}

impl PartialEq for MaybeBorrowedString {
    fn eq(&self, other: &Self) -> bool {
        let self_s = match self {
            MaybeBorrowedString::O(s) => s.as_str(),
        };
        let other_s = match other {
            MaybeBorrowedString::O(s) => s.as_str(),
        };

        self_s == other_s
    }
}

impl PartialEq<str> for MaybeBorrowedString {
    fn eq(&self, other: &str) -> bool {
        match self {
            MaybeBorrowedString::O(s) => s == &other,
        }
    }
}

impl PartialEq<String> for MaybeBorrowedString {
    fn eq(&self, other: &String) -> bool {
        match self {
            MaybeBorrowedString::O(s) => s == other,
        }
    }
}

impl From<& str> for MaybeBorrowedString {
    fn from(v: & str) -> Self {
        MaybeBorrowedString::O(v.to_owned())
    }
}

impl From<String> for MaybeBorrowedString {
    fn from(v: String) -> Self {
        MaybeBorrowedString::O(v)
    }
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(MaybeBorrowedString),
    Array(Vec<Value>),
    Map(Map),
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(MaybeBorrowedString::O(s.to_owned()))
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(MaybeBorrowedString::O(s))
    }
}


impl From<i8> for Value {
    fn from(i: i8) -> Self {
        Value::Number(Number::I64(i as i64))
    }
}

impl From<i16> for Value {
    fn from(i: i16) -> Self {
        Value::Number(Number::I64(i as i64))
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Number(Number::I64(i as i64))
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Number(Number::I64(i as i64))
    }
}

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::Number(Number::F64(f as f64))
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Number(Number::F64(f as f64))
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

/*
impl<'v, T> From<&[T]> for Value<'v> where Value<'v>: From<T> {
    fn from(s: &[T]) -> Self {
        let v: Vec<Value<'v>> = s.into_iter().map(Value::from).collect();
        Value::Array(v)
    }
}
*/

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl PartialEq<()> for Value {
    fn eq(&self, _other: &()) -> bool {
        if let Value::Null = self {
            true
        } else {
            false
        }
    }
}

impl PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        if let Value::Bool(v) = self {
            v == other
        } else {
            false
        }
    }
}

impl PartialEq<f64> for Value {
    fn eq(&self, other: &f64) -> bool {
        if let Value::Number(Number::F64(v)) = self {
            v == other
        } else {
            false
        }
    }
}


impl PartialEq<f32> for Value {
    fn eq(&self, other: &f32) -> bool {
        if let Value::Number(Number::F64(v)) = self {
            *v == *other as f64
        } else {
            false
        }
    }
}
