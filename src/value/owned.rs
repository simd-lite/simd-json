mod cmp;
mod from;
mod mbs;

use crate::numberparse::Number;
use crate::scalemap::ScaleMap;
pub use mbs::*;
use std::fmt;
use std::ops::Index;

pub type Map = ScaleMap<String, Value>;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(MaybeBorrowedString),
    Array(Vec<Value>),
    Object(Map),
}

impl fmt::Display for Value {
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

impl Index<&str> for Value {
    type Output = Value;
    fn index(&self, index: &str) -> &Value {
        static NULL: Value = Value::Null;
        self.get(index).unwrap_or(&NULL)
    }
}

impl Value {
    pub fn get(&self, k: &str) -> Option<&Value> {
        match self {
            Value::Object(m) => m.get(k),
            _ => None,
        }
    }
    pub fn get_mut(&mut self, k: &str) -> Option<&mut Value> {
        match self {
            Value::Object(m) => m.get_mut(k),
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

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}
