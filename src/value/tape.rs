/// A tape of a parsed json, all values are extracted and validated and
/// can be used without further computation.
mod cmp;
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

    #[inline]
    fn value_type(&self) -> ValueType {
        self.value_type()
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

    #[inline]
    fn as_i64(&self) -> Option<i64> {
        match self {
            Self::I64(i) => Some(*i),
            Self::U64(i) => i64::try_from(*i).ok(),
            #[cfg(feature = "128bit")]
            Self::I128(i) => i64::try_from(*i).ok(),
            #[cfg(feature = "128bit")]
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

    #[inline]
    #[allow(clippy::cast_sign_loss)]
    fn as_u64(&self) -> Option<u64> {
        match self {
            Self::I64(i) => u64::try_from(*i).ok(),
            Self::U64(i) => Some(*i),
            #[cfg(feature = "128bit")]
            Self::I128(i) => u64::try_from(*i).ok(),
            #[cfg(feature = "128bit")]
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

    #[inline]
    #[allow(clippy::cast_precision_loss)]
    fn cast_f64(&self) -> Option<f64> {
        match self {
            Self::F64(i) => Some(*i),
            Self::I64(i) => Some(*i as f64),
            Self::U64(i) => Some(*i as f64),
            #[cfg(feature = "128bit")]
            Self::I128(i) => Some(*i as f64),
            #[cfg(feature = "128bit")]
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

impl StaticNode {
    /// The type of a static value
    pub fn value_type(&self) -> ValueType {
        match self {
            Self::Null => ValueType::Null,
            Self::Bool(_) => ValueType::Bool,
            Self::F64(_) => ValueType::F64,
            #[cfg(feature = "128bit")]
            Self::I128(_) => ValueType::I128,
            Self::U64(_) => ValueType::U64,
            #[cfg(feature = "128bit")]
            Self::U128(_) => ValueType::U128,
            Self::I64(_) => ValueType::I64,
        }
    }
}

#[cfg_attr(tarpaulin, skip)]
impl<'v> fmt::Display for StaticNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::I64(n) => write!(f, "{}", n),
            #[cfg(feature = "128bit")]
            Self::I128(n) => write!(f, "{}", n),
            Self::U64(n) => write!(f, "{}", n),
            #[cfg(feature = "128bit")]
            Self::U128(n) => write!(f, "{}", n),
            Self::F64(n) => write!(f, "{}", n),
        }
    }
}

#[allow(clippy::cast_sign_loss, clippy::default_trait_access)]
impl<'a> PartialEq for StaticNode {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(v1), Self::Bool(v2)) => v1.eq(v2),
            (Self::F64(v1), Self::F64(v2)) => approx_eq!(f64, *v1, *v2),
            (Self::U64(v1), Self::U64(v2)) => v1.eq(v2),
            #[cfg(feature = "128bit")]
            (Self::U128(v1), Self::U128(v2)) => v1.eq(v2),
            (Self::I64(v1), Self::I64(v2)) => v1.eq(v2),
            #[cfg(feature = "128bit")]
            (Self::I128(v1), Self::I128(v2)) => v1.eq(v2),

            (Self::U64(v1), Self::I64(v2)) if *v2 >= 0 => (*v2 as u64).eq(v1),
            #[cfg(feature = "128bit")]
            (Self::U64(v1), Self::I128(v2)) if *v2 >= 0 => (*v2 as u128).eq(&u128::from(*v1)),
            #[cfg(feature = "128bit")]
            (Self::U64(v1), Self::U128(v2)) => v2.eq(&u128::from(*v1)),

            (Self::I64(v1), Self::U64(v2)) if *v1 >= 0 => (*v1 as u64).eq(v2),
            #[cfg(feature = "128bit")]
            (Self::I64(v1), Self::I128(v2)) => (*v2 as i128).eq(&i128::from(*v1)),
            #[cfg(feature = "128bit")]
            (Self::I64(v1), Self::U128(v2)) if *v1 >= 0 => v2.eq(&(*v1 as u128)),

            #[cfg(feature = "128bit")]
            (Self::U128(v1), Self::I128(v2)) if *v2 >= 0 => (*v2 as u128).eq(v1),
            #[cfg(feature = "128bit")]
            (Self::U128(v1), Self::U64(v2)) => v1.eq(&u128::from(*v2)),
            #[cfg(feature = "128bit")]
            (Self::U128(v1), Self::I64(v2)) if *v2 >= 0 => v1.eq(&(*v2 as u128)),

            #[cfg(feature = "128bit")]
            (Self::I128(v1), Self::U128(v2)) if *v1 >= 0 => (*v1 as u128).eq(v2),
            #[cfg(feature = "128bit")]
            (Self::I128(v1), Self::U64(v2)) => v1.eq(&i128::from(*v2)),
            #[cfg(feature = "128bit")]
            (Self::I128(v1), Self::I64(v2)) => v1.eq(&i128::from(*v2)),
            _ => false,
        }
    }
}
