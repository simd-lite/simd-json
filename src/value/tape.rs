/// A tape of a parsed json, all values are extracted and validated and
/// can be used without further computation.
use crate::ValueType;
use float_cmp::approx_eq;
use std::fmt;
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
    /// A signed integer.
    I64(i64),
    /// An unsigned integer.
    U64(u64),
    /// A floating point value
    F64(f64),
    /// A boolean value
    Bool(bool),
    /// The null value
    Null,
}

impl StaticNode {
    /// The type of a static value
    pub fn value_type(&self) -> ValueType {
        match self {
            Self::Null => ValueType::Null,
            Self::Bool(_) => ValueType::Bool,
            Self::F64(_) => ValueType::F64,
            Self::I64(_) => ValueType::I64,
            Self::U64(_) => ValueType::U64,
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
            Self::U64(n) => write!(f, "{}", n),
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
            (Self::I64(v1), Self::I64(v2)) => v1.eq(v2),
            (Self::F64(v1), Self::F64(v2)) => approx_eq!(f64, *v1, *v2),
            (Self::U64(v1), Self::U64(v2)) => v1.eq(v2),
            // NOTE: We swap v1 and v2 here to avoid having to juggle ref's
            (Self::U64(v1), Self::I64(v2)) if *v2 >= 0 => (*v2 as u64).eq(v1),
            (Self::I64(v1), Self::U64(v2)) if *v1 >= 0 => (*v1 as u64).eq(v2),
            _ => false,
        }
    }
}
