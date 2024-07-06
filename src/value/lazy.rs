use crate::cow::Cow;
use crate::prelude::*;
use crate::{borrowed, tape};
use std::fmt;

mod array;
mod cmp;
mod from;
mod object;

pub use array::Array;
pub use object::Object;

/// A lazy value, this gets initialized with a tape and as long as only non mutating operations are
/// performed it will stay a tape. If a mutating operation is performed it will upgrade to a borrowed
/// value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value<'tape, 'input> {
    /// tape variant
    Tape(tape::Value<'tape, 'input>),
    /// borrowed variant
    Value(Cow<'tape, borrowed::Value<'input>>),
}

impl Default for Value<'static, '_> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn default() -> Self {
        Value::Value(Cow::Owned(borrowed::Value::default()))
    }
}

impl<'tape, 'input> Value<'tape, 'input> {
    /// Creates a new lazy Value from a tape
    #[must_use]
    pub fn from_tape(tape: tape::Value<'tape, 'input>) -> Self {
        Value::Tape(tape)
    }
    unsafe fn into_tape(self) -> tape::Value<'tape, 'input> {
        match self {
            Value::Tape(tape) => tape,
            Value::Value(_) => unreachable!("we know we are not a value"),
        }
    }

    fn upgrade(&mut self) {
        if let Value::Value(_) = &self {
            return;
        }
        let mut dummy = Value::Tape(tape::Value::null());
        std::mem::swap(self, &mut dummy);
        let tape = unsafe { dummy.into_tape() };

        let value = super::borrowed::BorrowSliceDeserializer::from_tape(tape.0).parse();

        *self = Value::Value(Cow::Owned(value));
    }

    fn is_tape(&self) -> bool {
        match &self {
            Value::Tape(_) => true,
            Value::Value(_) => false,
        }
    }

    fn as_mut(&mut self) -> &mut borrowed::Value<'input> {
        if self.is_tape() {
            self.upgrade();
        }

        if let Value::Value(value) = self {
            value.to_mut()
        } else {
            unreachable!()
        }
    }
}

impl<'value> ValueBuilder<'value> for Value<'static, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn null() -> Self {
        Value::Value(Cow::Owned(borrowed::Value::null()))
    }
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn array_with_capacity(capacity: usize) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::array_with_capacity(capacity)))
    }
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn object_with_capacity(capacity: usize) -> Self {
        Value::Value(Cow::Owned(borrowed::Value::object_with_capacity(capacity)))
    }
}

impl<'tape, 'value> ValueAsMutContainer for Value<'tape, 'value> {
    type Array = Vec<borrowed::Value<'value>>;
    type Object = super::borrowed::Object<'value>;
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn as_array_mut(&mut self) -> Option<&mut Vec<borrowed::Value<'value>>> {
        self.as_mut().as_array_mut()
    }
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn as_object_mut(&mut self) -> Option<&mut super::borrowed::Object<'value>> {
        self.as_mut().as_object_mut()
    }
}

impl<'tape, 'value> TypedValue for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn value_type(&self) -> ValueType {
        match &self {
            Value::Tape(tape) => tape.value_type(),
            Value::Value(value) => value.value_type(),
        }
    }
}

impl<'tape, 'value> ValueAsScalar for Value<'tape, 'value> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn as_null(&self) -> Option<()> {
        match &self {
            Value::Tape(tape) => tape.as_null(),
            Value::Value(value) => value.as_null(),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn as_bool(&self) -> Option<bool> {
        match &self {
            Value::Tape(tape) => tape.as_bool(),
            Value::Value(value) => value.as_bool(),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn as_i64(&self) -> Option<i64> {
        match &self {
            Value::Tape(tape) => tape.as_i64(),
            Value::Value(value) => value.as_i64(),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn as_i128(&self) -> Option<i128> {
        match &self {
            Value::Tape(tape) => tape.as_i128(),
            Value::Value(value) => value.as_i128(),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn as_u64(&self) -> Option<u64> {
        match &self {
            Value::Tape(tape) => tape.as_u64(),
            Value::Value(value) => value.as_u64(),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn as_u128(&self) -> Option<u128> {
        match &self {
            Value::Tape(tape) => tape.as_u128(),
            Value::Value(value) => value.as_u128(),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn as_f64(&self) -> Option<f64> {
        match &self {
            Value::Tape(tape) => tape.as_f64(),
            Value::Value(value) => value.as_f64(),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn cast_f64(&self) -> Option<f64> {
        match &self {
            Value::Tape(tape) => tape.cast_f64(),
            Value::Value(value) => value.cast_f64(),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn as_str(&self) -> Option<&str> {
        match &self {
            Value::Tape(tape) => tape.as_str(),
            Value::Value(value) => value.as_str(),
        }
    }
}

// impl<'tape, 'value> ValueAsContainer for Value<'tape, 'value> {
impl<'tape, 'value> Value<'tape, 'value> {
    // type Array = array::Array<'tape, 'value>;
    // type Object = Object<'tape, 'value>;

    /// Tries to represent the value as an array and returns a reference to it
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    pub fn as_array(&self) -> Option<array::Array<'_, 'value>> {
        match self {
            Value::Tape(tape) => tape.as_array().map(Array::Tape),
            Value::Value(value) => value.as_array().map(array::Array::Value),
        }
    }

    /// Tries to represent the value as an array and returns a reference to it
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    pub fn as_object(&self) -> Option<object::Object> {
        match self {
            Value::Tape(tape) => tape.as_object().map(Object::Tape),
            Value::Value(value) => value.as_object().map(Object::Value),
        }
    }
}

impl<'tape, 'value> ValueIntoString for Value<'tape, 'value> {
    type String = Cow<'value, str>;

    fn into_string(self) -> Option<<Self as ValueIntoString>::String> {
        match self {
            Value::Tape(tape) => tape.into_string().map(Cow::Borrowed),
            // This is a bit complex but it allows us to avoid cloning
            Value::Value(value) => match value {
                Cow::Borrowed(value) => match value {
                    borrowed::Value::String(s) => Some(s.clone()),
                    _ => None,
                },
                Cow::Owned(value) => match value {
                    borrowed::Value::String(s) => Some(s),
                    _ => None,
                },
            },
        }
    }
}

// impl<'value> ValueIntoContainer for Value<'value> {
//     type Array = Vec<Self>;
//     type Object = Object<'value>;

//     fn into_array(self) -> Option<<Self as ValueIntoContainer>::Array> {
//         match self {
//             Self::Array(a) => Some(a),
//             _ => None,
//         }
//     }

//     fn into_object(self) -> Option<<Self as ValueIntoContainer>::Object> {
//         match self {
//             Self::Object(a) => Some(*a),
//             _ => None,
//         }
//     }
// }

#[cfg(not(tarpaulin_include))]
impl<'tape, 'value> fmt::Display for Value<'tape, 'value> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Value::Tape(tape) => write!(f, "{tape:?}"),
            Value::Value(value) => write!(f, "{value}"),
        }
    }
}

// impl<'value> Index<&str> for Value<'value> {
//     type Output = Value<'value>;
//     #[cfg_attr(not(feature = "no-inline"), inline)]
//     #[must_use]
//     fn index(&self, index: &str) -> &Self::Output {
//         self.get(index).expect("index out of bounds")
//     }
// }

// impl<'value> Index<usize> for Value<'value> {
//     type Output = Value<'value>;
//     #[cfg_attr(not(feature = "no-inline"), inline)]
//     #[must_use]
//     fn index(&self, index: usize) -> &Self::Output {
//         self.get_idx(index).expect("index out of bounds")
//     }
// }

// impl<'value> IndexMut<&str> for Value<'value> {
//     #[cfg_attr(not(feature = "no-inline"), inline)]
//     #[must_use]
//     fn index_mut(&mut self, index: &str) -> &mut Self::Output {
//         self.get_mut(index).expect("index out of bounds")
//     }
// }

// impl<'value> IndexMut<usize> for Value<'value> {
//     #[cfg_attr(not(feature = "no-inline"), inline)]
//     #[must_use]
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         self.get_idx_mut(index).expect("index out of bounds")
//     }
// }

#[cfg(test)]
mod test {
    #![allow(clippy::cognitive_complexity)]
    use super::Value;
    use super::*;

    #[test]
    #[should_panic = "Not supported"]
    #[allow(unused_variables, clippy::no_effect)]
    fn object_index() {
        let v = StaticNode::Null;
        v["test"];
    }

    #[test]
    #[should_panic = "Not supported"]
    fn mut_object_index() {
        let mut v = StaticNode::Null;
        v["test"] = ();
    }

    #[test]
    #[should_panic = "Not supported"]
    #[allow(unused_variables, clippy::no_effect)]
    fn array_index() {
        let v = StaticNode::Null;
        v[0];
    }

    #[test]
    #[should_panic = "Not supported"]
    fn mut_array_index() {
        let mut v = StaticNode::Null;
        v[0] = ();
    }

    #[test]
    fn conversion_str() {
        let v = StaticNode::Null;
        assert!(!v.is_str());
    }
    #[cfg(feature = "128bit")]
    #[test]
    fn conversions_i128() {
        let v = Value::from(i128::MAX);
        assert!(v.is_i128());
        assert!(v.is_u128());
        assert!(!v.is_i64());
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
        let v = Value::from(i128::MIN);
        assert!(v.is_i128());
        assert!(!v.is_u128());
        assert!(!v.is_i64());
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
    fn conversions_i64() {
        let v = Value::from(i64::MAX);
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
        let v = Value::from(i64::MIN);
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
        let v = Value::from(i32::MAX);
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
        let v = Value::from(i32::MIN);
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
        let v = Value::from(i16::MAX);
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
        let v = Value::from(i16::MIN);
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
        let v = Value::from(i8::MAX);
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
        let v = Value::from(i8::MIN);
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
        let v = Value::from(usize::MIN as u64);
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

    #[cfg(feature = "128bit")]
    #[test]
    fn conversions_u128() {
        let v = Value::from(u128::MIN);
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
    fn conversions_u64() {
        let v = Value::from(u64::MIN);
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
        let v = Value::from(u32::MAX);
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
        let v = Value::from(u16::MAX);
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
        let v = Value::from(u8::MAX);
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
        let v = Value::from(f64::MAX);
        assert!(!v.is_i64());
        assert!(!v.is_u64());
        assert!(v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
        let v = Value::from(f64::MIN);
        assert!(!v.is_i64());
        assert!(!v.is_u64());
        assert!(v.is_f64());
        assert!(!v.is_f32());
        assert!(v.is_f64_castable());
        let v = Value::from(());
        assert!(!v.is_f64_castable());
    }

    #[test]
    fn conversions_f32() {
        let v = Value::from(f32::MAX);
        assert!(!v.is_i64());
        assert!(!v.is_u64());
        assert!(v.is_f64());
        assert!(v.is_f32());
        assert!(v.is_f64_castable());
        let v = Value::from(f32::MIN);
        assert!(!v.is_i64());
        assert!(!v.is_u64());
        assert!(v.is_f64());
        assert!(v.is_f32());
        assert!(v.is_f64_castable());
    }

    #[test]
    fn conversions_bool() {
        let v = Value::from(true);
        assert!(v.is_bool());
        assert_eq!(v.value_type(), ValueType::Bool);
        let v = Value::from(());
        assert!(!v.is_bool());
    }

    #[test]
    fn conversions_float() {
        let v = Value::from(42.0);
        assert!(v.is_f64());
        assert_eq!(v.value_type(), ValueType::F64);
        let v = Value::from(());
        assert!(!v.is_f64());
    }

    #[test]
    fn conversions_int() {
        let v = Value::from(-42);
        assert!(v.is_i64());
        assert_eq!(v.value_type(), ValueType::I64);
        #[cfg(feature = "128bit")]
        {
            let v = Value::from(-42_i128);
            assert!(v.is_i64());
            assert!(v.is_i128());
            assert_eq!(v.value_type(), ValueType::I128);
        }
        let v = Value::from(());
        assert!(!v.is_i64());
        assert!(!v.is_i128());
    }

    #[test]
    fn conversions_uint() {
        let v = Value::from(42_u64);
        assert!(v.is_u64());
        assert_eq!(v.value_type(), ValueType::U64);
        #[cfg(feature = "128bit")]
        {
            let v = Value::from(42_u128);
            assert!(v.is_u64());
            assert!(v.is_u128());
            assert_eq!(v.value_type(), ValueType::U128);
        }
        let v = Value::from(());
        assert!(!v.is_u64());
        assert!(!v.is_u128());
    }

    #[test]
    fn conversions_null() {
        let v = Value::from(());
        assert!(v.is_null());
        assert_eq!(v.value_type(), ValueType::Null);
        let v = Value::from(1);
        assert!(!v.is_null());
    }

    #[test]
    fn default() {
        assert_eq!(Value::default(), Value::null());
    }

    #[test]
    fn mixed_int_cmp() {
        assert_eq!(Value::from(1_u64), Value::from(1_i64));
        assert_eq!(Value::from(1_i64), Value::from(1_u64));
    }

    #[test]
    #[cfg(feature = "128bit")]
    fn mixed_int_cmp_128() {
        assert_eq!(Value::from(1_u64), Value::from(1_u128));
        assert_eq!(Value::from(1_u64), Value::from(1_i128));
        assert_eq!(Value::from(1_i64), Value::from(1_u128));
        assert_eq!(Value::from(1_i64), Value::from(1_i128));

        assert_eq!(Value::from(1_u128), Value::from(1_u128));
        assert_eq!(Value::from(1_u128), Value::from(1_i128));
        assert_eq!(Value::from(1_u128), Value::from(1_u64));
        assert_eq!(Value::from(1_u128), Value::from(1_i64));

        assert_eq!(Value::from(1_i128), Value::from(1_u128));
        assert_eq!(Value::from(1_i128), Value::from(1_i128));
        assert_eq!(Value::from(1_i128), Value::from(1_u64));
        assert_eq!(Value::from(1_i128), Value::from(1_i64));
    }

    #[test]
    fn test_union_cmp() {
        let v: Value = ().into();
        assert_eq!(v, ());
    }
    #[test]
    #[allow(clippy::bool_assert_comparison)]
    fn test_bool_cmp() {
        let v: Value = true.into();
        assert_eq!(v, true);
        let v: Value = false.into();
        assert_eq!(v, false);
    }
}
