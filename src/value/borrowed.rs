///A dom object that references the raw input data to avoid allocations
// it tradecs having lifetimes for a gain in performance.
mod cmp;
mod from;

use halfbrown::HashMap;
use crate::{stry, unlikely, Deserializer, ErrorType, Result};
use std::borrow::Cow;
use std::fmt;
use std::ops::Index;
pub type Map<'a> = HashMap<Cow<'a, str>, Value<'a>>;

/// Parses a slice of butes into a Value dom. This function will
/// rewrite the slice to de-escape strings.
/// As we reference parts of the input slice the resulting dom
/// has the dame lifetime as the slice it was created from.
pub fn to_value<'a>(s: &'a mut [u8]) -> Result<Value<'a>> {
    let mut deserializer = stry!(Deserializer::from_slice(s));
    deserializer.to_value_borrowed_root()
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value<'a> {
    Null,
    Bool(bool),
    F64(f64),
    I64(i64),
    String(Cow<'a, str>),
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

impl<'de> Deserializer<'de> {
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    pub fn to_value_borrowed_root(&mut self) -> Result<Value<'de>> {
        #[cfg(feature = "paranoid")]
        {
            if self.idx + 1 > self.structural_indexes.len() {
                return Err(self.error(ErrorType::UnexpectedEnd));
            }
        }
        match self.next_() {
            b'"' => {
                let next = unsafe { *self.structural_indexes.get_unchecked(self.idx + 1) as usize };
                if next - self.iidx < 32 {
                    return self.parse_short_str_().map(Value::from);
                }
                self.parse_str_().map(Value::from)
            }
            b'-' => self.parse_number(true).map(Value::from),
            b'0'...b'9' => self.parse_number(false).map(Value::from),
            b'n' => {
                stry!(self.parse_null());
                Ok(Value::Null)
            }
            b't' => self.parse_true().map(Value::Bool),
            b'f' => self.parse_false().map(Value::Bool),
            b'[' => self.parse_array_borrowed(),
            b'{' => self.parse_map_borrowed(),
            _c => Err(self.error(ErrorType::UnexpectedCharacter)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn to_value_borrowed(&mut self) -> Result<Value<'de>> {
        #[cfg(feature = "paranoid")]
        {
            if self.idx + 1 > self.structural_indexes.len() {
                return Err(self.error(ErrorType::UnexpectedEnd));
            }
        }
        match self.next_() {
            b'"' => {
                // We can only have entered this by being in an object so we know there is
                // something following as we made sure during checking for sizes.
                let next = unsafe { *self.structural_indexes.get_unchecked(self.idx + 1) as usize };
                if next - self.iidx < 32 {
                    return self.parse_short_str_().map(Value::from);
                }
                self.parse_str_().map(Value::from)
            }
            b'-' => self.parse_number_(true).map(Value::from),
            b'0'...b'9' => self.parse_number_(false).map(Value::from),
            b'n' => {
                stry!(self.parse_null_());
                Ok(Value::Null)
            }
            b't' => self.parse_true_().map(Value::Bool),
            b'f' => self.parse_false_().map(Value::Bool),
            b'[' => self.parse_array_borrowed(),
            b'{' => self.parse_map_borrowed(),
            _c => Err(self.error(ErrorType::UnexpectedCharacter)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_array_borrowed(&mut self) -> Result<Value<'de>> {
        // We short cut for empty arrays
        if unlikely!(self.peek_() == b']') {
            self.skip();
            return Ok(Value::Array(Vec::new()));
        }

        let mut res = Vec::with_capacity(self.count_elements());

        // Since we checked if it's empty we know that we at least have one
        // element so we eat this

        res.push(stry!(self.to_value_borrowed()));
        loop {
            // We now exect one of two things, a comma with a next
            // element or a closing bracket
            match self.peek_() {
                b']' => break,
                b',' => self.skip(),
                _c => return Err(self.error(ErrorType::ExpectedArrayComma)),
            }
            res.push(stry!(self.to_value_borrowed()));
        }
        self.skip();
        // We found a closing bracket and ended our loop, we skip it
        Ok(Value::Array(res))
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_map_borrowed(&mut self) -> Result<Value<'de>> {
        // We short cut for empty arrays

        if unlikely!(self.peek_() == b'}') {
            self.skip();
            return Ok(Value::Object(Map::new()));
        }

        let mut res = Map::with_capacity(self.count_elements());

        // Since we checked if it's empty we know that we at least have one
        // element so we eat this

        if self.next_() != b'"' {
            return Err(self.error(ErrorType::ExpectedString));
        }

        let key = stry!(self.parse_short_str_());

        match self.next_() {
            b':' => (),
            _c => return Err(self.error(ErrorType::ExpectedMapColon)),
        }
        res.insert_nocheck(key.into(), stry!(self.to_value_borrowed()));
        loop {
            // We now exect one of two things, a comma with a next
            // element or a closing bracket
            match self.peek_() {
                b'}' => break,
                b',' => self.skip(),
                _c => return Err(self.error(ErrorType::ExpectedArrayComma)),
            }
            if unlikely!(self.next_() != b'"') {
                return Err(self.error(ErrorType::ExpectedString));
            }

            let key = stry!(self.parse_short_str_());

            if unlikely!(self.next_() != b':') {
                return Err(self.error(ErrorType::ExpectedMapColon));
            }
            res.insert_nocheck(key.into(), stry!(self.to_value_borrowed()));
        }
        // We found a closing bracket and ended our loop, we skip it
        self.skip();
        Ok(Value::Object(res))
    }
}
