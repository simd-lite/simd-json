/// A lifetime less DOM implementation. It uses strings to make te
/// structure fully owned, avoiding lifetimes at the cost of performance.
mod cmp;
mod from;
mod serialize;

use crate::value::{ValueTrait, ValueType};
use crate::{stry, unlikely, Deserializer, ErrorType, Result};
use halfbrown::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::ops::Index;

/// Representation of a JSON object
pub type Object = HashMap<String, Value>;

/// Parses a slice of bytes into a Value dom. This function will
/// rewrite the slice to de-escape strings.
/// We do not keep any references to the raw data but re-allocate
/// owned memory whereever required thus returning a value without
/// a lifetime.
pub fn to_value(s: &mut [u8]) -> Result<Value> {
    let de = stry!(Deserializer::from_slice(s));
    OwnedDeserializer::from_deserializer(de).parse()
}

/// Owned JSON-DOM Value, consider using the `ValueTrait`
/// to access it's content.
/// This is slower then the `BorrowedValue` as a tradeoff
/// for getting rid of lifetimes.
#[derive(Debug, Clone)]
pub enum Value {
    /// null
    Null,
    /// boolean type
    Bool(bool),
    /// float type
    F64(f64),
    /// signed integer type
    I64(i64),
    /// unsigned integer type
    U64(u64),
    /// string type
    String(String),
    /// array type
    Array(Vec<Value>),
    /// object type
    Object(Object),
}

impl ValueTrait for Value {
    type Key = String;

    fn value_type(&self) -> ValueType {
        match self {
            Self::Null => ValueType::Null,
            Self::Bool(_) => ValueType::Bool,
            Self::F64(_) => ValueType::F64,
            Self::I64(_) => ValueType::I64,
            Self::U64(_) => ValueType::U64,
            Self::String(_) => ValueType::String,
            Self::Array(_) => ValueType::Array,
            Self::Object(_) => ValueType::Object,
        }
    }

    #[inline]
    fn is_null(&self) -> bool {
        match self {
            Self::Null => true,
            _ => false,
        }
    }

    #[inline]
    fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    #[inline]
    fn as_i64(&self) -> Option<i64> {
        match self {
            Self::I64(i) => Some(*i),
            Self::U64(i) => i64::try_from(*i).ok(),
            _ => None,
        }
    }

    #[inline]
    fn as_u64(&self) -> Option<u64> {
        #[allow(clippy::cast_sign_loss)]
        match self {
            Self::I64(i) => u64::try_from(*i).ok(),
            Self::U64(i) => Some(*i),
            _ => None,
        }
    }

    #[inline]
    fn as_f64(&self) -> Option<f64> {
        match self {
            Self::F64(i) => Some(*i),
            _ => None,
        }
    }

    #[inline]
    fn cast_f64(&self) -> Option<f64> {
        #[allow(clippy::cast_precision_loss)]
        match self {
            Self::F64(i) => Some(*i),
            Self::I64(i) => Some(*i as f64),
            Self::U64(i) => Some(*i as f64),
            _ => None,
        }
    }

    #[inline]
    fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    #[inline]
    fn as_array(&self) -> Option<&Vec<Self>> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    #[inline]
    fn as_array_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    #[inline]
    fn as_object(&self) -> Option<&HashMap<Self::Key, Self>> {
        match self {
            Self::Object(m) => Some(m),
            _ => None,
        }
    }

    #[inline]
    fn as_object_mut(&mut self) -> Option<&mut HashMap<Self::Key, Self>> {
        match self {
            Self::Object(m) => Some(m),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Null => f.write_str("null"),
            Self::Bool(false) => f.write_str("false"),
            Self::Bool(true) => f.write_str("true"),
            Self::I64(n) => f.write_str(&n.to_string()),
            Self::U64(n) => f.write_str(&n.to_string()),
            Self::F64(n) => f.write_str(&n.to_string()),
            Self::String(s) => write!(f, "{}", s),
            Self::Array(a) => write!(f, "{:?}", a),
            Self::Object(o) => write!(f, "{:?}", o),
        }
    }
}

impl Index<&str> for Value {
    type Output = Self;
    fn index(&self, index: &str) -> &Self {
        static NULL: Value = Value::Null;
        self.get(index).unwrap_or(&NULL)
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Null
    }
}

struct OwnedDeserializer<'de> {
    de: Deserializer<'de>,
}

impl<'de> OwnedDeserializer<'de> {
    pub fn from_deserializer(de: Deserializer<'de>) -> Self {
        Self { de }
    }
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    pub fn parse(&mut self) -> Result<Value> {
        match self.de.next_() {
            b'"' => self.de.parse_str_().map(Value::from),
            b'n' => Ok(Value::Null),
            b't' => Ok(Value::Bool(true)),
            b'f' => Ok(Value::Bool(false)),
            b'-' => self.de.parse_number_root(true).map(Value::from),
            b'0'..=b'9' => self.de.parse_number_root(false).map(Value::from),
            b'[' => self.parse_array(),
            b'{' => self.parse_map(),
            _c => Err(self.de.error(ErrorType::UnexpectedCharacter)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_value(&mut self) -> Result<Value> {
        match self.de.next_() {
            b'"' => self.de.parse_str_().map(Value::from),
            b'n' => Ok(Value::Null),
            b't' => Ok(Value::Bool(true)),
            b'f' => Ok(Value::Bool(false)),
            b'-' => self.de.parse_number(true).map(Value::from),
            b'0'..=b'9' => self.de.parse_number(false).map(Value::from),
            b'[' => self.parse_array(),
            b'{' => self.parse_map(),
            _c => Err(self.de.error(ErrorType::UnexpectedCharacter)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_array(&mut self) -> Result<Value> {
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
    fn parse_map(&mut self) -> Result<Value> {
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
    #![allow(clippy::cognitive_complexity)]
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
        assert!(!v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
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
        assert!(!v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
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
        assert!(!v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
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
        assert!(!v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
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
        assert!(!v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
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
        assert!(!v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
        assert!(v.is_f64_castable());
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
        assert!(!v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
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
        assert!(!v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
    }

    #[test]
    fn conversions_usize() {
        let v = Value::from(usize::min_value() as u64);
        assert!(v.is_i128());
        assert!(v.is_u128());
        assert!(v.is_i64());
        assert!(v.is_u64());
        assert!(v.is_usize());
        assert!(v.is_i32());
        assert!(v.is_u32());
        assert!(v.is_i16());
        assert!(v.is_u16());
        assert!(v.is_i8());
        assert!(v.is_u8());
        assert!(!v.is_f64());
        assert!(!v.is_f32());
        assert!(!v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
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
        assert!(!v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
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
        assert!(!v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
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
        assert!(!v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
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
        assert!(!v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
    }

    #[test]
    fn conversions_f64() {
        let v = Value::from(std::f64::MAX);
        assert!(!v.is_i64());
        assert!(!v.is_u64());
        assert!(v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
        let v = Value::from(std::f64::MIN);
        assert!(!v.is_i64());
        assert!(!v.is_u64());
        assert!(v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
    }

    #[test]
    fn conversions_f32() {
        let v = Value::from(std::f32::MAX);
        assert!(!v.is_i64());
        assert!(!v.is_u64());
        assert!(v.is_f64());
        assert!(v.is_f32());
        assert!(v.is_f64_castable());
        let v = Value::from(std::f32::MIN);
        assert!(!v.is_i64());
        assert!(!v.is_u64());
        assert!(v.is_f64());
        assert!(v.is_f32());
        assert!(v.is_f64_castable());
    }

    #[test]
    fn conversions_array() {
        let v = Value::from(vec![true]);
        assert!(v.is_array());
        assert_eq!(v.value_type(), ValueType::Array);
    }

    #[test]
    fn conversions_bool() {
        let v = Value::from(true);
        assert!(v.is_bool());
        assert_eq!(v.value_type(), ValueType::Bool);
    }

    #[test]
    fn conversions_float() {
        let v = Value::from(42.0);
        assert!(v.is_f64());
        assert_eq!(v.value_type(), ValueType::F64);
    }

    #[test]
    fn conversions_int() {
        let v = Value::from(42);
        assert!(v.is_i64());
        assert_eq!(v.value_type(), ValueType::I64);
    }

    #[test]
    fn conversions_null() {
        let v = Value::from(());
        assert!(v.is_null());
        assert_eq!(v.value_type(), ValueType::Null);
    }

    #[test]
    fn conversions_object() {
        let v = Value::from(Object::new());
        assert!(v.is_object());
        assert_eq!(v.value_type(), ValueType::Object);
    }

    #[test]
    fn conversions_str() {
        let v = Value::from("bla");
        assert!(v.is_str());
        assert_eq!(v.value_type(), ValueType::String);
    }
    use proptest::prelude::*;
    fn arb_value() -> BoxedStrategy<Value> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::Bool),
            any::<i64>().prop_map(Value::I64),
            any::<f64>().prop_map(Value::F64),
            ".*".prop_map(Value::from),
        ];
        leaf.prop_recursive(
            8,   // 8 levels deep
            256, // Shoot for maximum size of 256 nodes
            10,  // We put up to 10 items per collection
            |inner| {
                prop_oneof![
                    // Take the inner strategy and make the two recursive cases.
                    prop::collection::vec(inner.clone(), 0..10).prop_map(Value::Array),
                    prop::collection::hash_map(".*", inner.clone(), 0..10)
                        .prop_map(|m| Value::Object(m.into_iter().collect())),
                ]
            },
        )
        .boxed()
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            .. ProptestConfig::default()
        })]

        #[test]
        fn prop_to_owned(owned in arb_value()) {
            use crate::BorrowedValue;
            let borrowed: BorrowedValue = owned.clone().into();
            prop_assert_eq!(owned, borrowed);
        }

        #[test]
        fn prop_serialize_deserialize(owned in arb_value()) {
            let mut string = owned.encode();
            let mut bytes = unsafe{ string.as_bytes_mut()};
            let decoded = to_value(&mut bytes).expect("Failed to decode");
            prop_assert_eq!(owned, decoded)
        }
        #[test]
        fn prop_f64_cmp(f in proptest::num::f64::NORMAL) {
            #[allow(clippy::float_cmp)]
            let v: Value = f.into();
            prop_assert_eq!(v, f)

        }

        #[test]
        fn prop_f32_cmp(f in proptest::num::f32::NORMAL) {
            #[allow(clippy::float_cmp)]
            let v: Value = f.into();
            prop_assert_eq!(v, f)

        }
        #[test]
        fn prop_i64_cmp(f in proptest::num::i64::ANY) {
            let v: Value = f.into();
            prop_assert_eq!(v, f)
        }
        #[test]
        fn prop_i32_cmp(f in proptest::num::i32::ANY) {
            let v: Value = f.into();
            prop_assert_eq!(v, f)
        }
        #[test]
        fn prop_i16_cmp(f in proptest::num::i16::ANY) {
            let v: Value = f.into();
            prop_assert_eq!(v, f)
        }
        #[test]
        fn prop_i8_cmp(f in proptest::num::i8::ANY) {
            let v: Value = f.into();
            prop_assert_eq!(v, f)
        }
        #[test]
        fn prop_u64_cmp(f in proptest::num::u64::ANY) {
            let v: Value = f.into();
            prop_assert_eq!(v, f)
        }

        #[allow(clippy::cast_possible_truncation)]
        #[test]
        fn prop_usize_cmp(f in proptest::num::usize::ANY) {
            let v: Value = f.into();
            prop_assert_eq!(v, f)
        }
         #[test]
        fn prop_u32_cmp(f in proptest::num::u32::ANY) {
            let v: Value = f.into();
            prop_assert_eq!(v, f)
        }
        #[test]
        fn prop_u16_cmp(f in proptest::num::u16::ANY) {
            let v: Value = f.into();
            prop_assert_eq!(v, f)
        }
        #[test]
        fn prop_u8_cmp(f in proptest::num::u8::ANY) {
            let v: Value = f.into();
            prop_assert_eq!(v.clone(), &f);
            prop_assert_eq!(v, f);
        }
        #[test]
        fn prop_string_cmp(f in ".*") {
            let v: Value = f.clone().into();
            prop_assert_eq!(v.clone(), f.as_str());
            prop_assert_eq!(v, f);
        }

    }
    #[test]
    fn test_union_cmp() {
        let v: Value = ().into();
        assert_eq!(v, ())
    }
    #[test]
    fn test_bool_cmp() {
        let v: Value = true.into();
        assert_eq!(v, true);
        let v: Value = false.into();
        assert_eq!(v, false);
    }
}
