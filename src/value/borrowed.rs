mod cmp;
mod de;
mod from;
mod mbs;
mod se;

use crate::halfbrown::HashMap;
use std::fmt;
use std::ops::Index;

pub type Map<'a> = HashMap<&'a str, Value<'a>>;
pub use mbs::*;
pub use se::Serializer;

#[derive(Debug, PartialEq, Clone)]
pub enum Value<'a> {
    Null,
    Bool(bool),
    //Number(Number),
    F64(f64),
    I64(i64),
    String(MaybeBorrowedString<'a>),
    Array(Vec<Value<'a>>),
    Object(Map<'a>),
}

impl<'a> fmt::Display for Value<'a> {
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

    pub fn get_mut(&mut self, k: &str) -> Option<&mut Value<'a>> {
        match self {
            Value::Object(m) => m.get_mut(k),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Value::Null => true,
            _ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            Value::Bool(_) => true,
            _ => false,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn is_i64(&self) -> bool {
        match self {
            Value::I64(_i) => true,
            _ => false,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::I64(i) => Some(*i),
            _ => None,
        }
    }

    pub fn is_u64(&self) -> bool {
        match self {
            Value::I64(i) if i >= &0 => true,
            _ => false,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::I64(i) if i >= &0 => Some(*i as u64),
            _ => None,
        }
    }

    pub fn is_f64(&self) -> bool {
        match self {
            Value::F64(_i) => true,
            _ => false,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::F64(i) => Some(*i),
            _ => None,
        }
    }

    pub fn cast_f64(&self) -> Option<f64> {
        match self {
            Value::F64(i) => Some(*i),
            Value::I64(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Value::String(_m) => true,
            _ => false,
        }
    }
    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.to_string()),
            _ => None,
        }
    }
    pub fn is_array(&self) -> bool {
        match self {
            Value::Array(_m) => true,
            _ => false,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }
    pub fn is_object(&self) -> bool {
        match self {
            Value::Object(_m) => true,
            _ => false,
        }
    }

    pub fn as_object(&self) -> Option<&Map> {
        match self {
            Value::Object(m) => Some(m),
            _ => None,
        }
    }
}

impl<'a> Default for Value<'a> {
    fn default() -> Self {
        Value::Null
    }
}
