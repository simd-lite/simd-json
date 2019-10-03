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

/// Representation of a JSON object
#[deprecated(since = "0.1.21", note = "Please use Object instead")]
pub type Map<'v> = Object<'v>;
/// Representation of a JSON object
pub type Object<'v> = HashMap<Cow<'v, str>, Value<'v>>;

/// Parses a slice of butes into a Value dom. This function will
/// rewrite the slice to de-escape strings.
/// As we reference parts of the input slice the resulting dom
/// has the dame lifetime as the slice it was created from.
pub fn to_value<'v>(s: &'v mut [u8]) -> Result<Value<'v>> {
    let de = stry!(Deserializer::from_slice(s));
    BorrowDeserializer::from_deserializer(de).parse()
}

/// Borrowed JSON-DOM Value, consider using the `ValueTrait`
/// to access it'scontent
#[derive(Debug, PartialEq, Clone)]
pub enum Value<'v> {
    /// null
    Null,
    /// boolean type
    Bool(bool),
    /// float type
    F64(f64),
    /// integer type
    I64(i64),
    /// string type
    String(Cow<'v, str>),
    /// array type
    Array(Vec<Value<'v>>),
    /// object type
    Object(Object<'v>),
}

impl<'v> ValueTrait for Value<'v> {
    type Object = Object<'v>;
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

    fn get_idx(&self, i: usize) -> Option<&Self> {
        self.as_array().and_then(|a| a.get(i))
    }
    fn get_idx_mut(&mut self, i: usize) -> Option<&mut Self> {
        self.as_array_mut().and_then(|a| a.get_mut(i))
    }

    fn value_type(&self) -> ValueType {
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
        #[allow(clippy::cast_sign_loss)]
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
        #[allow(clippy::cast_precision_loss)]
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

    fn as_str(&self) -> Option<&str> {
        use std::borrow::Borrow;
        match self {
            Value::String(s) => Some(s.borrow()),
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

    fn as_object(&self) -> Option<&<Value<'v> as ValueTrait>::Object> {
        match self {
            Value::Object(m) => Some(m),
            _ => None,
        }
    }

    fn as_object_mut(&mut self) -> Option<&mut <Value<'v> as ValueTrait>::Object> {
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

struct BorrowDeserializer<'de> {
    de: Deserializer<'de>,
}
impl<'de> BorrowDeserializer<'de> {
    pub fn from_deserializer(de: Deserializer<'de>) -> Self {
        Self { de }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    pub fn parse(&mut self) -> Result<Value<'de>> {
        match self.de.next_() {
            b'"' => self.de.parse_str_().map(Value::from),
            b'-' => self.de.parse_number_root(true).map(Value::from),
            b'0'..=b'9' => self.de.parse_number_root(false).map(Value::from),
            b'n' => Ok(Value::Null),
            b't' => Ok(Value::Bool(true)),
            b'f' => Ok(Value::Bool(false)),
            b'[' => self.parse_array(),
            b'{' => self.parse_map(),
            _c => Err(self.de.error(ErrorType::UnexpectedCharacter)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_value(&mut self) -> Result<Value<'de>> {
        match self.de.next_() {
            b'"' => self.de.parse_str_().map(Value::from),
            b'-' => self.de.parse_number_(true).map(Value::from),
            b'0'..=b'9' => self.de.parse_number_(false).map(Value::from),
            b'n' => Ok(Value::Null),
            b't' => Ok(Value::Bool(true)),
            b'f' => Ok(Value::Bool(false)),
            b'[' => self.parse_array(),
            b'{' => self.parse_map(),
            _c => Err(self.de.error(ErrorType::UnexpectedCharacter)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_array(&mut self) -> Result<Value<'de>> {
        let es = self.de.count_elements();
        if unlikely!(es == 0) {
            self.de.skip();
            return Ok(Value::Array(Vec::new()));
        }
        let mut res = Vec::with_capacity(es);

        for _i in 0..es {
            res.push(stry!(self.parse_value()));
            self.de.skip();
        }
        Ok(Value::Array(res))
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_map(&mut self) -> Result<Value<'de>> {
        // We short cut for empty arrays
        let es = self.de.count_elements();

        if unlikely!(es == 0) {
            self.de.skip();
            return Ok(Value::Object(Object::new()));
        }

        let mut res = Object::with_capacity(es);

        // Since we checked if it's empty we know that we at least have one
        // element so we eat this

        for _ in 0..es {
            self.de.skip();
            let key = stry!(self.de.parse_str_());
            // We have to call parse short str twice since parse_short_str
            // does not move the cursor forward
            self.de.skip();
            res.insert_nocheck(key.into(), stry!(self.parse_value()));
            self.de.skip();
        }
        Ok(Value::Object(res))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn conversions_i64() {
        let v = Value::from(i64::max_value());
        assert!(v.is_i128());
        assert!(v.is_u128());
        assert!(v.is_i64());
        assert!(v.is_u64());
        assert!(!v.is_i32());
        assert!(!v.is_u32());
        assert!(!v.is_i16());
        assert!(!v.is_u16());
        assert!(!v.is_i8());
        assert!(!v.is_u8());
        let v = Value::from(i64::min_value());
        assert!(v.is_i128());
        assert!(!v.is_u128());
        assert!(v.is_i64());
        assert!(!v.is_u64());
        assert!(!v.is_i32());
        assert!(!v.is_u32());
        assert!(!v.is_i16());
        assert!(!v.is_u16());
        assert!(!v.is_i8());
        assert!(!v.is_u8());
    }

    #[test]
    fn conversions_i32() {
        let v = Value::from(i32::max_value());
        assert!(v.is_i128());
        assert!(v.is_u128());
        assert!(v.is_i64());
        assert!(v.is_u64());
        assert!(v.is_i32());
        assert!(v.is_u32());
        assert!(!v.is_i16());
        assert!(!v.is_u16());
        assert!(!v.is_i8());
        assert!(!v.is_u8());
        let v = Value::from(i32::min_value());
        assert!(v.is_i128());
        assert!(!v.is_u128());
        assert!(v.is_i64());
        assert!(!v.is_u64());
        assert!(v.is_i32());
        assert!(!v.is_u32());
        assert!(!v.is_i16());
        assert!(!v.is_u16());
        assert!(!v.is_i8());
        assert!(!v.is_u8());
    }

    #[test]
    fn conversions_i16() {
        let v = Value::from(i16::max_value());
        assert!(v.is_i128());
        assert!(v.is_u128());
        assert!(v.is_i64());
        assert!(v.is_u64());
        assert!(v.is_i32());
        assert!(v.is_u32());
        assert!(v.is_i16());
        assert!(v.is_u16());
        assert!(!v.is_i8());
        assert!(!v.is_u8());
        let v = Value::from(i16::min_value());
        assert!(v.is_i128());
        assert!(!v.is_u128());
        assert!(v.is_i64());
        assert!(!v.is_u64());
        assert!(v.is_i32());
        assert!(!v.is_u32());
        assert!(v.is_i16());
        assert!(!v.is_u16());
        assert!(!v.is_i8());
        assert!(!v.is_u8());
    }

    #[test]
    fn conversions_i8() {
        let v = Value::from(i8::max_value());
        assert!(v.is_i128());
        assert!(v.is_u128());
        assert!(v.is_i64());
        assert!(v.is_u64());
        assert!(v.is_i32());
        assert!(v.is_u32());
        assert!(v.is_i16());
        assert!(v.is_u16());
        assert!(v.is_i8());
        assert!(v.is_u8());
        let v = Value::from(i8::min_value());
        assert!(v.is_i128());
        assert!(!v.is_u128());
        assert!(v.is_i64());
        assert!(!v.is_u64());
        assert!(v.is_i32());
        assert!(!v.is_u32());
        assert!(v.is_i16());
        assert!(!v.is_u16());
        assert!(v.is_i8());
        assert!(!v.is_u8());
    }
    #[test]
    fn conversions_u64() {
        let v = Value::from(u64::min_value());
        assert!(v.is_i128());
        assert!(v.is_u128());
        assert!(v.is_i64());
        assert!(v.is_u64());
        assert!(v.is_i32());
        assert!(v.is_u32());
        assert!(v.is_i16());
        assert!(v.is_u16());
        assert!(v.is_i8());
        assert!(v.is_u8());
    }

    #[test]
    fn conversions_u32() {
        let v = Value::from(u32::max_value());
        assert!(v.is_i128());
        assert!(v.is_u128());
        assert!(v.is_i64());
        assert!(v.is_u64());
        assert!(!v.is_i32());
        assert!(v.is_u32());
        assert!(!v.is_i16());
        assert!(!v.is_u16());
        assert!(!v.is_i8());
        assert!(!v.is_u8());
    }

    #[test]
    fn conversions_u16() {
        let v = Value::from(u16::max_value());
        assert!(v.is_i128());
        assert!(v.is_u128());
        assert!(v.is_i64());
        assert!(v.is_u64());
        assert!(v.is_i32());
        assert!(v.is_u32());
        assert!(!v.is_i16());
        assert!(v.is_u16());
        assert!(!v.is_i8());
        assert!(!v.is_u8());
    }

    #[test]
    fn conversions_u8() {
        let v = Value::from(u8::max_value());
        assert!(v.is_i128());
        assert!(v.is_u128());
        assert!(v.is_i64());
        assert!(v.is_u64());
        assert!(v.is_i32());
        assert!(v.is_u32());
        assert!(v.is_i16());
        assert!(v.is_u16());
        assert!(!v.is_i8());
        assert!(v.is_u8());
    }
}
