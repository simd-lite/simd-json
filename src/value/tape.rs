/// A tape of a parsed json, all values are extracted and validated and
/// can be used without further computation.
mod cmp;
mod from;
use crate::{Value as ValueTrait, ValueType};
use float_cmp::approx_eq;
use halfbrown::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::ops::{Index, IndexMut};

/// `Tape`
pub struct Tape<'input>(Vec<Node<'input>>);

/// Tape `Node`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Node<'input> {
    /// A string, located inside the input slice
    String(&'input str),
    /// An `Object` with the given `size` starts here.
    /// the following values are keys and values, alternating
    /// however values can be nested and have a length thsemlfs.
    Object(usize, usize),
    /// An array with a given size starts here. The next `size`
    /// elements belong to it - values can be nested and have a
    /// `size` of their own.
    Array(usize, usize),
    /// A static value that is interned into the tape, it can
    /// be directly taken and isn't nested.
    Static(StaticNode),
}

/// Static tape node
#[derive(Debug, Clone, Copy)]
pub enum StaticNode {
    /// A signed 64 bit integer.
    I64(i64),
    #[cfg(feature = "128bit")]
    /// A signed 128 bit integer.
    I128(i128),
    /// An unsigned 64 bit integer.
    U64(u64),
    #[cfg(feature = "128bit")]
    /// An unsigned 128 bit integer.
    U128(u128),
    /// A floating point value
    F64(f64),
    /// A boolean value
    Bool(bool),
    /// The null value
    Null,
}

impl Index<&str> for StaticNode {
    type Output = ();
    #[inline]
    fn index(&self, _index: &str) -> &Self::Output {
        panic!("Not supported")
    }
}

impl Index<usize> for StaticNode {
    type Output = ();
    #[inline]
    fn index(&self, _index: usize) -> &Self::Output {
        panic!("Not supported")
    }
}

impl IndexMut<&str> for StaticNode {
    #[inline]
    fn index_mut(&mut self, _index: &str) -> &mut Self::Output {
        panic!("Not supported")
    }
}

impl IndexMut<usize> for StaticNode {
    #[inline]
    fn index_mut(&mut self, _index: usize) -> &mut Self::Output {
        panic!("Not supported")
    }
}

impl ValueTrait for StaticNode {
    type Key = ();

    #[cfg(not(feature = "128bit"))]
    #[inline]
    fn value_type(&self) -> ValueType {
        match self {
            Self::Null => ValueType::Null,
            Self::Bool(_) => ValueType::Bool,
            Self::F64(_) => ValueType::F64,
            Self::I64(_) => ValueType::I64,

            Self::U64(_) => ValueType::U64,
        }
    }

    #[cfg(feature = "128bit")]
    #[inline]
    fn value_type(&self) -> ValueType {
        match self {
            Self::Null => ValueType::Null,
            Self::Bool(_) => ValueType::Bool,
            Self::F64(_) => ValueType::F64,
            Self::I128(_) => ValueType::I128,
            Self::I64(_) => ValueType::I64,
            Self::U128(_) => ValueType::U128,
            Self::U64(_) => ValueType::U64,
        }
    }

    #[inline]
    fn is_null(&self) -> bool {
        self == &Self::Null
    }

    #[inline]
    fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    #[cfg(not(feature = "128bit"))]
    #[inline]
    fn as_i64(&self) -> Option<i64> {
        match self {
            Self::I64(i) => Some(*i),
            Self::U64(i) => i64::try_from(*i).ok(),
            _ => None,
        }
    }

    #[cfg(feature = "128bit")]
    #[inline]
    fn as_i64(&self) -> Option<i64> {
        match self {
            Self::I64(i) => Some(*i),
            Self::U64(i) => i64::try_from(*i).ok(),
            Self::I128(i) => i64::try_from(*i).ok(),
            Self::U128(i) => i64::try_from(*i).ok(),
            _ => None,
        }
    }

    #[cfg(feature = "128bit")]
    #[inline]
    fn as_i128(&self) -> Option<i128> {
        match self {
            Self::I128(i) => Some(*i),
            Self::U128(i) => i128::try_from(*i).ok(),
            Self::I64(i) => Some(i128::from(*i)),
            Self::U64(i) => i128::try_from(*i).ok(),
            _ => None,
        }
    }

    #[cfg(not(feature = "128bit"))]
    #[inline]
    #[allow(clippy::cast_sign_loss)]
    fn as_u64(&self) -> Option<u64> {
        match self {
            Self::I64(i) => u64::try_from(*i).ok(),
            Self::U64(i) => Some(*i),
            _ => None,
        }
    }

    #[cfg(feature = "128bit")]
    #[inline]
    #[allow(clippy::cast_sign_loss)]
    fn as_u64(&self) -> Option<u64> {
        match self {
            Self::I64(i) => u64::try_from(*i).ok(),
            Self::U64(i) => Some(*i),
            Self::I128(i) => u64::try_from(*i).ok(),
            Self::U128(i) => u64::try_from(*i).ok(),
            _ => None,
        }
    }
    #[cfg(feature = "128bit")]
    #[inline]
    #[allow(clippy::cast_sign_loss)]
    fn as_u128(&self) -> Option<u128> {
        match self {
            Self::U128(i) => Some(*i),
            Self::I128(i) => u128::try_from(*i).ok(),
            Self::I64(i) => u128::try_from(*i).ok(),
            Self::U64(i) => Some(u128::from(*i)),
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

    #[cfg(not(feature = "128bit"))]
    #[inline]
    #[allow(clippy::cast_precision_loss)]
    fn cast_f64(&self) -> Option<f64> {
        match self {
            Self::F64(i) => Some(*i),
            Self::I64(i) => Some(*i as f64),
            Self::U64(i) => Some(*i as f64),
            _ => None,
        }
    }

    #[cfg(feature = "128bit")]
    #[inline]
    #[allow(clippy::cast_precision_loss)]
    fn cast_f64(&self) -> Option<f64> {
        match self {
            Self::F64(i) => Some(*i),
            Self::I64(i) => Some(*i as f64),
            Self::U64(i) => Some(*i as f64),
            Self::I128(i) => Some(*i as f64),
            Self::U128(i) => Some(*i as f64),
            _ => None,
        }
    }
    #[inline]
    fn as_str(&self) -> Option<&str> {
        None
    }
    #[inline]
    fn as_array(&self) -> Option<&Vec<Self>> {
        None
    }
    #[inline]
    fn as_object(&self) -> Option<&HashMap<Self::Key, Self>> {
        None
    }
}

#[cfg_attr(tarpaulin, skip)]
impl<'v> fmt::Display for StaticNode {
    #[cfg(not(feature = "128bit"))]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::I64(n) => write!(f, "{}", n),
            Self::U64(n) => write!(f, "{}", n),
            Self::F64(n) => write!(f, "{}", n),
        }
    }
    #[cfg(feature = "128bit")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::I64(n) => write!(f, "{}", n),
            Self::U64(n) => write!(f, "{}", n),
            Self::F64(n) => write!(f, "{}", n),
            Self::I128(n) => write!(f, "{}", n),
            Self::U128(n) => write!(f, "{}", n),
        }
    }
}

#[allow(clippy::cast_sign_loss, clippy::default_trait_access)]
impl<'a> PartialEq for StaticNode {
    #[cfg(not(feature = "128bit"))]
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(v1), Self::Bool(v2)) => v1.eq(v2),
            (Self::F64(v1), Self::F64(v2)) => approx_eq!(f64, *v1, *v2),
            (Self::U64(v1), Self::U64(v2)) => v1.eq(v2),
            (Self::I64(v1), Self::I64(v2)) => v1.eq(v2),
            (Self::U64(v1), Self::I64(v2)) if *v2 >= 0 => (*v2 as u64).eq(v1),
            (Self::I64(v1), Self::U64(v2)) if *v1 >= 0 => (*v1 as u64).eq(v2),
            _ => false,
        }
    }

    #[cfg(feature = "128bit")]
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(v1), Self::Bool(v2)) => v1.eq(v2),
            (Self::F64(v1), Self::F64(v2)) => approx_eq!(f64, *v1, *v2),
            (Self::U64(v1), Self::U64(v2)) => v1.eq(v2),
            (Self::U128(v1), Self::U128(v2)) => v1.eq(v2),
            (Self::I64(v1), Self::I64(v2)) => v1.eq(v2),
            (Self::I128(v1), Self::I128(v2)) => v1.eq(v2),

            (Self::U64(v1), Self::I64(v2)) if *v2 >= 0 => (*v2 as u64).eq(v1),
            (Self::U64(v1), Self::I128(v2)) if *v2 >= 0 => (*v2 as u128).eq(&u128::from(*v1)),
            (Self::U64(v1), Self::U128(v2)) => v2.eq(&u128::from(*v1)),

            (Self::I64(v1), Self::U64(v2)) if *v1 >= 0 => (*v1 as u64).eq(v2),
            (Self::I64(v1), Self::I128(v2)) => (*v2 as i128).eq(&i128::from(*v1)),
            (Self::I64(v1), Self::U128(v2)) if *v1 >= 0 => v2.eq(&(*v1 as u128)),

            (Self::U128(v1), Self::I128(v2)) if *v2 >= 0 => (*v2 as u128).eq(v1),
            (Self::U128(v1), Self::U64(v2)) => v1.eq(&u128::from(*v2)),
            (Self::U128(v1), Self::I64(v2)) if *v2 >= 0 => v1.eq(&(*v2 as u128)),

            (Self::I128(v1), Self::U128(v2)) if *v1 >= 0 => (*v1 as u128).eq(v2),
            (Self::I128(v1), Self::U64(v2)) => v1.eq(&i128::from(*v2)),
            (Self::I128(v1), Self::I64(v2)) => v1.eq(&i128::from(*v2)),
            _ => false,
        }
    }
}

impl Default for StaticNode {
    fn default() -> Self {
        Self::Null
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::cognitive_complexity)]
    use super::StaticNode as Value;
    use super::*;
    use crate::value::Value as ValueTrait;

    #[test]
    #[should_panic]
    #[allow(unused_variables)]
    fn object_index() {
        let v = StaticNode::Null;
        let a = v["test"];
    }

    #[test]
    #[should_panic]
    fn mut_object_index() {
        let mut v = StaticNode::Null;
        v["test"] = ();
    }

    #[test]
    #[should_panic]
    #[allow(unused_variables)]
    fn array_index() {
        let v = StaticNode::Null;
        let a = v[0];
    }

    #[test]
    #[should_panic]
    fn mut_array_index() {
        let mut v = StaticNode::Null;
        v[0] = ();
    }

    #[test]
    fn conversion_obj() {
        let v = StaticNode::Null;
        assert!(!v.is_object())
    }

    #[test]
    fn conversion_arr() {
        let v = StaticNode::Null;
        assert!(!v.is_array())
    }

    #[test]
    fn conversion_str() {
        let v = StaticNode::Null;
        assert!(!v.is_str())
    }
    #[cfg(feature = "128bit")]
    #[test]
    fn conversions_i128() {
        let v = Value::from(i128::max_value());
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
        let v = Value::from(i128::min_value());
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

    #[cfg(feature = "128bit")]
    #[test]
    fn conversions_u128() {
        let v = Value::from(u128::min_value());
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
        let v = Value::from(());
        assert!(!v.is_f64_castable());
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
        assert_eq!(Value::default(), Value::Null)
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
