use crate::numberparse::Number;
use crate::scalemap::ScaleMap;
use std::fmt;
use std::borrow::Borrow;
use std::ops::Deref;

pub type Map<'a> = ScaleMap<&'a str, Value<'a>>;

#[derive(Clone)]
pub enum MaybeBorrowedString<'a> {
    B(&'a str),
    O(String)
}

impl<'a> Borrow<str> for MaybeBorrowedString<'a> {
    fn borrow(&self) -> &str {
        match self {
            MaybeBorrowedString::B(s) => s,
            MaybeBorrowedString::O(s) => &s
        }
    }
}

impl<'a> Deref for MaybeBorrowedString<'a> {
    type Target = str;
    fn deref(&self) -> &str {
        match self {
            MaybeBorrowedString::B(s) => s,
            MaybeBorrowedString::O(s) => &s
        }
    }
}

impl<'a> fmt::Display for MaybeBorrowedString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MaybeBorrowedString::B(s) => write!(f, "{}", s),
            MaybeBorrowedString::O(s) => write!(f, "{}", s),
        }
    }
}

impl<'a> fmt::Debug for MaybeBorrowedString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MaybeBorrowedString::B(s) => write!(f, "{:?}", s),
            MaybeBorrowedString::O(s) => write!(f, "{:?}", s),
        }
    }
}

impl<'a> PartialEq for MaybeBorrowedString<'a> {
    fn eq(&self, other: &Self) -> bool {
        let self_s = match self {
            MaybeBorrowedString::B(s) => s,
            MaybeBorrowedString::O(s) => s.as_str(),
        };
        let other_s = match other {
            MaybeBorrowedString::B(s) => s,
            MaybeBorrowedString::O(s) => s.as_str(),
        };

        self_s == other_s
    }
}

impl<'a> PartialEq<str> for MaybeBorrowedString<'a> {
    fn eq(&self, other: &str) -> bool {
        match self {
            MaybeBorrowedString::B(s) => s == &other,
            MaybeBorrowedString::O(s) => s == &other,
        }
    }
}

impl<'a> PartialEq<String> for MaybeBorrowedString<'a> {
    fn eq(&self, other: &String) -> bool {
        match self {
            MaybeBorrowedString::B(s) => s == other,
            MaybeBorrowedString::O(s) => s == other,
        }
    }
}

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

#[derive(Debug, PartialEq, Clone)]
pub enum Value<'a> {
    Null,
    Bool(bool),
    Number(Number),
    String(MaybeBorrowedString<'a>),
    Array(Vec<Value<'a>>),
    Object(Map<'a>),
}

impl<'a> Value<'a> {
    pub fn get(&self, k: &str) -> Option<&Value>{
        match self {
            Value::Object(m) => m.get(k),
            _ => None
        }
    }

    pub fn get_mut(&'a mut self, k: &'a str) -> Option<&mut Value>{
        match self {
            Value::Object(m) => m.get_mut(&k),
            _ => None
        }
    }
    pub fn is_object(&self) -> bool {
        match self {
            Value::Object(_m) => true,
            _ => false
        }
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

/*
impl<'v, T> From<&[T]> for Value<'v> where Value<'v>: From<T> {
    fn from(s: &[T]) -> Self {
        let v: Vec<Value<'v>> = s.into_iter().map(Value::from).collect();
        Value::Array(v)
    }
}
*/

impl<'a> Default for Value<'a> {
    fn default() -> Self {
        Value::Null
    }
}

impl<'a> PartialEq<()> for Value<'a> {
    fn eq(&self, _other: &()) -> bool {
        if let Value::Null = self {
            true
        } else {
            false
        }
    }
}

impl<'a> PartialEq<bool> for Value<'a> {
    fn eq(&self, other: &bool) -> bool {
        if let Value::Bool(v) = self {
            v == other
        } else {
            false
        }
    }
}

impl<'a> PartialEq<f64> for Value<'a> {
    fn eq(&self, other: &f64) -> bool {
        if let Value::Number(Number::F64(v)) = self {
            v == other
        } else {
            false
        }
    }
}


impl<'a> PartialEq<f32> for Value<'a> {
    fn eq(&self, other: &f32) -> bool {
        if let Value::Number(Number::F64(v)) = self {
            *v == *other as f64
        } else {
            false
        }
    }
}
