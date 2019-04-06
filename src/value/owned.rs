mod cmp;
mod from;
mod mbs;

use crate::halfbrown::HashMap;
pub use mbs::*;
use std::fmt;
use std::ops::Index;

pub type Map = HashMap<String, Value>;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    String(MaybeBorrowedString),
    Array(Vec<Value>),
    Object(Map),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::I64(n) => write!(f, "{}", n),
            Value::F64(n) => write!(f, "{}", n),
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
