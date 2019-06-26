///A dom object that references the raw input data to avoid allocations
// it tradecs having lifetimes for a gain in performance.
mod cmp;
mod from;
mod serialize;

use crate::value::{ValueTrait, ValueType};
use crate::{stry, unlikely, Deserializer, ErrorType, Result};
use halfbrown::HashMap;
use std::borrow::Cow;
use std::fmt;
use std::ops::Index;

pub type Map<'v> = HashMap<Cow<'v, str>, Value<'v>>;

/// Parses a slice of butes into a Value dom. This function will
/// rewrite the slice to de-escape strings.
/// As we reference parts of the input slice the resulting dom
/// has the dame lifetime as the slice it was created from.
pub fn to_value<'v>(s: &'v mut [u8]) -> Result<Value<'v>> {
    let mut deserializer = stry!(Deserializer::from_slice(s));
    deserializer.parse_value_borrowed()
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value<'v> {
    Null,
    Bool(bool),
    F64(f64),
    I64(i64),
    String(Cow<'v, str>),
    Array(Vec<Value<'v>>),
    Object(Map<'v>),
}

impl<'v> ValueTrait for Value<'v> {
    type Map = Map<'v>;
    type Array = Vec<Value<'v>>;

    fn get(&self, k: &str) -> Option<&Value<'v>> {
        match self {
            Value::Object(m) => m.get(k),
            _ => None,
        }
    }

    fn get_mut(&mut self, k: &str) -> Option<&mut Value<'v>> {
        match self {
            Value::Object(m) => m.get_mut(k),
            _ => None,
        }
    }

    fn kind(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::Bool(_) => ValueType::Bool,
            Value::F64(_) => ValueType::F64,
            Value::I64(_) => ValueType::I64,
            Value::String(_) => ValueType::String,
            Value::Array(_) => ValueType::Array,
            Value::Object(_) => ValueType::Object,
        }
    }

    fn is_null(&self) -> bool {
        match self {
            Value::Null => true,
            _ => false,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            Value::I64(i) => Some(*i),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match self {
            Value::I64(i) if *i >= 0 => Some(*i as u64),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            Value::F64(i) => Some(*i),
            _ => None,
        }
    }

    fn cast_f64(&self) -> Option<f64> {
        match self {
            Value::F64(i) => Some(*i),
            Value::I64(i) => Some(*i as f64),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.to_string()),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&Vec<Value<'v>>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    fn as_array_mut(&mut self) -> Option<&mut Vec<Value<'v>>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    fn as_object(&self) -> Option<&Self::Map> {
        match self {
            Value::Object(m) => Some(m),
            _ => None,
        }
    }

    fn as_object_mut(&mut self) -> Option<&mut Self::Map> {
        match self {
            Value::Object(m) => Some(m),
            _ => None,
        }
    }
}

impl<'v> fmt::Display for Value<'v> {
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

impl<'v> Index<&str> for Value<'v> {
    type Output = Value<'v>;
    fn index(&self, index: &str) -> &Value<'v> {
        static NULL: Value = Value::Null;
        self.get(index).unwrap_or(&NULL)
    }
}

impl<'v> Default for Value<'v> {
    fn default() -> Self {
        Value::Null
    }
}

impl<'de> Deserializer<'de> {
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_value_borrowed(&mut self) -> Result<Value<'de>> {
        match self.next_() {
            b'"' => self.parse_str_().map(Value::from),
            b'-' => self.parse_number_(true).map(Value::from),
            b'0'..=b'9' => self.parse_number_(false).map(Value::from),
            b'n' => Ok(Value::Null),
            b't' => Ok(Value::Bool(true)),
            b'f' => Ok(Value::Bool(false)),
            b'[' => self.parse_array_borrowed(),
            b'{' => self.parse_map_borrowed(),
            _c => Err(self.error(ErrorType::UnexpectedCharacter)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_array_borrowed(&mut self) -> Result<Value<'de>> {
        let es = self.count_elements();
        if unlikely!(es == 0) {
            self.skip();
            return Ok(Value::Array(Vec::new()));
        }
        let mut res = Vec::with_capacity(es);

        for _i in 0..es {
            res.push(stry!(self.parse_value_borrowed()));
            self.skip();
        }
        Ok(Value::Array(res))
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_map_borrowed(&mut self) -> Result<Value<'de>> {
        // We short cut for empty arrays
        let es = self.count_elements();

        if unlikely!(es == 0) {
            self.skip();
            return Ok(Value::Object(Map::new()));
        }

        let mut res = Map::with_capacity(es);

        // Since we checked if it's empty we know that we at least have one
        // element so we eat this

        for _ in 0..es {
            self.skip();
            let key = stry!(self.parse_str_());
            // We have to call parse short str twice since parse_short_str
            // does not move the cursor forward
            self.skip();
            res.insert_nocheck(key.into(), stry!(self.parse_value_borrowed()));
            self.skip();
        }
        Ok(Value::Object(res))
    }
}
