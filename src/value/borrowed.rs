mod cmp;
mod from;
mod mbs;

use crate::halfbrown::HashMap;
use crate::numberparse::Number;
use std::fmt;
use std::ops::Index;

pub type Map<'a> = HashMap<&'a str, Value<'a>>;
pub use mbs::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Value<'a> {
    Null,
    Bool(bool),
    Number(Number),
    String(MaybeBorrowedString<'a>),
    Array(Vec<Value<'a>>),
    Object(Map<'a>),
}

impl<'a> fmt::Display for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Array(a) => write!(f, "{:?}", a),
            Value::Object(o) => write!(f, "{:?}", o),
        }
    }
}

impl<'a> Index<&str> for Value<'a> {
    type Output = Value<'a>;
    fn index(&self, index: &str) -> &Value<'a> {
        static NULL: Value = Value::Null;
        self.get(index).unwrap_or(&NULL)
    }
}

impl<'a> Value<'a> {
    pub fn get(&self, k: &str) -> Option<&Value<'a>> {
        match self {
            Value::Object(m) => m.get(k),
            _ => None,
        }
    }

    pub fn get_mut(&'a mut self, k: &'a str) -> Option<&mut Value> {
        match self {
            Value::Object(m) => m.get_mut(&k),
            _ => None,
        }
    }
    pub fn is_object(&self) -> bool {
        match self {
            Value::Object(_m) => true,
            _ => false,
        }
    }
}

impl<'a> Default for Value<'a> {
    fn default() -> Self {
        Value::Null
    }
}
